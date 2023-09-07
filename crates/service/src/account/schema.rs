use async_graphql::{Context, Object};
use chrono::{DateTime, Utc};
use tracing::instrument;

use super::{Account, AuthCreds, AuthenticatedAccount, CreateAccount};
use crate::prelude::*;

#[derive(Default)]
pub struct AccountQuery;

#[Object]
impl AccountQuery {
    /// Get the account associated with the current session.
    ///
    /// If this returns `null`, then it means that the account associated with
    /// the current session has been deleted.
    #[instrument(skip_all)]
    async fn me(&self, ctx: &Context<'_>) -> GqlResult<Option<Account>> {
        ctx.account_persist().current().await.extend()
    }
}

#[derive(Default)]
pub struct AccountMutation;

#[Object]
impl AccountMutation {
    /// Log into the target account.
    #[instrument(skip_all)]
    async fn login(&self, ctx: &Context<'_>, creds: AuthCreds) -> GqlResult<AuthenticatedAccount> {
        ctx.account_persist().login(creds).await.extend()
    }

    /// Refresh tokens and account data.
    #[instrument(skip_all)]
    async fn refresh(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(min_length = 1, max_length = 1024))] refresh_token: String,
    ) -> GqlResult<AuthenticatedAccount> {
        ctx.account_persist().refresh(refresh_token).await.extend()
    }

    /// Register a new account.
    #[instrument(skip_all)]
    async fn create_account(
        &self,
        ctx: &Context<'_>,
        create: CreateAccount,
    ) -> GqlResult<AuthenticatedAccount> {
        ctx.account_persist().create(create).await.extend()
    }

    /// Revoke all tokens issued for the current account.
    #[instrument(skip_all)]
    async fn revoke_tokens(&self, ctx: &Context<'_>) -> GqlResult<DateTime<Utc>> {
        ctx.account_persist().revoke_tokens().await.extend()
    }
}
