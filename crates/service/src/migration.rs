use std::{fmt::Debug, time::Duration};

use nanorand::{Rng as _, WyRand};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use surrealdb::{method::Query, Connection};
use tokio::time::sleep;
use tracing::{debug, instrument, trace};

use crate::{account::AccountMigration, persist::Persist};

pub trait Migration: Sized + Default + Serialize + DeserializeOwned + Debug + Send + Sync {
    fn subsystem() -> &'static str;
    fn next(self) -> Option<Self>;
    fn build<'a, C>(&self, query: Query<'a, C>) -> Query<'a, C>
    where
        C: Connection;
}

pub struct Migrations<'a> {
    persist: &'a Persist,
}

static UPDATE_TABLE: &str = "updates";

#[derive(Debug, Serialize, Deserialize)]
struct Update<M> {
    current: M,
}

impl Migrations<'_> {
    #[instrument(skip_all)]
    pub async fn run(persist: &Persist) -> surrealdb::Result<()> {
        let migrations = Migrations { persist };

        migrations.iterate::<AccountMigration>().await?;

        Ok(())
    }

    #[instrument(skip_all, fields(subsystem = M::subsystem()))]
    async fn iterate<M: Migration>(&self) -> surrealdb::Result<()> {
        let mut prng = WyRand::new();
        while let Some(update) = self.next_update::<M>().await? {
            match self
                .persist
                .execute_in_lock(M::subsystem(), || async {
                    update
                        .build(self.persist.db().query("BEGIN"))
                        .query(
                            "
                            UPDATE type::thing($__update_tbl, $__update_subsys)
                            SET
                                current = $__update_done,
                                history += [{ update: $__update_done, timestamp: time::now() }]
                            ",
                        )
                        .bind(("__update_tbl", UPDATE_TABLE))
                        .bind(("__update_subsys", M::subsystem()))
                        .bind(("__update_done", &update))
                        .query("COMMIT")
                        .await
                })
                .await?
            {
                Some(res) => {
                    res?.check()?;
                }
                None => {
                    trace!("Migration locked, sleeping");
                    // Introduce a bit of jitter to avoid thundering herd.
                    sleep(Duration::from_millis(
                        5000 + prng.generate_range(0..=10_000),
                    ))
                    .await;
                }
            }
        }

        Ok(())
    }

    #[instrument(skip_all)]
    async fn next_update<M>(&self) -> surrealdb::Result<Option<M>>
    where
        M: Migration,
    {
        match self
            .persist
            .db()
            .select((UPDATE_TABLE, M::subsystem()))
            .await?
        {
            Some::<Update<M>>(Update { current }) => match current.next() {
                Some(next) => {
                    debug!(?next, "Next migration step");
                    Ok(Some(next))
                }
                None => {
                    debug!("Migration complete");
                    Ok(None)
                }
            },
            None => {
                let update = M::default();
                debug!(?update, "Beginning migration");
                Ok(Some(update))
            }
        }
    }
}
