use async_graphql::{ComplexObject, InputObject, MaybeUndefined, SimpleObject, ID};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use surrealdb::sql::Thing;

use super::POST_TABLE_NAME;
use crate::{board::BOARD_TABLE_NAME, id_obj_impls, prelude::*, query::OpaqueCursor};

pub type PostCursor = OpaqueCursor<String>;

#[derive(SimpleObject, Debug, Clone, Deserialize)]
#[graphql(complex)]
pub struct Post {
    #[graphql(skip)]
    pub id: Thing,
    #[graphql(skip)]
    pub creator_id: Option<Thing>,
    #[graphql(skip)]
    pub board_id: Option<Thing>,

    /// The post's title.
    pub title: Option<String>,
    /// The post's content.
    pub content: Option<String>,

    /// A timestamp indicating the last time the board was updated.
    ///
    /// If not present, the post has never been updated.
    pub updated_at: Option<DateTime<Utc>>,
}

#[ComplexObject]
impl Post {
    /// The post's unique ID.
    ///
    /// This cannot change, and can be safely used to refer to the post permanently.
    async fn id(&self) -> ID {
        self.id.to_gql_id()
    }

    /// The ID of the board that this post belongs to. This cannot be changed,
    /// and should not be used for authorisation.
    async fn board_id(&self) -> Option<ID> {
        self.board_id.as_ref().map(ToGqlId::to_gql_id)
    }

    /// The ID of the account that created this post. This cannot be changed,
    /// but is only used for informational purposes, so should not be used for
    /// authorisation.
    async fn creator_id(&self) -> Option<ID> {
        self.creator_id.as_ref().map(ToGqlId::to_gql_id)
    }
}

id_obj_impls!(Post);

impl Post {
    pub fn create(
        creator_id: Option<Thing>,
        params: CreatePost,
    ) -> (Option<(Thing, String)>, srql::CreateStatement) {
        let mut create = vec![];
        creator_id.push_field(srql::field("creator_id"), &mut create);
        let board_id = params.board_id.clone();
        params.append(&mut create);
        let id = srql::ulid();
        (
            board_id.map(|board_id| {
                (
                    srql::Thing::from((BOARD_TABLE_NAME.to_owned(), board_id.0)),
                    id.clone(),
                )
            }),
            srql::obj_create_query_id(POST_TABLE_NAME, create, id.into()),
        )
    }
}

#[derive(InputObject, Debug, Default, Clone, PartialEq, Eq)]
pub struct CreatePost {
    /// The ID of the board that this post belongs to. This cannot be changed.
    pub board_id: Option<ID>,
    /// The post's title.
    #[graphql(validator(max_length = 1024))]
    pub title: Option<String>,
    /// The post's content.
    #[graphql(validator(max_length = 32_768))]
    pub content: Option<String>,
}

impl CreateObject for CreatePost {
    fn append(self, expr: &mut srql::SetExpr) {
        self.board_id
            .map(|id| (BOARD_TABLE_NAME, id))
            .push_field(srql::field("board_id"), expr);
        self.title.push_field(srql::field("title"), expr);
        self.content.push_field(srql::field("content"), expr);
    }
}

#[derive(InputObject, Debug, Default, Clone, PartialEq, Eq)]
pub struct UpdatePost {
    /// The post's title. If not given, the title is not changed. If null is given,
    /// the title is cleared.
    #[graphql(validator(max_length = 1024))]
    pub title: MaybeUndefined<String>,
    /// The post's content. If not given, the content is not changed. If null is given,
    /// the content is cleared.
    #[graphql(validator(max_length = 32_768))]
    pub content: MaybeUndefined<String>,
}

impl IntoUpdateQuery for UpdatePost {
    fn into_update(self, thing: srql::Thing) -> Option<srql::UpdateStatement> {
        let mut update = vec![];
        self.title.push_field(srql::field("title"), &mut update);
        self.content.push_field(srql::field("content"), &mut update);
        srql::obj_update_query(thing, update)
    }
}
