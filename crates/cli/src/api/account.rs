use async_trait::async_trait;
use c11ity_client::{Account as AccountClient, Result};
use c11ity_common::api::account::{Account, LoginReq, LoginRes};

#[derive(Debug)]
pub struct DbAccount;

impl DbAccount {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AccountClient for DbAccount {
    async fn login<'a>(&self, req: LoginReq<'a>) -> Result<LoginRes> {
        tracing::debug!("{:?}", req);
        Ok(Ok(Account {
            id: "test".to_owned(),
            name: "Test".to_owned().into(),
        }))
    }
}
