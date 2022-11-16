use derive_more::From;
use serde::{Deserialize, Serialize};

#[derive(Debug, From, Serialize, Deserialize)]
pub enum Method<'a> {
    #[serde(borrow)]
    Login(LoginReq<'a>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account<'a> {
    pub id: &'a str,
    pub name: Option<&'a str>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginReq<'a> {
    pub uname: &'a str,
    pub pword: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LoginRes<'a> {
    #[serde(borrow)]
    Success(Account<'a>),
    Failed,
}
