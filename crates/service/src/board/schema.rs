use async_graphql::{connection::Connection, Context, Object, ID};
use tracing::instrument;

use super::{Board, BoardCursor, CreateBoard, UpdateBoard};
use crate::{prelude::*, query::PaginationArgs};

#[derive(Default)]
pub struct BoardQuery;

#[Object]
impl BoardQuery {
    /// Gets a board by its handle.
    #[instrument(skip_all)]
    async fn board(&self, ctx: &Context<'_>, handle: String) -> GqlResult<Option<Board>> {
        ctx.board_persist().get_by_handle(&handle).await.extend()
    }

    /// Lists boards.
    #[instrument(skip_all)]
    async fn boards(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> GqlResult<Connection<BoardCursor, Board>> {
        ctx.board_persist()
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
pub struct BoardMutation;

#[Object]
impl BoardMutation {
    /// Creates a new board.
    #[instrument(skip_all)]
    async fn create_board(&self, ctx: &Context<'_>, create: CreateBoard) -> GqlResult<Board> {
        ctx.board_persist().create(create).await.extend()
    }

    /// Updates a board.
    #[instrument(skip_all)]
    async fn update_board(
        &self,
        ctx: &Context<'_>,
        id: ID,
        update: UpdateBoard,
    ) -> GqlResult<Board> {
        ctx.board_persist().update(&id, update).await.extend()
    }
}
