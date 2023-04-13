use std::borrow::Cow;

use axum::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use ring::pbkdf2;
use secrecy::ExposeSecret as _;
use serde::{Deserialize, Serialize};

use super::{Account, AuthCreds, CurrentAccount, PartialAccount};
use crate::error::{Error, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    // aud: String, // Optional. Audience
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    // iss: String, // Optional. Issuer
    nbf: usize, // Optional. Not Before (as UTC timestamp)
                // sub: String, // Optional. Subject (whom token refers to)
}

impl JwtClaims {
    pub fn new(duration: Duration) -> Self {
        let now = Utc::now();
        Self {
            exp: (now + duration).timestamp() as usize,
            iat: now.timestamp() as usize,
            nbf: now.timestamp() as usize,
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
            jwt: JwtClaims::new(Duration::minutes(30)),
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

    Ok(token_data.claims.into())
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
        100_000.try_into().unwrap(),
        pword_salt.expose_secret().as_bytes(),
        pword.expose_secret().as_bytes(),
        pword_hash.expose_secret().as_bytes(),
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
