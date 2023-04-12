use async_graphql::Context;
use surrealdb::{engine::any::Any, Surreal};

use crate::account::{AccountPersist, CurrentAccount};

// TODO: use features to select specific engines when building as a service
pub type DbLayer = Surreal<Any>;

static DB: DbLayer = Surreal::init();

pub trait PersistExt {
    fn account_persist(&self) -> AccountPersist;
}

pub struct Persist;

impl Persist {
    pub async fn new(address: String) -> surrealdb::Result<Self> {
        DB.connect(address).await?;
        Ok(Self)
    }

    pub fn db(&self) -> &DbLayer {
        &DB
    }
}

impl PersistExt for Context<'_> {
    fn account_persist(&self) -> AccountPersist {
        AccountPersist::new(
            self.data_unchecked::<Persist>(),
            self.data_unchecked::<CurrentAccount>(),
        )
    }
}
