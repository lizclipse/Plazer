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

    /// Request an access token for authenticating requests.
    #[instrument(skip_all)]
    async fn access_token(&self, ctx: &Context<'_>, refresh_token: String) -> Result<String> {
        ctx.account_persist()
            .access_token(refresh_token)
            .await
            .extend()
    }
}

#[derive(Default)]
pub struct AccountMutation;

#[Object]
impl AccountMutation {
    /// Request a new refresh token.
    #[instrument(skip_all)]
    async fn refresh_token(&self, ctx: &Context<'_>, creds: AuthCreds) -> Result<String> {
        ctx.account_persist().refresh_token(creds).await.extend()
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
