use serde::{Deserialize, Serialize};
use surrealdb::{method::Query, Connection};

use super::TABLE_NAME;
use crate::{migration::Migration, prelude::*};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardMigration {
    #[default]
    Init,
}

impl Migration for BoardMigration {
    const SUBSYSTEM: &'static str = "subsys_board";

    fn next(self) -> Option<Self> {
        match self {
            Self::Init => None,
        }
    }

    fn build<'a, C>(&self, q: Query<'a, C>) -> Query<'a, C>
    where
        C: Connection,
    {
        use BoardMigration as S;
        match self {
            S::Init => Self::build_init(q),
        }
    }
}

impl BoardMigration {
    fn build_init<C>(q: Query<'_, C>) -> Query<'_, C>
    where
        C: Connection,
    {
        q.query(srql::define_uniq_index(
            "board_handle_index",
            TABLE_NAME,
            [srql::field("handle")],
        ))
    }
}
