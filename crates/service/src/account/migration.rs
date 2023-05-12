use serde::{Deserialize, Serialize};
use surrealdb::{method::Query, Connection};

use crate::migration::Migration;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountMigration {
    #[default]
    Init,
}

impl Migration for AccountMigration {
    fn subsystem() -> &'static str {
        "subsys_account"
    }

    fn next(self) -> Option<Self> {
        match self {
            Self::Init => None,
        }
    }

    fn build<'a, C>(&self, query: Query<'a, C>) -> Query<'a, C>
    where
        C: Connection,
    {
        use AccountMigration::*;
        match self {
            Init => Self::build_init(query),
        }
    }
}

impl AccountMigration {
    fn build_init<'a, C>(query: Query<'a, C>) -> Query<'a, C>
    where
        C: Connection,
    {
        query
    }
}
