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

        verify_creds(&creds.pword, &acc.pword_salt, &acc.pword_hash)?;

        create_refresh_token(acc.id.id.into(), self.jwt_enc_key)
    }

    #[instrument(skip_all)]
    pub async fn access_token(&self, refresh_token: String) -> Result<String> {
        let claims = match verify_refresh_token(&refresh_token, self.jwt_dec_key) {
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
            &PartialAccount::new(acc.id.id.into(), acc.handle),
            self.jwt_enc_key,
        )
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
        let creds = create_creds(acc.pword.expose_secret())?;

        // TODO: Use unique constraint on handle instead of this when SurrealDB supports it
        // TODO: support invites and reject if required/invalid
        let acc = self
            .persist
            .db()
            .query(
                "
IF (SELECT _ FROM type::table($tbl) WHERE handle = $handle) THEN
    NONE
ELSE
    (CREATE type::table($tbl) SET
        handle = $handle,
        pword_salt = $pword_salt,
        pword_hash = $pword_hash)
END
            ",
            )
            .bind(("tbl", TABLE_NAME))
            .bind(("handle", acc.handle))
            .bind(("pword_salt", creds.salt.expose_secret()))
            .bind(("pword_hash", creds.hash.expose_secret()))
            .await?
            .take(0)?;

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
            .query("UPDATE type::thing($tbl, $id) SET revoked_at = $revoked_at")
            .bind(("tbl", TABLE_NAME))
            .bind(("id", acc))
            .bind(("revoked_at", now))
            .await?;

        Ok(now)
    }
}

#[cfg(test)]
mod tests {
    use super::{testing::*, *};

    #[tokio::test]
    async fn test_create() {
        let data = TestData::new().await;
        let account = data.account();

        let acc = CreateAccount {
            handle: "test".into(),
            pword: "test".to_owned().into(),
            invite: None,
        };

        let res = account.create(acc).await;
        assert!(res.is_ok());

        let res = res.unwrap();
        assert_eq!(res.handle, "test");
    }

    #[tokio::test]
    async fn test_get() {
        let data = TestData::new().await;
        let account = data.account();
        let AccData { handle, acc, .. } = account.create_test_user().await;

        let res = account.get(&acc.id_str()).await;
        assert!(res.is_ok());

        let res = res.unwrap();
        assert!(res.is_some());

        let res = res.unwrap();
        assert_eq!(res.handle, handle);
    }

    #[tokio::test]
    async fn test_get_handle() {
        let data = TestData::new().await;
        let account = data.account();
        let AccData { handle, acc, .. } = account.create_test_user().await;

        let res = account.get_by_handle(&handle).await;
        assert!(res.is_ok());

        let res = res.unwrap();
        assert!(res.is_some());

        let res = res.unwrap();
        assert_eq!(res.id, acc.id);
    }

    #[tokio::test]
    async fn test_duplicate_handle() {
        let data = TestData::new().await;
        let account = data.account();
        let AccData { handle, .. } = account.create_test_user().await;

        let acc = CreateAccount {
            handle,
            pword: "test2".to_owned().into(),
            invite: None,
        };

        let res = account.create(acc).await;
        assert!(res.is_err());

        let res = res.unwrap_err();
        assert_eq!(res, Error::HandleAlreadyExists);
    }

    #[tokio::test]
    async fn test_refresh_token() {
        let data = TestData::new().await;
        let account = data.account();
        let AccData { handle, pword, .. } = account.create_test_user().await;

        let res = account.refresh_token(AuthCreds { handle, pword }).await;
        assert!(res.is_ok());

        let res = res.unwrap();
        assert!(!res.is_empty());
    }

    #[tokio::test]
    async fn test_refresh_token_fail() {
        let data = TestData::new().await;
        let account = data.account();
        let AccData { handle, .. } = account.create_test_user().await;

        let res = account
            .refresh_token(AuthCreds {
                handle,
                pword: "bad password".to_owned().into(),
            })
            .await;
        assert!(res.is_err());

        let res = res.unwrap_err();
        assert_eq!(res, Error::CredentialsInvalid);
    }

    #[tokio::test]
    async fn test_access_token() {
        let data = TestData::new().await;
        let account = data.account();
        let AccData { handle, pword, .. } = account.create_test_user().await;
        let refresh_token = account
            .refresh_token(AuthCreds { handle, pword })
            .await
            .unwrap();

        let res = account.access_token(refresh_token).await;
        assert!(res.is_ok());

        let res = res.unwrap();
        assert!(!res.is_empty());
    }

    #[tokio::test]
    async fn test_access_token_fail() {
        let data = TestData::new().await;
        let account = data.account();

        let res = account.access_token("invalid.refresh.token".into()).await;
        assert!(res.is_err());

        let res = res.unwrap_err();
        assert_eq!(res, Error::CredentialsInvalid);
    }

    #[tokio::test]
    async fn test_revoke_tokens() {
        let (data, AccData { handle, pword, .. }) = TestData::with_user().await;
        let account = data.account();
        let refresh_token = account
            .refresh_token(AuthCreds { handle, pword })
            .await
            .unwrap();

        let res = account.revoke_tokens().await;
        assert!(res.is_ok());

        let res = account.access_token(refresh_token).await;
        assert!(res.is_err());

        let res = res.unwrap_err();
        assert_eq!(res, Error::Unauthenticated);
    }

    #[tokio::test]
    async fn test_revoke_tokens_fail() {
        let data = TestData::new().await;
        let account = data.account();

        let res = account.revoke_tokens().await;
        assert!(res.is_err());

        let res = res.unwrap_err();
        assert_eq!(res, Error::Unauthenticated);
    }
}

#[cfg(test)]
mod testing {
    use base64::prelude::*;
    use chrono::Duration;
    use ring::rand::{self, SecureRandom as _};
    use secrecy::SecretString;
    use surrealdb::sql::Id;

    use crate::{account::testing::generate_keys, persist::testing::persist};

    use super::*;

    pub struct TestData {
        pub persist: Persist,
        pub current: CurrentAccount,
        pub jwt_enc_key: jsonwebtoken::EncodingKey,
        pub jwt_dec_key: jsonwebtoken::DecodingKey,
    }

    pub struct AccData {
        pub handle: String,
        pub pword: SecretString,
        pub acc: Account,
    }

    impl TestData {
        pub async fn new() -> Self {
            let (jwt_enc_key, jwt_dec_key) = generate_keys();
            Self {
                persist: persist().await,
                current: Default::default(),
                jwt_enc_key,
                jwt_dec_key,
            }
        }

        pub async fn with_user() -> (Self, AccData) {
            let mut data = Self::new().await;
            let account = data.account();
            let acc = account.create_test_user().await;
            data.current = CurrentAccount::new(
                PartialAccount::new(acc.acc.id_str().into(), acc.handle.clone()),
                Utc::now() + Duration::minutes(30),
            );
            (data, acc)
        }

        pub fn account(&self) -> AccountPersist<'_> {
            AccountPersist::new(
                &self.persist,
                &self.current,
                &self.jwt_enc_key,
                &self.jwt_dec_key,
            )
        }
    }

    impl AccountPersist<'_> {
        pub async fn create_test_user(&self) -> AccData {
            let rng = rand::SystemRandom::new();
            let mut handle = [0u8; 16];
            rng.fill(&mut handle).unwrap();
            let handle = BASE64_STANDARD_NO_PAD.encode(handle);
            let mut pword = [0u8; 16];
            rng.fill(&mut pword).unwrap();
            let pword = BASE64_STANDARD_NO_PAD.encode(pword);

            let acc = CreateAccount {
                handle: handle.clone(),
                pword: pword.clone().into(),
                invite: None,
            };

            AccData {
                handle,
                pword: pword.into(),
                acc: self.create(acc).await.unwrap(),
            }
        }
    }

    impl Account {
        pub fn id_str(&self) -> String {
            match &self.id.id {
                Id::String(id) => id.clone(),
                _ => panic!("unexpected id type"),
            }
        }
    }
}
