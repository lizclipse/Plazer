use chrono::{DateTime, Utc};
use tracing::instrument;

use super::{Account, AuthCreds, CreateAccount, CurrentAccount};
use crate::{
    error::{Error, Result},
    persist::Persist,
};

pub struct AccountPersist<'a> {
    persist: &'a Persist,
    current: &'a CurrentAccount,
}

static TABLE_NAME: &str = "account";

impl<'a> AccountPersist<'a> {
    pub fn new(persist: &'a Persist, current: &'a CurrentAccount) -> Self {
        Self { persist, current }
    }

    #[instrument(skip_all)]
    pub async fn current(&self) -> Result<Option<Account>> {
        let id = self.current.id()?;
        self.get(id).await
    }

    #[instrument(skip_all)]
    pub async fn auth_token(&self, _creds: Option<AuthCreds>) -> Result<String> {
        Err(Error::NotImplemented)
    }

    #[instrument(skip_all)]
    pub async fn get(&self, id: &str) -> Result<Option<Account>> {
        Ok(self.persist.db().select((TABLE_NAME, id)).await?)
    }

    #[instrument(skip_all)]
    pub async fn create(&self, _acc: CreateAccount) -> Result<Account> {
        Err(Error::NotImplemented)
    }

    #[instrument(skip_all)]
    pub async fn revoke_tokens(&self) -> Result<DateTime<Utc>> {
        Err(Error::NotImplemented)
    }
}
