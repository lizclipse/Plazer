use serde::{Deserialize, Serialize};

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

    fn build(&self, statements: &mut Vec<srql::Statement>) {
        use BoardMigration as S;
        match self {
            S::Init => Self::build_init(statements),
        }
    }
}

impl BoardMigration {
    fn build_init(statements: &mut Vec<srql::Statement>) {
        statements.push(srql::define_uniq_index(
            "board_handle_index",
            TABLE_NAME,
            [srql::field("handle")],
        ));
    }
}
