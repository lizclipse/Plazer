use async_graphql::ID;

use super::{Account, CurrentAccount};
use crate::{
    error::{Result},
    persist::Persist,
};

pub struct AccountPersist<'a> {
    persist: &'a Persist,
    current: &'a CurrentAccount,
}

impl<'a> AccountPersist<'a> {
    pub fn new(persist: &'a Persist, current: &'a CurrentAccount) -> Self {
        Self { persist, current }
    }

    pub async fn current(&self) -> Result<Account> {
        let id = self.current.id()?;
        self.get(id).await
    }

    pub async fn get(&self, _id: &ID) -> Result<Account> {
        todo!()
    }
}
