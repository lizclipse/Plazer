use std::{fmt::Debug, time::Duration};

use nanorand::{Rng as _, WyRand};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::time::sleep;
use tracing::{debug, instrument, trace};

use crate::{account::AccountMigration, board::BoardMigration, persist::Persist, prelude::*};

pub trait Migration: Sized + Default + Serialize + DeserializeOwned + Debug + Send + Sync {
    const SUBSYSTEM: &'static str;

    fn next(self) -> Option<Self>;
    fn build(&self, statements: &mut Vec<srql::Statement>);
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

        debug!("Running migrations");
        migrations.iterate::<AccountMigration>().await?;
        migrations.iterate::<BoardMigration>().await?;
        debug!("Migrations complete");

        Ok(())
    }

    #[instrument(skip_all, fields(subsystem = M::SUBSYSTEM))]
    async fn iterate<M: Migration>(&self) -> surrealdb::Result<()> {
        let mut prng = WyRand::new();
        let mut migrated = false;
        while let Some(update) = self.next_update::<M>().await? {
            if let Some(res) = self
                .persist
                .execute_in_lock(M::SUBSYSTEM, || async {
                    let mut statements = vec![srql::Statement::Begin(srql::BeginStatement)];
                    update.build(&mut statements);
                    statements.push(srql::Statement::Update(iterate_complete_update(
                        M::SUBSYSTEM,
                        srql::to_value(&update)?,
                    )));
                    statements.push(srql::Statement::Commit(srql::CommitStatement));

                    self.persist.db().query(srql::query(statements)).await
                })
                .await?
            {
                res?.check()?;
                migrated = true;
            } else {
                trace!("Migration locked, sleeping");
                // Introduce a bit of jitter to avoid thundering herd.
                sleep(Duration::from_millis(
                    5000 + prng.generate_range(0..=10_000),
                ))
                .await;
            }
        }

        if migrated {
            debug!("Migration complete");
        } else {
            debug!("No migrations to run");
        }

        Ok(())
    }

    #[instrument(skip_all)]
    async fn next_update<M>(&self) -> surrealdb::Result<Option<M>>
    where
        M: Migration,
    {
        if let Some::<Update<M>>(Update { current }) = self
            .persist
            .db()
            .select((UPDATE_TABLE, M::SUBSYSTEM))
            .await?
        {
            if let Some(next) = current.next() {
                debug!(?next, "Next migration step");
                Ok(Some(next))
            } else {
                Ok(None)
            }
        } else {
            let update = M::default();
            debug!(?update, "Beginning migration");
            Ok(Some(update))
        }
    }
}

fn iterate_complete_update(subsystem: &str, update: srql::Value) -> srql::UpdateStatement {
    srql::UpdateStatement {
        what: srql::thing((UPDATE_TABLE, subsystem)),
        data: srql::Data::SetExpression(vec![
            (
                srql::field("current"),
                srql::Operator::Equal,
                update.clone(),
            ),
            (
                srql::field("history"),
                srql::Operator::Inc,
                srql::array([srql::object([
                    ("timestamp".into(), srql::time_now()),
                    ("update".into(), update),
                ])]),
            ),
        ])
        .into(),
        ..Default::default()
    }
}
