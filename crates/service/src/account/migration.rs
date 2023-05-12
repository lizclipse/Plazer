use indoc::formatdoc;
use serde::{Deserialize, Serialize};
use surrealdb::{method::Query, Connection};

use crate::migration::Migration;

use super::TABLE_NAME;

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
        use AccountMigration::*;
        match self {
            Init => Self::build_init(q),
        }
    }
}

impl AccountMigration {
    fn build_init<C>(q: Query<'_, C>) -> Query<'_, C>
    where
        C: Connection,
    {
        q.query(formatdoc! {"
            DEFINE INDEX account_handle_index
            ON {tbl}
            FIELDS
                handle UNIQUE
        ", tbl = TABLE_NAME})
    }
}
