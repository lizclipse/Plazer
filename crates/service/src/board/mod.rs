mod migration;
mod persist;
mod schema;

pub use migration::*;
pub use persist::*;
pub use schema::*;

use async_graphql::{ComplexObject, InputObject, SimpleObject, ID};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use surrealdb::sql::Thing;

use crate::prelude::*;

static TABLE_NAME: &str = "board";

/// A registered account.
#[derive(SimpleObject, Debug, Deserialize)]
#[graphql(complex)]
pub struct Board {
    #[graphql(skip)]
    pub id: Thing,
    #[graphql(skip)]
    pub creator_id: Option<Thing>,

    /// The board's unique handle. This is used to refer to the board in URLs
    /// and by users. It must be unique, but can be changed (if the server allows it).
    pub handle: String,
    /// The board's display name. If not present, the handle is (usually) used instead.
    pub name: Option<String>,
    /// The board's description.
    pub description: Option<String>,

    /// A timestamp indicating when the board was created.
    pub created_at: DateTime<Utc>,
    /// A timestamp indicating the last time the board was updated.
    pub updated_at: DateTime<Utc>,
}

#[ComplexObject]
impl Board {
    /// The board's unique ID.
    ///
    /// This cannot change, and can be safely used to refer to the board permanently.
    async fn id(&self) -> ID {
        self.id.to_gql_id()
    }

    /// The ID of the account that created this board. This cannot be changed,
    /// but is only used for informational purposes, so should not be used for
    /// authorisation.
    async fn creator_id(&self) -> Option<ID> {
        self.creator_id.as_ref().map(|id| id.to_gql_id())
    }
}

#[derive(InputObject, Debug)]
pub struct CreateBoard {
    /// The board's unique handle. This is used to refer to the board in URLs
    /// and by users. It must be unique, but can be changed (if the server allows it).
    ///
    /// This will default to the user's ID if not present.
    #[graphql(validator(min_length = 1, max_length = 128))]
    handle: Option<String>,
    /// The board's display name. If not present, the handle is (usually) used instead.
    #[graphql(validator(max_length = 1024))]
    name: Option<String>,
    /// The board's description.
    #[graphql(validator(min_length = 1, max_length = 32_768))]
    description: Option<String>,
}
