use chrono::{DateTime, Utc};
use tracing::instrument;

use super::{
    create_access_token, create_refresh_token, verify_creds, verify_refresh_token, Account,
    AuthCreds, CreateAccount, CurrentAccount, PartialAccount,
};
use crate::{
    error::{Error, Result},
    persist::Persist,
};

pub struct AccountPersist<'a> {
    persist: &'a Persist,
    current: &'a CurrentAccount,
    jwt_enc_key: &'a jsonwebtoken::EncodingKey,
    jwt_dec_key: &'a jsonwebtoken::DecodingKey,
}

static TABLE_NAME: &str = "account";

impl<'a> AccountPersist<'a> {
    pub fn new(
        persist: &'a Persist,
        current: &'a CurrentAccount,
        jwt_enc_key: &'a jsonwebtoken::EncodingKey,
        jwt_dec_key: &'a jsonwebtoken::DecodingKey,
    ) -> Self {
        Self {
            persist,
            current,
            jwt_enc_key,
            jwt_dec_key,
        }
    }

    #[instrument(skip_all)]
    pub async fn current(&self) -> Result<Option<Account>> {
        let id = self.current.id()?;
        self.get(id).await
    }

    #[instrument(skip_all)]
    pub async fn refresh_token(&self, creds: AuthCreds) -> Result<String> {
        let acc = match self.get_by_handle(&creds.handle).await? {
            Some(acc) => acc,
            None => return Err(Error::CredentialsInvalid),
        };

        verify_creds(&creds, &acc).await?;

        create_refresh_token(acc.id.id.into(), self.jwt_enc_key).await
    }

    #[instrument(skip_all)]
    pub async fn access_token(&self, refresh_token: String) -> Result<String> {
        let claims = match verify_refresh_token(&refresh_token, self.jwt_dec_key).await {
            Ok(claims) => claims,
            Err(_) => return Err(Error::CredentialsInvalid),
        };

        let acc = match self.get(claims.id()).await? {
            Some(acc) => acc,
            None => return Err(Error::CredentialsInvalid),
        };

        if let Some(revoked_at) = acc.revoked_at {
            if revoked_at >= claims.issued_at()? {
                return Err(Error::Unauthenticated);
            }
        }

        create_access_token(
            &PartialAccount {
                id: acc.id.id.into(),
                hdl: acc.handle,
            },
            self.jwt_enc_key,
        )
        .await
    }

    #[instrument(skip_all)]
    pub async fn get(&self, id: &str) -> Result<Option<Account>> {
        Ok(self.persist.db().select((TABLE_NAME, id)).await?)
    }

    #[instrument(skip_all)]
    pub async fn get_by_handle(&self, handle: &str) -> Result<Option<Account>> {
        Ok(self
            .persist
            .db()
            .query("SELECT * FROM type::table($tbl) WHERE handle = $handle")
            .bind(("tbl", TABLE_NAME))
            .bind(("handle", handle))
            .await
            .and_then(|mut res| res.take(0))?)
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
