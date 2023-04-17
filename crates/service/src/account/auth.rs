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
use secrecy::{ExposeSecret as _, SecretString};
use serde::{Deserialize, Serialize};

use super::{CurrentAccount, PartialAccount};
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

pub fn create_access_token(
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

pub fn authenticate(
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

    let mut validation = Validation::new(Algorithm::EdDSA);
    validation.validate_nbf = true;
    let token_data = jsonwebtoken::decode::<AccessClaims>(token, dec_key, &validation)?;

    match token_data.claims.jwt.kind {
        JwtKind::Access => Ok(token_data.claims.into()),
        _ => Err(Error::JwtInvalid),
    }
}

pub fn create_refresh_token(id: ID, enc_key: &jsonwebtoken::EncodingKey) -> Result<String> {
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(Algorithm::EdDSA),
        &RefreshClaims::new(id),
        enc_key,
    )?;

    Ok(token)
}

pub fn verify_refresh_token(token: &str, dec_key: &DecodingKey) -> Result<RefreshClaims> {
    let validation = Validation::new(Algorithm::EdDSA);
    let token_data = jsonwebtoken::decode::<RefreshClaims>(token, dec_key, &validation)?;

    match token_data.claims.jwt.kind {
        JwtKind::Refresh => Ok(token_data.claims),
        _ => Err(Error::JwtInvalid),
    }
}

static PBKDF2_ITERS: u32 = 100_000;

pub struct StoredPword {
    pub salt: SecretString,
    pub hash: SecretString,
}

pub fn create_creds(pword: &str) -> Result<StoredPword> {
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

    let pword_salt = BASE64_STANDARD_NO_PAD.encode(pword_salt);
    let pword_hash = BASE64_STANDARD_NO_PAD.encode(pword_hash);

    Ok(StoredPword {
        salt: pword_salt.into(),
        hash: pword_hash.into(),
    })
}

pub fn verify_creds(
    pword: &SecretString,
    pword_salt: &SecretString,
    pword_hash: &SecretString,
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

#[cfg(test)]
mod tests {
    use ring::signature::{self, KeyPair as _};
    use serde_json::json;

    use super::*;

    #[test]
    fn test_creds_valid() {
        let creds = create_creds("password").unwrap();

        let res = verify_creds(&"password".to_owned().into(), &creds.salt, &creds.hash);

        assert!(res.is_ok());
    }

    #[test]
    fn test_creds_invalid() {
        let creds = create_creds("password").unwrap();

        let res = verify_creds(&"password1".to_owned().into(), &creds.salt, &creds.hash);

        assert!(res.is_err());
    }

    fn generate_keys() -> (jsonwebtoken::EncodingKey, jsonwebtoken::DecodingKey) {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap();
        let enc_key = jsonwebtoken::EncodingKey::from_ed_der(pkcs8_bytes.as_ref());
        let dec_key = jsonwebtoken::DecodingKey::from_ed_der(key_pair.public_key().as_ref());

        (enc_key, dec_key)
    }

    #[test]
    fn test_access_token_valid() {
        let (enc_key, dec_key) = generate_keys();

        let acc = PartialAccount {
            id: "id".into(),
            hdl: "handle".into(),
        };
        let token = create_access_token(&acc, &enc_key).unwrap();

        for inp in [
            Into::<AuthenticateInput>::into(json!({ "token": token })),
            Some(TypedHeader(Authorization::bearer(&token).unwrap())).into(),
        ] {
            let auth = authenticate(inp, &dec_key);
            assert!(auth.is_ok());

            let auth = auth.unwrap();
            assert!(auth.0.is_some());

            let auth = auth.0.unwrap();
            assert_eq!(auth.id, acc.id);
            assert_eq!(auth.hdl, acc.hdl);
        }
    }

    #[test]
    fn test_access_token_invalid() {
        let (enc_key_a, dec_key_a) = generate_keys();
        let (enc_key_b, dec_key_b) = generate_keys();

        // Invalid token
        let auth = authenticate(json!({ "token": "not a token" }), &dec_key_a);
        assert!(auth.is_err());
        assert_eq!(auth.unwrap_err(), Error::JwtMalformed);

        // Invalid signature
        let acc = PartialAccount {
            id: "id".into(),
            hdl: "handle".into(),
        };

        let token = create_access_token(&acc, &enc_key_a).unwrap();
        let auth = authenticate(json!({ "token": token }), &dec_key_b);
        assert!(auth.is_err());
        assert_eq!(auth.unwrap_err(), Error::JwtInvalid);

        // Expired token
        let mut access_claims = AccessClaims::new(acc);
        println!("{:?}", access_claims);
        access_claims.jwt.iat -= 300;
        access_claims.jwt.nbf -= 300;
        access_claims.jwt.exp = (Utc::now().timestamp() as usize) - 100;
        println!("{:?}", access_claims);
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::EdDSA),
            &access_claims,
            &enc_key_b,
        )
        .unwrap();
        let auth = authenticate(json!({ "token": token }), &dec_key_b);
        assert!(auth.is_err());
        assert_eq!(auth.unwrap_err(), Error::JwtExpired);

        // Refresh token
        let token = create_refresh_token("id".into(), &enc_key_b).unwrap();
        let auth = authenticate(json!({ "token": token }), &dec_key_b);
        assert!(auth.is_err());
        assert_eq!(auth.unwrap_err(), Error::JwtInvalid);
    }

    #[test]
    fn test_refresh_token_valid() {
        let (enc_key, dec_key) = generate_keys();

        let token = create_refresh_token("id".into(), &enc_key).unwrap();

        let auth = verify_refresh_token(&token, &dec_key);
        assert!(auth.is_ok());

        let auth = auth.unwrap();
        assert_eq!(auth.id, "id");
    }

    #[test]
    fn test_refresh_token_invalid() {
        let (enc_key_a, dec_key_a) = generate_keys();
        let (enc_key_b, dec_key_b) = generate_keys();

        // Invalid token
        let auth = verify_refresh_token(&"not a token", &dec_key_a);
        assert!(auth.is_err());
        assert_eq!(auth.unwrap_err(), Error::JwtMalformed);

        // Invalid signature
        let token = create_refresh_token("id".into(), &enc_key_a).unwrap();
        let auth = verify_refresh_token(&token, &dec_key_b);
        assert!(auth.is_err());
        assert_eq!(auth.unwrap_err(), Error::JwtInvalid);

        // Expired token
        let mut refresh_claims = RefreshClaims::new("id".into());
        refresh_claims.jwt.iat -= 300;
        refresh_claims.jwt.nbf -= 300;
        refresh_claims.jwt.exp = (Utc::now().timestamp() as usize) - 100;
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::EdDSA),
            &refresh_claims,
            &enc_key_b,
        )
        .unwrap();
        let auth = verify_refresh_token(&token, &dec_key_b);
        assert!(auth.is_err());
        assert_eq!(auth.unwrap_err(), Error::JwtExpired);

        // Access token
        let acc = PartialAccount {
            id: "id".into(),
            hdl: "handle".into(),
        };
        let token = create_access_token(&acc, &enc_key_b).unwrap();
        let auth = verify_refresh_token(&token, &dec_key_b);
        assert!(auth.is_err());
        assert_eq!(auth.unwrap_err(), Error::JwtInvalid);
    }
}
