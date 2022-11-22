#[cfg(feature = "wasm")]
pub mod wasm;

use std::fmt::Display;

// use account::Account;
use async_trait::async_trait;
use c11ity_common::api;
use futures::channel::{mpsc, oneshot};
use getrandom::getrandom;
#[cfg(feature = "wasm")]
use gloo_net::websocket;
use thiserror::Error;

pub fn rand_u64() -> core::result::Result<u64, getrandom::Error> {
    let mut buf = [0u8; 8];
    getrandom(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

pub trait Client {
    fn connected(&self) -> bool;

    type Account<'a>: Account
    where
        Self: 'a;
    fn account<'a>(&'a self) -> Self::Account<'a>;
}

#[async_trait]
pub trait Account {
    async fn login<'a>(&self, req: api::account::LoginReq<'a>) -> Result<api::account::LoginRes>;
}

pub type Result<T> = core::result::Result<T, ClientError>;

#[derive(Debug, Error)]
pub enum ClientError {
    Closed,
    SendError,
    InvalidRequest(bincode::Error),
    InvalidResponse(bincode::Error),
    ResponseMismatch,
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Closed => write!(f, "channel closed"),
            ClientError::SendError => write!(f, "failed to send message"),
            ClientError::InvalidRequest(_) => write!(f, "given an invalid request"),
            ClientError::InvalidResponse(_) => write!(f, "received an invalid response"),
            ClientError::ResponseMismatch => write!(f, "received an unknown response"),
        }
    }
}

#[cfg(feature = "wasm")]
impl From<websocket::WebSocketError> for ClientError {
    fn from(err: websocket::WebSocketError) -> Self {
        match err {
            websocket::WebSocketError::ConnectionClose(_) => ClientError::Closed,
            _ => ClientError::SendError,
        }
    }
}

impl From<getrandom::Error> for ClientError {
    fn from(_: getrandom::Error) -> Self {
        ClientError::SendError
    }
}

impl From<mpsc::SendError> for ClientError {
    fn from(_: mpsc::SendError) -> Self {
        ClientError::Closed
    }
}

impl From<oneshot::Canceled> for ClientError {
    fn from(_: oneshot::Canceled) -> Self {
        ClientError::Closed
    }
}
