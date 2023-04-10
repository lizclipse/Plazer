use async_graphql::{Context, InputObject, Object, Result, SimpleObject, Subscription, ID};
use chrono::{DateTime, Utc};
use futures::Stream;
use secrecy::SecretString;

#[derive(Default)]
pub struct AuthQuery;

#[Object]
impl AuthQuery {
    /// Get the account associated with the current session.
    async fn me<'ctx>(&self, _ctx: &Context<'ctx>) -> Account {
        todo!()
    }
}

#[derive(Default)]
pub struct AuthMutation;

#[Object]
impl AuthMutation {
    /// Log in to an account.
    async fn login<'ctx>(
        &self,
        _ctx: &Context<'ctx>,
        _handle: String,
        _pword: SecretString,
    ) -> Account {
        todo!()
    }

    /// Register a new account.
    async fn create_account<'ctx>(&self, _ctx: &Context<'ctx>, _create: CreateAccount) -> Account {
        todo!()
    }

    /// Revoke all tokens issued for the current account.
    async fn revoke_tokens<'ctx>(&self, _ctx: &Context<'ctx>) -> Result<DateTime<Utc>> {
        todo!()
    }
}

#[derive(Default)]
pub struct AuthSubscription;

#[Subscription]
impl AuthSubscription {
    /// Subscribe to changes to the current account.
    async fn me<'ctx>(&self, _ctx: &Context<'ctx>) -> impl Stream<Item = Account> {
        async_stream::stream! {
            yield todo!();
        }
    }

    /// Subscribe to token revocations.
    async fn token_revocations<'ctx>(
        &self,
        _ctx: &Context<'ctx>,
    ) -> impl Stream<Item = DateTime<Utc>> {
        async_stream::stream! {
            yield todo!();
        }
    }
}

/// A registered account.
#[derive(SimpleObject, Debug)]
pub struct Account {
    /// The account's unique ID.
    id: ID,
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
