use async_graphql::{Context, Object, Result, ResultExt as _};
use chrono::{DateTime, Utc};
use secrecy::SecretString;

use super::{Account, CreateAccount};
use crate::persist::PersistExt as _;

#[derive(Default)]
pub struct AccountQuery;

#[Object]
impl AccountQuery {
    /// Get the account associated with the current session.
    async fn me(&self, ctx: &Context<'_>) -> Result<Account> {
        ctx.account_persist().current().await.extend()
    }
}

#[derive(Default)]
pub struct AccountMutation;

#[Object]
impl AccountMutation {
    /// Log in to an account.
    async fn login(&self, _ctx: &Context<'_>, _handle: String, _pword: SecretString) -> Account {
        todo!()
    }

    /// Register a new account.
    async fn create_account(&self, _ctx: &Context<'_>, _create: CreateAccount) -> Account {
        todo!()
    }

    /// Revoke all tokens issued for the current account.
    async fn revoke_tokens(&self, _ctx: &Context<'_>) -> Result<DateTime<Utc>> {
        todo!()
    }
}
