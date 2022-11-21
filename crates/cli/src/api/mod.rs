mod account;

use c11ity_client::Client;

use self::account::DbAccount;

#[derive(Debug)]
pub struct DbClient;

impl Client for DbClient {
    fn connected(&self) -> bool {
        todo!()
    }

    type Account<'a> = DbAccount;

    fn account<'a>(&'a self) -> Self::Account<'a> {
        todo!()
    }
}
