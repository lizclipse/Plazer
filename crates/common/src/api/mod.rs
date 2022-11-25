pub mod account;

use derive_more::From;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<T> {
    pub nonce: u64,
    pub payload: T,
}

#[derive(Debug, From, Serialize, Deserialize)]
pub enum Method<'a> {
    #[serde(borrow)]
    Account(account::Method<'a>),
}

#[derive(Debug, From, Serialize, Deserialize)]
pub enum Response {
    Account(account::Response),
}
