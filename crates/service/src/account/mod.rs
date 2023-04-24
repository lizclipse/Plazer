mod auth;
mod persist;
mod schema;

pub use auth::*;
pub use persist::*;
pub use schema::*;

use async_graphql::{ComplexObject, InputObject, SimpleObject, ID};
use chrono::{DateTime, Utc};
use secrecy::SecretString;
use serde::Deserialize;
use surrealdb::sql::Thing;

/// A registered account.
#[derive(SimpleObject, Debug, Deserialize)]
#[graphql(complex)]
pub struct Account {
    #[graphql(skip)]
    id: Thing,
    /// The account's unique handle. This is used to create default names for
    /// resources and for logging in.
    ///
    /// It can be changed, but this will not change the name of any resources
    /// that were created with the old handle.
    handle: String,
    /// A timestamp indicating the last time the user revoked all of their
    /// tokens.
    ///
    /// This is used to invalidate all tokens that were issued before the
    /// revocation.
    revoked_at: Option<DateTime<Utc>>,

    #[graphql(skip)]
    pword_salt: SecretString,
    #[graphql(skip)]
    pword_hash: SecretString,
}

#[ComplexObject]
impl Account {
    /// The account's unique ID.
    async fn id(&self) -> ID {
        self.id.id.to_owned().into()
    }
}

/// The information needed to create a new account.
#[derive(InputObject, Debug)]
pub struct CreateAccount {
    /// The account's unique handle. This is used to create default names for
    /// resources and for logging in.
    handle: String,
    /// The account's password.
    pword: SecretString,
    /// An optional invite code.
    ///
    /// Whether this is required will depend on the server's configuration.
    invite: Option<String>,
}

/// The information needed to authenticate an account.
#[derive(InputObject, Debug)]
pub struct AuthCreds {
    /// The handle of the account to authenticate.
    handle: String,
    /// The account's password.
    pword: SecretString,
}

pub use private::{CurrentAccount, PartialAccount};
// Keep important authentication types in a private module to avoid leaking
// the internal fields.
mod private {
    use std::borrow::Cow;

    use async_graphql::ID;
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    use crate::error::{Error, Result};

    #[derive(Debug)]
    pub struct CurrentAccount(Inner);

    #[derive(Debug)]
    enum Inner {
        Unauthenticated,
        Authenticated(PartialAccount, DateTime<Utc>),
    }

    impl CurrentAccount {
        pub fn new(acc: PartialAccount, expiry: DateTime<Utc>) -> Self {
            Self(Inner::Authenticated(acc, expiry))
        }

        pub fn account(&self) -> Result<&PartialAccount> {
            match &self.0 {
                Inner::Unauthenticated => Err(Error::Unauthenticated),
                Inner::Authenticated(acc, expiry) => match Utc::now() >= *expiry {
                    true => Err(Error::Unauthenticated),
                    false => Ok(acc),
                },
            }
        }

        pub fn id(&self) -> Result<&ID> {
            self.account().map(|acc| &acc.id)
        }

        // TODO: remove this when it's used outside of tests
        #[cfg(test)]
        pub fn handle(&self) -> Result<&str> {
            self.account().map(|acc| acc.hdl.as_str())
        }
    }

    impl Default for CurrentAccount {
        fn default() -> Self {
            Self(Inner::Unauthenticated)
        }
    }

    /// Account information stored in the JWT.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct PartialAccount {
        id: ID,
        hdl: String,
    }

    impl PartialAccount {
        pub fn new(id: ID, hdl: String) -> Self {
            Self { id, hdl }
        }

        pub fn id(&self) -> &ID {
            &self.id
        }

        pub fn handle(&self) -> &str {
            self.hdl.as_str()
        }
    }

    impl From<PartialAccount> for Cow<'_, PartialAccount> {
        fn from(acc: PartialAccount) -> Self {
            Cow::Owned(acc)
        }
    }

    impl<'a> From<&'a PartialAccount> for Cow<'a, PartialAccount> {
        fn from(acc: &'a PartialAccount) -> Self {
            Cow::Borrowed(acc)
        }
    }
}

#[cfg(test)]
mod tests {
    use async_graphql::ID;
    use chrono::Utc;

    use super::*;

    #[test]
    fn current_account() {
        let acc = PartialAccount::new("test".to_owned().into(), "test".into());
        let expiry = Utc::now() + chrono::Duration::minutes(5);
        let current = CurrentAccount::new(acc.clone(), expiry);

        assert!(current.account().is_ok());
        assert_eq!(current.id().unwrap(), acc.id());
        assert_eq!(current.handle().unwrap(), acc.handle());
    }

    #[test]
    fn current_account_expired() {
        let acc = PartialAccount::new("test".to_owned().into(), "test".to_owned());
        let expiry = Utc::now();
        let current = CurrentAccount::new(acc, expiry);

        assert!(current.account().is_err());
        assert!(current.id().is_err());
        assert!(current.handle().is_err());
    }

    #[test]
    fn current_account_default() {
        let current = CurrentAccount::default();

        assert!(current.account().is_err());
        assert!(current.id().is_err());
        assert!(current.handle().is_err());
    }

    #[test]
    fn partial_account() {
        let id: ID = "test".to_owned().into();
        let hdl = "test".to_owned();
        let acc = PartialAccount::new(id.clone(), hdl.clone());

        assert_eq!(acc.id(), &id);
        assert_eq!(acc.handle(), hdl.as_str());
    }
}
