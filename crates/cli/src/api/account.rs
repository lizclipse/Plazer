use async_trait::async_trait;
use c11ity_client::{Account, Result};
use c11ity_common::api::account::{LoginReq, LoginRes};

#[derive(Debug)]
pub struct DbAccount;

#[async_trait]
impl Account for DbAccount {
    type LoginRes = LoginRes;

    async fn login<'a>(&self, req: LoginReq<'a>) -> Result<Self::LoginRes> {
        todo!()
    }
}
