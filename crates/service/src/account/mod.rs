mod auth;
mod migration;
mod persist;
mod schema;

pub use auth::*;
pub use migration::*;
pub use persist::*;
pub use schema::*;

use async_graphql::{
    ComplexObject, Context, InputObject, Result as GqlResult, ResultExt as _, SimpleObject, ID,
};
use chrono::{DateTime, Utc};
use secrecy::SecretString;
use serde::Deserialize;
use surrealdb::sql::Thing;
use tracing::instrument;

use crate::{conv::ToGqlId as _, EncodingKey};

static TABLE_NAME: &str = "account";

pub trait ToAccountThing {
    fn to_account_thing(&self) -> Thing;
}

impl ToAccountThing for ID {
    fn to_account_thing(&self) -> Thing {
        (TABLE_NAME.to_owned(), self.0.clone()).into()
    }
}

/// A registered account.
#[derive(SimpleObject, Debug, Deserialize)]
#[graphql(complex)]
pub struct Account {
    #[graphql(skip)]
    pub id: Thing,
    /// The account's unique user ID. This is used to create default names for
    /// resources and for logging in.
    ///
    /// It can be changed, but this will not change the name of any resources
    /// that were created with the old user ID.
    pub user_id: String,
    /// A timestamp indicating the last time the user revoked all of their
    /// tokens.
    ///
    /// This is used to invalidate all tokens that were issued before the
    /// revocation.
    pub revoked_at: Option<DateTime<Utc>>,

    #[graphql(skip)]
    pword_salt: SecretString,
    #[graphql(skip)]
    pword_hash: SecretString,
}

#[ComplexObject]
impl Account {
    /// The account's unique ID.
    ///
    /// This cannot change, and can be safely used to refer to the account permanently.
    async fn id(&self) -> ID {
        self.id.to_gql_id()
    }
}

/// An account that has been authenticated, along with tokens to access it.
#[derive(SimpleObject, Debug, Deserialize)]
#[graphql(complex)]
pub struct AuthenticatedAccount {
    /// The account that has been authenticated.
    pub account: Account,
}

#[ComplexObject]
impl AuthenticatedAccount {
    /// A refresh token accociated with the account.
    #[instrument(skip_all)]
    async fn refresh_token(&self, ctx: &Context<'_>) -> GqlResult<String> {
        create_refresh_token(
            self.account.id.to_gql_id(),
            ctx.data_unchecked::<EncodingKey>(),
        )
        .extend()
    }

    /// An access token accociated with the account.
    #[instrument(skip_all)]
    async fn access_token(&self, ctx: &Context<'_>) -> GqlResult<String> {
        create_access_token(
            &PartialAccount::new(self.account.id.to_gql_id(), self.account.user_id.clone()),
            ctx.data_unchecked::<EncodingKey>(),
        )
        .extend()
    }
}

impl From<Account> for AuthenticatedAccount {
    fn from(account: Account) -> Self {
        Self { account }
    }
}

/// The information needed to create a new account.
#[derive(InputObject, Debug)]
pub struct CreateAccount {
    /// The account's unique user ID. This is used to create default names for
    /// resources and for logging in.
    #[graphql(validator(min_length = 1, max_length = 128))]
    user_id: String,
    /// The account's password.
    #[graphql(validator(min_length = 8, max_length = 1024), secret)]
    pword: SecretString,
    /// An optional invite code.
    ///
    /// Whether this is required will depend on the server's configuration.
    #[graphql(validator(min_length = 1, max_length = 1024))]
    invite: Option<String>,
}

/// The information needed to authenticate an account.
#[derive(InputObject, Debug)]
pub struct AuthCreds {
    /// The user ID of the account to authenticate.
    #[graphql(validator(min_length = 1, max_length = 64))]
    user_id: String,
    /// The account's password.
    #[graphql(validator(min_length = 8, max_length = 1024), secret)]
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

    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    pub struct CurrentAccount(Inner);

    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    enum Inner {
        #[default]
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

        pub fn user_id(&self) -> Result<&str> {
            self.account().map(|acc| acc.uid.as_str())
        }
    }

    /// Account information stored in the JWT.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct PartialAccount {
        id: ID,
        uid: String,
    }

    impl PartialAccount {
        pub fn new(id: ID, uid: String) -> Self {
            Self { id, uid }
        }

        pub fn id(&self) -> &ID {
            &self.id
        }

        pub fn user_id(&self) -> &str {
            self.uid.as_str()
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

        println!("{acc:?}\n{expiry:?}\n{current:?}");

        assert!(current.account().is_ok());
        assert_eq!(current.id().unwrap(), acc.id());
        assert_eq!(current.user_id().unwrap(), acc.user_id());
    }

    #[test]
    fn current_account_expired() {
        let acc = PartialAccount::new("test".to_owned().into(), "test".to_owned());
        let expiry = Utc::now();
        let current = CurrentAccount::new(acc.clone(), expiry);

        println!("{acc:?}\n{expiry:?}\n{current:?}");

        assert!(current.account().is_err());
        assert!(current.id().is_err());
        assert!(current.user_id().is_err());
    }

    #[test]
    fn current_account_default() {
        let current = CurrentAccount::default();

        println!("{current:?}");

        assert!(current.account().is_err());
        assert!(current.id().is_err());
        assert!(current.user_id().is_err());
    }

    #[test]
    fn partial_account() {
        let id: ID = "test".to_owned().into();
        let uid = "test".to_owned();
        let acc = PartialAccount::new(id.clone(), uid.clone());

        println!("{id:?}\n{uid:?}\n{acc:?}");

        assert_eq!(acc.id(), &id);
        assert_eq!(acc.user_id(), uid.as_str());
    }
}
