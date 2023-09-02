#[cfg(test)]
use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine as _};
#[cfg(test)]
use ring::rand::SecureRandom as _;

use chrono::{DateTime, Utc};
use indoc::indoc;
use ring::rand::SystemRandom;
use secrecy::ExposeSecret as _;
use tracing::instrument;

use super::{
    create_creds, verify_creds, verify_refresh_token, Account, AuthCreds, AuthenticatedAccount,
    CreateAccount, CurrentAccount, TABLE_NAME,
};
use crate::{persist::Persist, prelude::*};

pub struct AccountPersist<'a> {
    persist: &'a Persist,
    current: &'a CurrentAccount,
    csrng: &'a SystemRandom,
    jwt_dec_key: &'a jsonwebtoken::DecodingKey,
}

impl<'a> AccountPersist<'a> {
    pub fn new(
        persist: &'a Persist,
        current: &'a CurrentAccount,
        csrng: &'a SystemRandom,
        jwt_dec_key: &'a jsonwebtoken::DecodingKey,
    ) -> Self {
        Self {
            persist,
            current,
            csrng,
            jwt_dec_key,
        }
    }

    #[instrument(skip_all)]
    pub async fn current(&self) -> Result<Option<Account>> {
        let id = self.current.id()?;
        self.get(id).await
    }

    #[instrument(skip_all)]
    pub async fn login(&self, creds: AuthCreds) -> Result<AuthenticatedAccount> {
        let Some(acc) = self.get_by_user_id(&creds.user_id).await? else {
            return Err(Error::CredentialsInvalid);
        };

        verify_creds(&creds.pword, &acc.pword_salt, &acc.pword_hash)?;

        Ok(acc.into())
    }

    #[instrument(skip_all)]
    pub async fn refresh(&self, refresh_token: String) -> Result<AuthenticatedAccount> {
        let Ok(claims) = verify_refresh_token(&refresh_token, self.jwt_dec_key) else {
            return Err(Error::CredentialsInvalid);
        };

        let Some(acc) = self.get(claims.id()).await? else {
            return Err(Error::CredentialsInvalid);
        };

        if let Some(revoked_at) = acc.revoked_at {
            if revoked_at >= claims.issued_at()? {
                return Err(Error::Unauthenticated);
            }
        }

        Ok(acc.into())
    }

    #[instrument(skip_all)]
    pub async fn get(&self, id: &str) -> Result<Option<Account>> {
        Ok(self.persist.db().select((TABLE_NAME, id)).await?)
    }

    #[instrument(skip_all)]
    pub async fn get_by_user_id(&self, user_id: &str) -> Result<Option<Account>> {
        let acc = self
            .persist
            .db()
            .query("SELECT * FROM type::table($tbl) WHERE user_id = $user_id")
            .bind(("tbl", TABLE_NAME))
            .bind(("user_id", user_id))
            .await?
            .take(0)?;
        Ok(acc)
    }

    #[instrument(skip_all)]
    pub async fn create(&self, acc: CreateAccount) -> Result<AuthenticatedAccount> {
        let creds = create_creds(self.csrng, acc.pword.expose_secret())?;

        // TODO: support invites and reject if required/invalid
        let acc: Option<Account> = self
            .persist
            .db()
            .query(indoc! {"
                CREATE type::thing($tbl, rand::uuid::v7()) SET
                    user_id = $user_id,
                    pword_salt = $pword_salt,
                    pword_hash = $pword_hash,

                    updated_at = time::now()
            "})
            .bind(("tbl", TABLE_NAME))
            .bind(("user_id", acc.user_id))
            .bind(("pword_salt", creds.salt.expose_secret()))
            .bind(("pword_hash", creds.hash.expose_secret()))
            .await?
            .take(0)?;

        match acc {
            Some(acc) => Ok(acc.into()),
            None => Err(Error::UnavailableIdent),
        }
    }

    #[instrument(skip_all)]
    pub async fn revoke_tokens(&self) -> Result<DateTime<Utc>> {
        let acc = self.current.id()?;
        let now = Utc::now();

        self.persist
            .db()
            .query(indoc! {"
                UPDATE type::thing($tbl, $id) SET
                    revoked_at = $revoked_at
            "})
            .bind(("tbl", TABLE_NAME))
            .bind(("id", acc))
            .bind(("revoked_at", now))
            .await?;

        Ok(now)
    }
}

#[cfg(test)]
impl AccountPersist<'_> {
    pub async fn create_test_user(&self) -> super::testing::AccData {
        let mut user_id = [0u8; 16];
        self.csrng.fill(&mut user_id).unwrap();
        let user_id = BASE64_STANDARD_NO_PAD.encode(user_id);
        let mut pword = [0u8; 16];
        self.csrng.fill(&mut pword).unwrap();
        let pword = BASE64_STANDARD_NO_PAD.encode(pword);

        let acc = CreateAccount {
            user_id: user_id.clone(),
            pword: pword.clone().into(),
            invite: None,
        };

        let acc = self.create(acc).await.unwrap().account;
        super::testing::AccData {
            id: acc.id.clone(),
            user_id,
            pword: pword.into(),
            acc,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::account::{create_refresh_token, testing::*};

    use super::*;

    #[tokio::test]
    async fn test_create() {
        let data = TestData::new().await;
        let acc_persist = data.account();

        let acc = CreateAccount {
            user_id: "test".into(),
            pword: "test".to_owned().into(),
            invite: None,
        };

        let res = acc_persist.create(acc).await;
        println!("{res:?}");
        assert!(res.is_ok());

        let res = res.unwrap();
        assert_eq!(res.account.user_id, "test");
    }

    #[tokio::test]
    async fn test_get() {
        let data = TestData::new().await;
        let acc_persist = data.account();
        let AccData { user_id, acc, .. } = acc_persist.create_test_user().await;

        let res = acc_persist.get(&acc.id.into_gql_id()).await;
        println!("{res:?}");
        assert!(res.is_ok());

        let res = res.unwrap();
        assert!(res.is_some());

        let res = res.unwrap();
        assert_eq!(res.user_id, user_id);
    }

    #[tokio::test]
    async fn test_get_user_id() {
        let data = TestData::new().await;
        let acc_persist = data.account();
        let AccData { user_id, acc, .. } = acc_persist.create_test_user().await;

        let res = acc_persist.get_by_user_id(&user_id).await;
        println!("{res:?}");
        assert!(res.is_ok());

        let res = res.unwrap();
        assert!(res.is_some());

        let res = res.unwrap();
        assert_eq!(res.id, acc.id);
    }

    #[tokio::test]
    async fn test_duplicate_user_id() {
        let data = TestData::new().await;
        let acc_persist = data.account();
        let AccData { user_id, .. } = acc_persist.create_test_user().await;

        let acc = CreateAccount {
            user_id,
            pword: "test2".to_owned().into(),
            invite: None,
        };

        let res = acc_persist.create(acc).await;
        println!("{res:?}");
        assert!(res.is_err());

        let res = res.unwrap_err();
        assert_eq!(res, Error::UnavailableIdent);
    }

    #[tokio::test]
    async fn test_login() {
        let data = TestData::new().await;
        let acc_persist = data.account();
        let AccData { user_id, pword, .. } = acc_persist.create_test_user().await;

        let res = acc_persist
            .login(AuthCreds {
                user_id: user_id.clone(),
                pword,
            })
            .await;
        println!("{res:?}");
        assert!(res.is_ok());

        let res = res.unwrap();
        assert_eq!(res.account.user_id, user_id);
    }

    #[tokio::test]
    async fn test_login_fail() {
        let data = TestData::new().await;
        let acc_persist = data.account();
        let AccData { user_id, .. } = acc_persist.create_test_user().await;

        let res = acc_persist
            .login(AuthCreds {
                user_id,
                pword: "bad password".to_owned().into(),
            })
            .await;
        println!("{res:?}");
        assert!(res.is_err());

        let res = res.unwrap_err();
        assert_eq!(res, Error::CredentialsInvalid);
    }

    #[tokio::test]
    async fn test_refresh() {
        let data = TestData::new().await;
        let acc_persist = data.account();
        let AccData { user_id, acc, .. } = acc_persist.create_test_user().await;
        let refresh_token = create_refresh_token(acc.id.to_gql_id(), &data.jwt_enc_key).unwrap();

        let res = acc_persist.refresh(refresh_token).await;
        println!("{res:?}");
        assert!(res.is_ok());

        let res = res.unwrap();
        assert_eq!(res.account.id, acc.id);
        assert_eq!(res.account.user_id, user_id);
    }

    #[tokio::test]
    async fn test_access_token_fail() {
        let data = TestData::new().await;
        let acc_persist = data.account();

        let res = acc_persist.refresh("invalid.refresh.token".into()).await;
        println!("{res:?}");
        assert!(res.is_err());

        let res = res.unwrap_err();
        assert_eq!(res, Error::CredentialsInvalid);
    }

    #[tokio::test]
    async fn test_revoke_tokens() {
        let (data, AccData { acc, .. }) = TestData::with_user().await;
        let acc_persist = data.account();
        let refresh_token = create_refresh_token(acc.id.into_gql_id(), &data.jwt_enc_key).unwrap();

        let res = acc_persist.revoke_tokens().await;
        println!("{res:?}");
        assert!(res.is_ok());

        let res = acc_persist.refresh(refresh_token).await;
        println!("{res:?}");
        assert!(res.is_err());

        let res = res.unwrap_err();
        assert_eq!(res, Error::Unauthenticated);
    }

    #[tokio::test]
    async fn test_revoke_tokens_fail() {
        let data = TestData::new().await;
        let acc_persist = data.account();

        let res = acc_persist.revoke_tokens().await;
        println!("{res:?}");
        assert!(res.is_err());

        let res = res.unwrap_err();
        assert_eq!(res, Error::Unauthenticated);
    }
}
