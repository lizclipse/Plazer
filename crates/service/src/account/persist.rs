use chrono::{DateTime, Utc};
use indoc::indoc;
use ring::rand::SystemRandom;
use secrecy::ExposeSecret as _;
use tracing::instrument;

use super::{
    create_creds, verify_creds, verify_refresh_token, Account, AuthCreds, AuthenticatedAccount,
    CreateAccount, CurrentAccount, TABLE_NAME,
};
use crate::{
    error::{Error, Result},
    persist::Persist,
};

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
        let acc = match self.get_by_user_id(&creds.user_id).await? {
            Some(acc) => acc,
            None => return Err(Error::CredentialsInvalid),
        };

        verify_creds(&creds.pword, &acc.pword_salt, &acc.pword_hash)?;

        Ok(acc.into())
    }

    #[instrument(skip_all)]
    pub async fn refresh(&self, refresh_token: String) -> Result<AuthenticatedAccount> {
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
                CREATE type::table($tbl) SET
                    user_id = $user_id,
                    pword_salt = $pword_salt,
                    pword_hash = $pword_hash
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
    use crate::{
        account::create_refresh_token,
        conv::{IntoGqlId as _, ToGqlId as _},
    };

    use super::{testing::*, *};

    #[tokio::test]
    async fn test_create() {
        let data = TestData::new().await;
        let account = data.account();

        let acc = CreateAccount {
            user_id: "test".into(),
            pword: "test".to_owned().into(),
            invite: None,
        };

        let res = account.create(acc).await;
        println!("{res:?}");
        assert!(res.is_ok());

        let res = res.unwrap();
        assert_eq!(res.account.user_id, "test");
    }

    #[tokio::test]
    async fn test_get() {
        let data = TestData::new().await;
        let account = data.account();
        let AccData { user_id, acc, .. } = account.create_test_user().await;

        let res = account.get(&acc.id.into_gql_id()).await;
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
        let account = data.account();
        let AccData { user_id, acc, .. } = account.create_test_user().await;

        let res = account.get_by_user_id(&user_id).await;
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
        let account = data.account();
        let AccData { user_id, .. } = account.create_test_user().await;

        let acc = CreateAccount {
            user_id,
            pword: "test2".to_owned().into(),
            invite: None,
        };

        let res = account.create(acc).await;
        println!("{res:?}");
        assert!(res.is_err());

        let res = res.unwrap_err();
        assert_eq!(res, Error::UnavailableIdent);
    }

    #[tokio::test]
    async fn test_login() {
        let data = TestData::new().await;
        let account = data.account();
        let AccData { user_id, pword, .. } = account.create_test_user().await;

        let res = account
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
        let account = data.account();
        let AccData { user_id, .. } = account.create_test_user().await;

        let res = account
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
        let account = data.account();
        let AccData { user_id, acc, .. } = account.create_test_user().await;
        let refresh_token = create_refresh_token(acc.id.to_gql_id(), &data.jwt_enc_key).unwrap();

        let res = account.refresh(refresh_token).await;
        println!("{res:?}");
        assert!(res.is_ok());

        let res = res.unwrap();
        assert_eq!(res.account.id, acc.id);
        assert_eq!(res.account.user_id, user_id);
    }

    #[tokio::test]
    async fn test_access_token_fail() {
        let data = TestData::new().await;
        let account = data.account();

        let res = account.refresh("invalid.refresh.token".into()).await;
        println!("{res:?}");
        assert!(res.is_err());

        let res = res.unwrap_err();
        assert_eq!(res, Error::CredentialsInvalid);
    }

    #[tokio::test]
    async fn test_revoke_tokens() {
        let (data, AccData { acc, .. }) = TestData::with_user().await;
        let account = data.account();
        let refresh_token = create_refresh_token(acc.id.into_gql_id(), &data.jwt_enc_key).unwrap();

        let res = account.revoke_tokens().await;
        println!("{res:?}");
        assert!(res.is_ok());

        let res = account.refresh(refresh_token).await;
        println!("{res:?}");
        assert!(res.is_err());

        let res = res.unwrap_err();
        assert_eq!(res, Error::Unauthenticated);
    }

    #[tokio::test]
    async fn test_revoke_tokens_fail() {
        let data = TestData::new().await;
        let account = data.account();

        let res = account.revoke_tokens().await;
        println!("{res:?}");
        assert!(res.is_err());

        let res = res.unwrap_err();
        assert_eq!(res, Error::Unauthenticated);
    }
}

#[cfg(test)]
mod testing {
    use base64::prelude::*;
    use chrono::Duration;
    use ring::rand::SecureRandom as _;
    use secrecy::SecretString;

    use crate::{
        account::{testing::generate_keys, PartialAccount},
        conv::ToGqlId as _,
        persist::testing::persist,
    };

    use super::*;

    pub struct TestData {
        pub persist: Persist,
        pub current: CurrentAccount,
        pub csrng: SystemRandom,
        pub jwt_enc_key: jsonwebtoken::EncodingKey,
        pub jwt_dec_key: jsonwebtoken::DecodingKey,
    }

    pub struct AccData {
        pub user_id: String,
        pub pword: SecretString,
        pub acc: Account,
    }

    impl TestData {
        pub async fn new() -> Self {
            let (jwt_enc_key, jwt_dec_key) = generate_keys();
            Self {
                persist: persist().await,
                current: Default::default(),
                csrng: SystemRandom::new(),
                jwt_enc_key,
                jwt_dec_key,
            }
        }

        pub async fn with_user() -> (Self, AccData) {
            let mut data = Self::new().await;
            let account = data.account();
            let acc = account.create_test_user().await;
            data.current = CurrentAccount::new(
                PartialAccount::new(acc.acc.id.to_gql_id(), acc.user_id.clone()),
                Utc::now() + Duration::minutes(30),
            );
            (data, acc)
        }

        pub fn account(&self) -> AccountPersist<'_> {
            AccountPersist::new(&self.persist, &self.current, &self.csrng, &self.jwt_dec_key)
        }
    }

    impl AccountPersist<'_> {
        pub async fn create_test_user(&self) -> AccData {
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

            AccData {
                user_id,
                pword: pword.into(),
                acc: self.create(acc).await.unwrap().account,
            }
        }
    }
}
