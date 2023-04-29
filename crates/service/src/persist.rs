use std::sync::Arc;

use async_graphql::Context;
use cfg_if::cfg_if;
use surrealdb::{engine, opt::Strict, Surreal};

use crate::{
    account::{AccountPersist, CurrentAccount},
    DecodingKey, EncodingKey,
};

cfg_if! {
    if #[cfg(all(feature = "backend-mem", not(feature = "backend-file"), not(feature = "backend-tikv")))] {
        pub type DbLayer = Surreal<engine::local::Db>;

        async fn connect(_: String) -> surrealdb::Result<DbLayer> {
            Surreal::new::<engine::local::Mem>(Strict).await
        }
    } else if #[cfg(all(not(feature = "backend-mem"), feature = "backend-file", not(feature = "backend-tikv")))] {
        pub type DbLayer = Surreal<engine::local::Db>;

        async fn connect(address: String) -> surrealdb::Result<DbLayer> {
            Surreal::new::<engine::local::RocksDb>((address, Strict)).await
        }
    } else if #[cfg(all(not(feature = "backend-mem"), not(feature = "backend-file"), feature = "backend-tikv"))] {
        pub type DbLayer = Surreal<engine::local::Db>;

        async fn connect(address: String) -> surrealdb::Result<DbLayer> {
            Surreal::new::<engine::local::TiKv>((address, Strict)).await
        }
    } else {
        pub type DbLayer = Surreal<engine::any::Any>;

        async fn connect(address: String) -> surrealdb::Result<DbLayer> {
            engine::any::connect((address, Strict)).await
        }
    }
}

pub trait PersistExt {
    fn account_persist(&self) -> AccountPersist;
}

pub struct Persist(DbLayer);

impl Persist {
    pub async fn new(address: String) -> surrealdb::Result<Self> {
        let db = connect(address).await?;
        // TODO: select ns & db from config
        db.use_ns("test").use_db("test").await?;
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
            self.data_opt::<CurrentAccount>()
                .unwrap_or_else(|| self.data_unchecked::<Arc<CurrentAccount>>()),
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
