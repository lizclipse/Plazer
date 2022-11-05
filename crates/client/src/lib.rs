mod account;

use std::fmt::Display;

use account::Account;
use c11ity_common::api::Method;
use gloo_net::websocket::futures::WebSocket;
use serde::{Deserialize, Serialize};
use thiserror::Error;

type Result<T> = core::result::Result<T, ClientError>;

pub struct Client {
    inner: ClientInner,
}

impl<'a> Client {
    pub fn new(chl: WebSocket) -> Self {
        Self {
            inner: ClientInner { chl },
        }
    }

    pub fn account(&'a mut self) -> Account<'a> {
        Account::new(&mut self.inner)
    }
}

struct ClientInner {
    chl: WebSocket,
}

impl ClientInner {
    async fn call<'a, Res>(&mut self, inp: Method) -> Result<Res>
    where
        Res: Deserialize<'a>,
    {
        self.call_raw(inp).await
    }

    async fn call_raw<'a, Req, Res>(&mut self, inp: Req) -> Result<Res>
    where
        Req: Serialize,
        Res: Deserialize<'a>,
    {
        todo!()
    }
}

#[derive(Debug, Error)]
pub enum ClientError {
    Closed,
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Closed => write!(f, "channel closed"),
        }
    }
}
