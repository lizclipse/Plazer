pub mod account;

use derive_more::From;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Message<I> {
    nonce: u64,
    payload: I,
}

#[derive(Debug, From, Serialize, Deserialize)]
pub enum Method {
    Account(account::Method),
}
