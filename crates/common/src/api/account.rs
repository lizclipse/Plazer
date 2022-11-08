use derive_more::From;
use serde::{Deserialize, Serialize};

#[derive(Debug, From, Serialize, Deserialize)]
pub enum Method<'a> {
    #[serde(borrow)]
    Login(LoginReq<'a>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginReq<'a> {
    uname: &'a str,
    pword: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LoginRes {
    Success,
    Failed,
}
