use derive_more::From;
use serde::{Deserialize, Serialize};

#[derive(Debug, From, Serialize, Deserialize)]
pub enum Method {
    Login(LoginReq),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginReq {
    pub uname: String,
    pub pword: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LoginRes {
    Success,
    Failed,
}
