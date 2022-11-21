use async_trait::async_trait;
use c11ity_client::{Account, Result};
use c11ity_common::api::account::{LoginReq, LoginRes};

#[derive(Debug)]
pub struct DbAccount;

impl DbAccount {
    pub fn new() -> Self {
        DbAccount
    }
}

#[async_trait]
impl Account for DbAccount {
    async fn login<'a>(&self, req: LoginReq<'a>) -> Result<LoginRes> {
        todo!()
    }
}
