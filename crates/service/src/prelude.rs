//! Re-exports of common types and traits that are used throughout the service.

pub use async_graphql::ResultExt as _;

pub use crate::{
    account::ToAccountThing as _,
    conv::{AsMaybeStr as _, IntoGqlId as _, ToGqlId as _},
    error::{Error, GqlError, GqlResult, Result, SrlDbError, SrlError},
    persist::PersistExt as _,
    query::srql,
};
