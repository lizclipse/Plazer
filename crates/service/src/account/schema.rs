use async_graphql::{Context, Object, Result, ResultExt as _};
use chrono::{DateTime, Utc};
use tracing::instrument;

use super::{Account, AuthCreds, CreateAccount};
use crate::persist::PersistExt as _;

#[derive(Default)]
pub struct AccountQuery;

#[Object]
impl AccountQuery {
    /// Get the account associated with the current session.
    ///
    /// If this returns `null`, then it means that the account associated with
    /// the current session has been deleted.
    #[instrument(skip_all)]
    async fn me(&self, ctx: &Context<'_>) -> Result<Option<Account>> {
        ctx.account_persist().current().await.extend()
    }
}

#[derive(Default)]
pub struct AccountMutation;

#[Object]
impl AccountMutation {
    /// Request an authentication token.
    ///
    /// This can be used to refresh an existing token when requested without
    /// credentials.
    #[instrument(skip_all)]
    async fn auth_token(&self, ctx: &Context<'_>, creds: Option<AuthCreds>) -> Result<String> {
        ctx.account_persist().access_token(creds).await.extend()
    }

    /// Register a new account.
    #[instrument(skip_all)]
    async fn create_account(&self, ctx: &Context<'_>, create: CreateAccount) -> Result<Account> {
        ctx.account_persist().create(create).await.extend()
    }

    /// Revoke all tokens issued for the current account.
    #[instrument(skip_all)]
    async fn revoke_tokens(&self, ctx: &Context<'_>) -> Result<DateTime<Utc>> {
        ctx.account_persist().revoke_tokens().await.extend()
    }
}
