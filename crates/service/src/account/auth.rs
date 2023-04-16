use std::borrow::Cow;

use async_graphql::ID;
use axum::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use base64::prelude::*;
use chrono::{DateTime, Duration, LocalResult, TimeZone, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use ring::{
    digest, pbkdf2,
    rand::{self, SecureRandom as _},
};
use secrecy::ExposeSecret as _;
use serde::{Deserialize, Serialize};

use super::{Account, AuthCreds, CurrentAccount, PartialAccount};
use crate::error::{Error, Result};

#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    // aud: String, // Optional. Audience
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    // iss: String, // Optional. Issuer
    nbf: usize, // Optional. Not Before (as UTC timestamp)
    // sub: String, // Optional. Subject (whom token refers to)
    kind: JwtKind,
}

#[derive(Debug, Serialize, Deserialize)]
enum JwtKind {
    Access,
    Refresh,
}

impl JwtClaims {
    fn new(duration: Duration, kind: JwtKind) -> Self {
        let now = Utc::now();
        Self {
            exp: (now + duration).timestamp() as usize,
            iat: now.timestamp() as usize,
            nbf: now.timestamp() as usize,
            kind,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    id: ID,
    #[serde(flatten)]
    jwt: JwtClaims,
}

impl RefreshClaims {
    pub fn new(id: ID) -> Self {
        Self {
            id,
            jwt: JwtClaims::new(Duration::days(30), JwtKind::Refresh),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn issued_at(&self) -> Result<DateTime<Utc>> {
        match Utc.timestamp_opt(self.jwt.iat as i64, 0) {
            LocalResult::Single(dt) => Ok(dt),
            LocalResult::None | LocalResult::Ambiguous(..) => Err(Error::JwtInvalid),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims<'a> {
    #[serde(flatten)]
    acc: Cow<'a, PartialAccount>,
    #[serde(flatten)]
    jwt: JwtClaims,
}

impl<'a> AccessClaims<'a> {
    pub fn new(acc: impl Into<Cow<'a, PartialAccount>>) -> Self {
        Self {
            acc: acc.into(),
            jwt: JwtClaims::new(Duration::minutes(30), JwtKind::Access),
        }
    }
}

impl From<AccessClaims<'_>> for CurrentAccount {
    fn from(claims: AccessClaims) -> Self {
        Self(Some(claims.acc.into_owned()))
    }
}

pub async fn authenticate(
    input: impl Into<AuthenticateInput>,
    dec_key: &DecodingKey,
) -> Result<CurrentAccount> {
    let input = input.into();
    let token = match &input {
        AuthenticateInput::Header(header) => header.as_ref().map(|h| h.0.token()),
        AuthenticateInput::Init(init) => {
            let token = match init.as_object() {
                Some(obj) => obj.get("token"),
                None => {
                    if !init.is_null() {
                        return Err(Error::WsInitNotObject);
                    } else {
                        None
                    }
                }
            };

            match token.map(|t| t.as_str()) {
                Some(Some(token)) => Some(token),
                Some(None) => return Err(Error::WsInitTokenNotString),
                None => None,
            }
        }
    };

    let token = match token {
        Some(token) => token,
        None => return Ok(CurrentAccount(None)),
    };

    let validation = Validation::new(Algorithm::EdDSA);
    let token_data = jsonwebtoken::decode::<AccessClaims>(token, dec_key, &validation)?;

    match token_data.claims.jwt.kind {
        JwtKind::Access => Ok(token_data.claims.into()),
        _ => Err(Error::JwtInvalid),
    }
}

pub async fn verify_refresh_token(token: &str, dec_key: &DecodingKey) -> Result<RefreshClaims> {
    let validation = Validation::new(Algorithm::EdDSA);
    let token_data = jsonwebtoken::decode::<RefreshClaims>(token, dec_key, &validation)?;

    match token_data.claims.jwt.kind {
        JwtKind::Refresh => Ok(token_data.claims),
        _ => Err(Error::JwtInvalid),
    }
}

pub async fn create_refresh_token(id: ID, enc_key: &jsonwebtoken::EncodingKey) -> Result<String> {
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(Algorithm::EdDSA),
        &RefreshClaims::new(id),
        enc_key,
    )?;

    Ok(token)
}

pub async fn create_access_token(
    acc: &PartialAccount,
    enc_key: &jsonwebtoken::EncodingKey,
) -> Result<String> {
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(Algorithm::EdDSA),
        &AccessClaims::new(acc),
        enc_key,
    )?;

    Ok(token)
}

static PBKDF2_ITERS: u32 = 100_000;

pub async fn create_creds(pword: &str) -> Result<(String, String)> {
    const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
    let rng = rand::SystemRandom::new();

    let mut pword_salt = [0u8; CREDENTIAL_LEN];
    rng.fill(&mut pword_salt)?;

    let mut pword_hash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA512,
        PBKDF2_ITERS.try_into().unwrap(),
        &pword_salt,
        pword.as_bytes(),
        &mut pword_hash,
    );

    let pword_salt = BASE64_STANDARD_NO_PAD.encode(&pword_salt);
    let pword_hash = BASE64_STANDARD_NO_PAD.encode(&pword_hash);

    Ok((pword_salt, pword_hash))
}

pub async fn verify_creds(
    AuthCreds { pword, .. }: &AuthCreds,
    Account {
        pword_salt,
        pword_hash,
        ..
    }: &Account,
) -> Result<()> {
    pbkdf2::verify(
        pbkdf2::PBKDF2_HMAC_SHA512,
        PBKDF2_ITERS.try_into().unwrap(),
        &BASE64_STANDARD_NO_PAD.decode(pword_salt.expose_secret())?,
        pword.expose_secret().as_bytes(),
        &BASE64_STANDARD_NO_PAD.decode(pword_hash.expose_secret())?,
    )?;

    Ok(())
}

pub enum AuthenticateInput {
    Header(Option<TypedHeader<Authorization<Bearer>>>),
    Init(serde_json::Value),
}

impl From<Option<TypedHeader<Authorization<Bearer>>>> for AuthenticateInput {
    fn from(header: Option<TypedHeader<Authorization<Bearer>>>) -> AuthenticateInput {
        AuthenticateInput::Header(header)
    }
}

impl From<serde_json::Value> for AuthenticateInput {
    fn from(init: serde_json::Value) -> AuthenticateInput {
        AuthenticateInput::Init(init)
    }
}
