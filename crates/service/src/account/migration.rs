use serde::{Deserialize, Serialize};

use super::ACC_TABLE_NAME;
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

    fn build(&self, statements: &mut Vec<srql::Statement>) {
        use AccountMigration as S;
        match self {
            S::Init => Self::build_init(statements),
        }
    }
}

impl AccountMigration {
    fn build_init(statements: &mut Vec<srql::Statement>) {
        statements.push(srql::define_uniq_index(
            "account_user_id_index",
            ACC_TABLE_NAME,
            [srql::field("user_id")],
        ));
    }
}
