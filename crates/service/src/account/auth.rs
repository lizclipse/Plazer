use axum::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use super::{CurrentAccount, PartialAccount};
use crate::error::{Error, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    #[serde(flatten)]
    acc: PartialAccount,

    // aud: String, // Optional. Audience
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    // iss: String, // Optional. Issuer
    nbf: usize, // Optional. Not Before (as UTC timestamp)
                // sub: String, // Optional. Subject (whom token refers to)
}

impl From<Claims> for CurrentAccount {
    fn from(claims: Claims) -> Self {
        Self(Some(claims.acc))
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
    let token_data = jsonwebtoken::decode::<Claims>(token, dec_key, &validation)?;

    Ok(token_data.claims.into())
}

pub async fn create_token(
    acc: PartialAccount,
    enc_key: &jsonwebtoken::EncodingKey,
) -> Result<String> {
    let claims = Claims {
        acc,
        exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
        nbf: Utc::now().timestamp() as usize,
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(Algorithm::EdDSA),
        &claims,
        enc_key,
    )?;

    Ok(token)
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
