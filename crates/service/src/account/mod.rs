mod auth;
mod persist;
mod schema;

use std::borrow::Cow;

pub use auth::*;
pub use persist::*;
pub use schema::*;

use async_graphql::{ComplexObject, InputObject, SimpleObject, ID};
use chrono::{DateTime, Utc};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use crate::error::{Error, Result};

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

/// Account information stored in the JWT.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartialAccount {
    id: ID,
    hdl: String,
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

#[derive(Debug)]
pub struct CurrentAccount(Option<PartialAccount>);

impl CurrentAccount {
    pub fn account(&self) -> Result<&PartialAccount> {
        self.0.as_ref().ok_or(Error::Unauthenticated)
    }

    pub fn id(&self) -> Result<&ID> {
        self.account().map(|acc| &acc.id)
    }
}
