use derive_more::From;
use serde::{Deserialize, Serialize};

#[derive(Debug, From, Serialize, Deserialize)]
pub enum Method {
    Login(LoginReq),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginReq {
    uname: String,
    pword: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LoginRes {
    Success,
    Failed,
}
