//! Re-exports of common types and traits that are used throughout the service.

pub use async_graphql::ResultExt as _;
pub use futures::{Stream as AsyncIterator, StreamExt as _};

pub use crate::{
    account::ToAccountThing,
    conv::{AsMaybeStr, IntoGqlId, ToGqlId},
    error::{Error, GqlError, GqlResult, Result, SrlDbError, SrlError},
    persist::PersistExt as _,
    query::{srql, CreateObject, IntoUpdateQuery, QueryValue},
};
