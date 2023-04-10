use async_graphql::ErrorExtensions;
use axum::Json;
use hyper::StatusCode;
use jsonwebtoken::errors::{Error as JwtError, ErrorKind as JwtErrorKind};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unauthenticated")]
    Unauthenticated,
    #[error("Unauthorized")]
    Unauthorized,

    #[error("JWT is malformed")]
    JwtMalformed,
    #[error("JWT is expired")]
    JwtExpired,
    #[error("JWT is invalid")]
    JwtInvalid,

    #[error("GraphQL WebSocket init must be an object, null, or undefined")]
    WsInitNotObject,
    #[error("GraphQL WebSocket init `token` must be a string or undefined")]
    WsInitTokenNotString,

    #[error("The server is misconfigured")]
    ServerMisconfigured,
    #[error("An internal server error occurred")]
    InternalServerError,
}

impl ErrorExtensions for Error {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(format!("{}", self)).extend_with(|_, e| {
            e.set("code", format!("{:?}", self));
        })
    }
}

impl From<JwtError> for Error {
    fn from(err: JwtError) -> Self {
        match err.kind() {
            JwtErrorKind::InvalidToken
            | JwtErrorKind::InvalidAlgorithmName
            | JwtErrorKind::InvalidKeyFormat => Self::JwtMalformed,
            JwtErrorKind::InvalidEcdsaKey
            | JwtErrorKind::InvalidRsaKey(_)
            | JwtErrorKind::RsaFailedSigning => Self::ServerMisconfigured,
            JwtErrorKind::ExpiredSignature => Self::JwtExpired,
            JwtErrorKind::Crypto(_) => Self::InternalServerError,
            _ => Self::JwtInvalid,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorData {
    code: String,
    message: String,
}

pub type ErrorResponse = (StatusCode, Json<ErrorData>);

impl From<Error> for ErrorResponse {
    fn from(err: Error) -> Self {
        let code = match err {
            Error::Unauthenticated | Error::JwtExpired | Error::JwtInvalid => {
                StatusCode::UNAUTHORIZED
            }
            Error::Unauthorized => StatusCode::FORBIDDEN,
            Error::JwtMalformed | Error::WsInitNotObject | Error::WsInitTokenNotString => {
                StatusCode::BAD_REQUEST
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let data = ErrorData {
            code: format!("{:?}", err),
            message: format!("{}", err),
        };
        (code, Json(data))
    }
}
