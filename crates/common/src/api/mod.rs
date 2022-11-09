pub mod account;

use derive_more::From;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<'a> {
    pub nonce: u64,
    pub payload: &'a [u8],
}

#[derive(Debug, From, Serialize, Deserialize)]
pub enum Method<'a> {
    #[serde(borrow)]
    Account(account::Method<'a>),
}
