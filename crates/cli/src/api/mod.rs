mod account;

use c11ity_client::{Account, Client};
use c11ity_common::api;

use self::account::DbAccount;

#[derive(Debug)]
pub struct DbClient;

impl Client for DbClient {
    fn connected(&self) -> bool {
        todo!()
    }

    type Account<'a> = DbAccount;

    fn account<'a>(&'a self) -> Self::Account<'a> {
        DbAccount::new()
    }
}

impl DbClient {
    pub async fn dispatch(&self, nonce: u64, req: api::Method<'_>) -> bincode::Result<Vec<u8>> {
        // Method calls are safe to unwrap here because the ClientError result is purely for
        // client-side networking issues, which can't happen here.
        match req {
            api::Method::Account(req) => {
                let account = self.account();
                match req {
                    api::account::Method::Login(req) => bincode::serialize(&api::Message {
                        nonce,
                        payload: account.login(req).await.unwrap(),
                    }),
                }
            }
        }
    }
}
