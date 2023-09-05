use serde::{Deserialize, Serialize};
use surrealdb::{method::Query, Connection};

use super::TABLE_NAME;
use crate::{migration::Migration, prelude::*};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountMigration {
    #[default]
    Init,
}

impl Migration for AccountMigration {
    const SUBSYSTEM: &'static str = "subsys_account";

    fn next(self) -> Option<Self> {
        match self {
            Self::Init => None,
        }
    }

    fn build<'a, C>(&self, q: Query<'a, C>) -> Query<'a, C>
    where
        C: Connection,
    {
        use AccountMigration as S;
        match self {
            S::Init => Self::build_init(q),
        }
    }
}

impl AccountMigration {
    fn build_init<C>(q: Query<'_, C>) -> Query<'_, C>
    where
        C: Connection,
    {
        q.query(srql::define_uniq_index(
            "account_user_id_index",
            TABLE_NAME,
            [srql::field("user_id")],
        ))
    }
}
