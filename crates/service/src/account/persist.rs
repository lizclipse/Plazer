use chrono::{DateTime, Utc};
use secrecy::ExposeSecret as _;
use tracing::instrument;

use super::{
    create_access_token, create_creds, create_refresh_token, verify_creds, verify_refresh_token,
    Account, AuthCreds, CreateAccount, CurrentAccount, PartialAccount,
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
        let acc = self
            .persist
            .db()
            .query("SELECT * FROM type::table($tbl) WHERE handle = $handle")
            .bind(("tbl", TABLE_NAME))
            .bind(("handle", handle))
            .await?
            .take(0)?;
        Ok(acc)
    }

    #[instrument(skip_all)]
    pub async fn create(&self, acc: CreateAccount) -> Result<Account> {
        let (pword_salt, pword_hash) = create_creds(acc.pword.expose_secret()).await?;

        // TODO: Use unique constraint on handle instead of this when SurrealDB supports it
        // TODO: support invites and reject if required/invalid
        let acc = self
            .persist
            .db()
            .query(
                "
BEGIN TRANSACTION;
LET $id = (IF (SELECT _ FROM type::table($tbl) WHERE handle = $handle) THEN
    NONE
ELSE
    (CREATE type::table($tbl) SET handle = $handle, pword_salt = $pword_salt, pword_hash = $pword_hash)
END);
IF $id THEN
    (SELECT * FROM type::table($tbl) WHERE id = $id)
ELSE
    []
END;
COMMIT TRANSACTION;
            ",
            )
            .bind(("tbl", TABLE_NAME))
            .bind(("handle", acc.handle))
            .bind(("pword_salt", pword_salt))
            .bind(("pword_hash", pword_hash))
            .await?
            .take(1)?;

        match acc {
            Some(acc) => Ok(acc),
            None => Err(Error::HandleAlreadyExists),
        }
    }

    #[instrument(skip_all)]
    pub async fn revoke_tokens(&self) -> Result<DateTime<Utc>> {
        let acc = self.current.id()?;
        let now = Utc::now();

        self.persist
            .db()
            .query("UPDATE type::table($tbl) SET revoked_at = $revoked_at WHERE id = $id")
            .bind(("tbl", TABLE_NAME))
            .bind(("revoked_at", now))
            .bind(("id", acc))
            .await?;

        Ok(now)
    }
}
