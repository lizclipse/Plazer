use async_graphql::Context;
use surrealdb::{
    engine::any::{connect, Any},
    Surreal,
};

use crate::{
    account::{AccountPersist, CurrentAccount},
    DecodingKey, EncodingKey,
};

// TODO: use features to select specific engines when building as a service
pub type DbLayer = Surreal<Any>;

pub trait PersistExt {
    fn account_persist(&self) -> AccountPersist;
}

pub struct Persist(DbLayer);

impl Persist {
    pub async fn new(address: String) -> surrealdb::Result<Self> {
        let db = connect(address).await?;
        Ok(Self(db))
    }

    pub fn db(&self) -> &DbLayer {
        &self.0
    }
}

impl PersistExt for Context<'_> {
    fn account_persist(&self) -> AccountPersist {
        AccountPersist::new(
            self.data_unchecked::<Persist>(),
            self.data_unchecked::<CurrentAccount>(),
            self.data_unchecked::<EncodingKey>(),
            self.data_unchecked::<DecodingKey>(),
        )
    }
}

#[cfg(test)]
pub mod testing {
    use super::*;

    pub async fn persist() -> Persist {
        let p = Persist::new("memory".into()).await.unwrap();
        p.db().use_ns("test").use_db("test").await.unwrap();
        p
    }
}
