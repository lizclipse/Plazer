use async_graphql::Context;
use surrealdb::{engine::any::Any, Surreal};

use crate::account::{AccountPersist, CurrentAccount};

pub type PersistLayer = Surreal<Any>;

static DB: PersistLayer = Surreal::init();

pub trait PersistExt {
    fn account_persist(&self) -> AccountPersist;
}

pub struct Persist;

impl Persist {
    pub fn new() -> Self {
        Self
    }

    pub fn db(&self) -> &PersistLayer {
        &DB
    }
}

impl PersistExt for Context<'_> {
    fn account_persist(&self) -> AccountPersist {
        AccountPersist::new(
            self.data_unchecked::<Persist>(),
            self.data_unchecked::<CurrentAccount>(),
        )
    }
}
