mod account;

use c11ity_client::{Account, Client};
use c11ity_common::api;
use tracing::instrument;

use self::account::DbAccount;

#[derive(Clone, Debug)]
pub struct Db;

impl Db {
    pub fn new() -> Self {
        Self
    }

    pub fn client(&self) -> DbClient {
        DbClient::new()
    }
}

#[derive(Debug)]
pub struct DbClient;

impl DbClient {
    fn new() -> Self {
        Self
    }

    #[instrument]
    pub async fn dispatch(&self, req: api::Method<'_>) -> api::Response {
        // Method calls are safe to unwrap here because the ClientError result is purely for
        // client-side networking issues, which can't happen here.
        match req {
            api::Method::Account(req) => api::Response::Account({
                let account = self.account();
                match req {
                    api::account::Method::Login(req) => account.login(req).await.unwrap(),
                }
                .into()
            }),
        }
        .into()
    }
}

impl Client for DbClient {
    fn connected(&self) -> bool {
        true
    }

    type Account<'a> = DbAccount;

    fn account<'a>(&'a self) -> Self::Account<'a> {
        DbAccount::new()
    }
}
