use std::{future::IntoFuture, sync::Arc};

use async_graphql::Context;
use cfg_if::cfg_if;
use ring::rand::SystemRandom;
use surrealdb::{engine, Surreal};
use tracing::{error, instrument};

use crate::{
    account::{AccountPersist, CurrentAccount},
    DecodingKey, EncodingKey,
};

cfg_if! {
    if #[cfg(all(feature = "backend-mem", not(feature = "backend-file"), not(feature = "backend-tikv")))] {
        pub type DbLayer = Surreal<engine::local::Db>;

        async fn connect(_: String) -> surrealdb::Result<DbLayer> {
            Surreal::new::<engine::local::Mem>(()).await
        }
    } else if #[cfg(all(not(feature = "backend-mem"), feature = "backend-file", not(feature = "backend-tikv")))] {
        pub type DbLayer = Surreal<engine::local::Db>;

        async fn connect(address: String) -> surrealdb::Result<DbLayer> {
            Surreal::new::<engine::local::RocksDb>(address).await
        }
    } else if #[cfg(all(not(feature = "backend-mem"), not(feature = "backend-file"), feature = "backend-tikv"))] {
        pub type DbLayer = Surreal<engine::local::Db>;

        async fn connect(address: String) -> surrealdb::Result<DbLayer> {
            Surreal::new::<engine::local::TiKv>(address).await
        }
    } else {
        pub type DbLayer = Surreal<engine::any::Any>;

        async fn connect(address: String) -> surrealdb::Result<DbLayer> {
            engine::any::connect(address).await
        }
    }
}

pub trait PersistExt {
    fn account_persist(&self) -> AccountPersist;
}

pub struct Persist(DbLayer);

static LOCK_TABLE: &str = "locks";

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

    #[instrument(skip(self, f))]
    pub async fn execute_in_lock<F, Fut, O>(&self, id: &str, f: F) -> surrealdb::Result<Option<O>>
    where
        F: FnOnce() -> Fut,
        Fut: IntoFuture<Output = O>,
    {
        match self
            .db()
            .query("CREATE type::thing($tbl, $id) RETURN NONE")
            .bind(("tbl", LOCK_TABLE))
            .bind(("id", id))
            .await
            .and_then(|mut r| r.take(0))
        {
            Ok::<Option<()>, _>(_) => {
                let res = f().await;

                match self
                    .db()
                    .query("DELETE type::thing($tbl, $id) RETURN NONE")
                    .bind(("tbl", LOCK_TABLE))
                    .bind(("id", id))
                    .await
                    .and_then(|mut r| r.take(0))
                {
                    Ok::<Option<()>, _>(_) => Ok(Some(res)),
                    Err::<_, surrealdb::Error>(err) => {
                        error!(
                            error = ?err,
                            "Failed to clear lock, database might be corrupt"
                        );
                        Err(err)
                    }
                }
            }
            Err::<_, surrealdb::Error>(err) => match err {
                surrealdb::Error::Db(surrealdb::error::Db::RecordExists { .. }) => Ok(None),
                _ => Err(err),
            },
        }
    }
}

impl PersistExt for Context<'_> {
    fn account_persist(&self) -> AccountPersist {
        AccountPersist::new(
            self.data_unchecked::<Persist>(),
            self.data_opt::<CurrentAccount>()
                .unwrap_or_else(|| self.data_unchecked::<Arc<CurrentAccount>>()),
            self.data_unchecked::<SystemRandom>(),
            self.data_unchecked::<EncodingKey>(),
            self.data_unchecked::<DecodingKey>(),
        )
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use futures::join;
    use tokio::time::sleep;

    use super::testing::*;

    #[tokio::test]
    async fn test_lock() {
        let p = persist().await;
        let id = "test_lock";

        let (a, b) = join!(
            async {
                p.execute_in_lock(id, || async {
                    sleep(Duration::from_millis(5)).await;
                })
                .await
            },
            async {
                sleep(Duration::from_millis(2)).await;
                p.execute_in_lock(id, || async {}).await
            },
        );

        println!("{a:?}\n{b:?}");

        assert!(a.is_ok());
        assert!(b.is_ok());

        assert!(a.unwrap().is_some());
        assert!(b.unwrap().is_none());
    }
}

#[cfg(test)]
pub mod testing {
    use super::*;

    pub async fn persist() -> Persist {
        Persist::new("memory".into()).await.unwrap()
    }
}
