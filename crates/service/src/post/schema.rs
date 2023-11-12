use async_graphql::{connection::Connection, Context, Object, ID};
use tracing::instrument;

use super::{CreatePost, Post, PostCursor, UpdatePost};
use crate::{prelude::*, query::PaginationArgs};

#[derive(Default)]
pub struct PostQuery;

#[Object]
impl PostQuery {
    /// Gets a post by its ID.
    #[instrument(skip_all)]
    async fn post(&self, ctx: &Context<'_>, id: ID) -> GqlResult<Option<Post>> {
        ctx.post_persist().get(&id).await.extend()
    }

    /// Lists posts.
    #[instrument(skip_all)]
    async fn posts(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> GqlResult<Connection<PostCursor, Post>> {
        ctx.post_persist()
            .list()
            .with_pagination(
                PaginationArgs {
                    after,
                    before,
                    first,
                    last,
                }
                .validate()
                .extend()?,
            )
            .execute()
            .await
            .extend()
    }
}

#[derive(Default)]
pub struct PostMutation;

#[Object]
impl PostMutation {
    /// Creates a new post.
    #[instrument(skip_all)]
    async fn create_post(&self, ctx: &Context<'_>, create: CreatePost) -> GqlResult<Post> {
        ctx.post_persist().create(create).await.extend()
    }

    /// Updates a post.
    #[instrument(skip_all)]
    async fn update_post(
        &self,
        ctx: &Context<'_>,
        id: ID,
        update: UpdatePost,
    ) -> GqlResult<Option<Post>> {
        ctx.post_persist().update(&id, update).await.extend()
    }

    /// Deletes a post.
    #[instrument(skip_all)]
    async fn delete_post(&self, ctx: &Context<'_>, id: ID) -> GqlResult<Option<Post>> {
        ctx.post_persist().delete(&id).await.extend()
    }
}
