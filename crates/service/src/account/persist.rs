#[cfg(test)]
mod tests;

#[cfg(test)]
use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine as _};
use chrono::{DateTime, Utc};
#[cfg(test)]
use ring::rand::SecureRandom as _;
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
            .query(srql::SelectStatement {
                expr: srql::Fields::all(),
                what: srql::table(TABLE_NAME),
                cond: srql::Cond(
                    srql::Expression::Binary {
                        l: srql::field("user_id").into(),
                        o: srql::Operator::Equal,
                        r: srql::string(user_id).into(),
                    }
                    .into(),
                )
                .into(),
                ..Default::default()
            })
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
            .query(Account::create(creds, acc))
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

        let mut updates = vec![];
        now.push_field(srql::field("revoked_at"), &mut updates);
        let Some(update) = srql::update_obj_query((TABLE_NAME, &***acc).into(), updates) else {
            return Err("".into());
        };

        self.persist.db().query(update).await?;

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
