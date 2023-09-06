use std::{future::IntoFuture, sync::Arc};

use async_graphql::Context;
use cfg_if::cfg_if;
use ring::rand::SystemRandom;
use surrealdb::{
    engine,
    opt::{capabilities::Capabilities, Config as SrlConfig},
    Result as SrlResult, Surreal,
};
use tracing::{error, instrument};

use crate::{
    account::{AccountPersist, CurrentAccount},
    board::BoardPersist,
    prelude::*,
    DecodingKey,
};

fn config() -> SrlConfig {
    SrlConfig::new().capabilities(Capabilities::all())
}

cfg_if! {

if #[cfg(all(
    feature = "backend-mem",
    not(feature = "backend-file"),
    not(feature = "backend-tikv")
))] {
    pub type DbLayer = Surreal<engine::local::Db>;

    async fn connect(_: String) -> SrlResult<DbLayer> {
        Surreal::new::<engine::local::Mem>(config()).await
    }
} else if #[cfg(all(
    not(feature = "backend-mem"),
    feature = "backend-file",
    not(feature = "backend-tikv")
))] {
    pub type DbLayer = Surreal<engine::local::Db>;

    async fn connect(address: String) -> SrlResult<DbLayer> {
        Surreal::new::<engine::local::RocksDb>((address, config())).await
    }
} else if #[cfg(all(
    not(feature = "backend-mem"),
    not(feature = "backend-file"),
    feature = "backend-tikv"
))] {
    pub type DbLayer = Surreal<engine::local::Db>;

    async fn connect(address: String) -> SrlResult<DbLayer> {
        Surreal::new::<engine::local::TiKv>((address, config())).await
    }
} else {
    pub type DbLayer = Surreal<engine::any::Any>;

    async fn connect(address: String) -> SrlResult<DbLayer> {
        engine::any::connect((address, config())).await
    }
}

}

pub trait PersistExt {
    fn current_account(&self) -> &CurrentAccount;
    fn account_persist(&self) -> AccountPersist;
    fn board_persist(&self) -> BoardPersist;
}

pub struct Persist(DbLayer);

static LOCK_TABLE: &str = "locks";

impl Persist {
    pub async fn new(address: String) -> SrlResult<Self> {
        let db = connect(address).await?;
        // TODO: select ns & db from config
        db.use_ns("test").use_db("test").await?;
        Ok(Self(db))
    }

    pub fn db(&self) -> &DbLayer {
        &self.0
    }

    #[instrument(skip(self, f))]
    pub async fn execute_in_lock<F, Fut, O>(&self, id: &str, f: F) -> SrlResult<Option<O>>
    where
        F: FnOnce() -> Fut,
        Fut: IntoFuture<Output = O>,
    {
        match self
            .db()
            .query(srql::CreateStatement {
                what: srql::thing((LOCK_TABLE, id)),
                output: srql::Output::None.into(),
                ..Default::default()
            })
            .await
            .and_then(|mut r| r.take(0))
        {
            Ok::<Option<()>, _>(_) => {
                let res = f().await;

                match self
                    .db()
                    .query(srql::DeleteStatement {
                        what: srql::thing((LOCK_TABLE, id)),
                        output: srql::Output::None.into(),
                        ..Default::default()
                    })
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
    fn current_account(&self) -> &CurrentAccount {
        self.data_opt::<CurrentAccount>()
            .unwrap_or_else(|| self.data_unchecked::<Arc<CurrentAccount>>())
    }

    fn account_persist(&self) -> AccountPersist {
        AccountPersist::new(
            self.data_unchecked::<Persist>(),
            self.current_account(),
            self.data_unchecked::<SystemRandom>(),
            self.data_unchecked::<DecodingKey>(),
        )
    }

    fn board_persist(&self) -> BoardPersist {
        BoardPersist::new(self.data_unchecked::<Persist>(), self.current_account())
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
                    sleep(Duration::from_millis(30)).await;
                })
                .await
            },
            async {
                sleep(Duration::from_millis(5)).await;
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
    use crate::migration::Migrations;

    use super::*;

    pub async fn persist() -> Persist {
        let persist = Persist::new("memory".into()).await.unwrap();
        Migrations::run(&persist).await.unwrap();
        persist
    }
}
