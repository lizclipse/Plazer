use async_graphql::{ComplexObject, InputObject, MaybeUndefined, SimpleObject, ID};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use surrealdb::sql::Thing;

use super::TABLE_NAME;
use crate::{id_obj_impls, prelude::*, query::OpaqueCursor};

pub type BoardCursor = OpaqueCursor<String>;

/// A registered account.
#[derive(SimpleObject, Debug, Clone, Deserialize)]
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
        self.creator_id.as_ref().map(ToGqlId::to_gql_id)
    }
}

id_obj_impls!(Board);

impl Board {
    pub fn create(creator_id: Option<Thing>, params: CreateBoard) -> srql::Query {
        let mut create = vec![];
        creator_id.push_field(srql::field("creator_id"), &mut create);
        params.append(&mut create);
        srql::create_obj_query(TABLE_NAME, create)
    }
}

#[derive(InputObject, Debug, Default, Clone, PartialEq, Eq)]
pub struct CreateBoard {
    /// The board's unique handle. This is used to refer to the board in URLs
    /// and by users. It must be unique, but can be changed (if the server allows it).
    ///
    /// This will default to the user's ID if not present.
    #[graphql(validator(min_length = 1, max_length = 128))]
    pub handle: Option<String>,
    /// The board's display name. If not present, the handle is (usually) used instead.
    #[graphql(validator(min_length = 1, max_length = 1024))]
    pub name: Option<String>,
    /// The board's description.
    #[graphql(validator(min_length = 1, max_length = 32_768))]
    pub description: Option<String>,
}

impl CreateObject for CreateBoard {
    fn append(self, expr: &mut srql::SetExpr) {
        self.handle.push_field(srql::field("handle"), expr);
        self.name.push_field(srql::field("name"), expr);
        self.description
            .push_field(srql::field("description"), expr);
    }
}

#[derive(InputObject, Debug, Default, Clone, PartialEq, Eq)]
pub struct UpdateBoard {
    /// The new handle. If not given, the handle is not changed.
    #[graphql(validator(min_length = 1, max_length = 128))]
    pub handle: Option<String>,
    /// The new name. If not given, the name is not changed. If null is given,
    /// the name is cleared.
    #[graphql(validator(min_length = 1, max_length = 1024))]
    pub name: MaybeUndefined<String>,
    /// The new description. If not given, the description is not changed. If
    /// null is given, the description is cleared.
    #[graphql(validator(min_length = 1, max_length = 32_768))]
    pub description: MaybeUndefined<String>,
}

impl IntoUpdateQuery for UpdateBoard {
    fn into_update(self, thing: srql::Thing) -> Option<srql::Query> {
        let mut update = vec![];
        self.handle.push_field(srql::field("handle"), &mut update);
        self.name.push_field(srql::field("name"), &mut update);
        self.description
            .push_field(srql::field("description"), &mut update);
        srql::update_obj_query(thing, update)
    }
}
