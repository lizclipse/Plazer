use async_graphql::{ComplexObject, Context, Object, Result, ResultExt as _, SimpleObject};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::instrument;

use super::{
    create_access_token, create_refresh_token, Account, AuthCreds, CreateAccount, PartialAccount,
};
use crate::{persist::PersistExt as _, EncodingKey};

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
#[derive(SimpleObject, Debug, Deserialize)]
#[graphql(complex)]
struct CreateAccountResult {
    account: Account,
}

#[ComplexObject]
impl CreateAccountResult {
    /// Request a refresh token for the newly created account.
    #[instrument(skip_all)]
    async fn refresh_token(&self, ctx: &Context<'_>) -> Result<String> {
        create_refresh_token(
            self.account.id.id.clone().into(),
            ctx.data_unchecked::<EncodingKey>(),
        )
        .extend()
    }

    /// Request an access token for the newly created account.
    #[instrument(skip_all)]
    async fn access_token(&self, ctx: &Context<'_>) -> Result<String> {
        create_access_token(
            &PartialAccount::new(
                self.account.id.id.clone().into(),
                self.account.handle.clone(),
            ),
            ctx.data_unchecked::<EncodingKey>(),
        )
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
    async fn create_account(
        &self,
        ctx: &Context<'_>,
        create: CreateAccount,
    ) -> Result<CreateAccountResult> {
        Ok(CreateAccountResult {
            account: ctx.account_persist().create(create).await.extend()?,
        })
    }

    /// Revoke all tokens issued for the current account.
    #[instrument(skip_all)]
    async fn revoke_tokens(&self, ctx: &Context<'_>) -> Result<DateTime<Utc>> {
        ctx.account_persist().revoke_tokens().await.extend()
    }
}
