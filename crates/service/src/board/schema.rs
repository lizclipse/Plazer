use async_graphql::{Context, Object};
use tracing::instrument;

use crate::prelude::*;

use super::{Board, CreateBoard};

#[derive(Default)]
pub struct BoardQuery;

#[Object]
impl BoardQuery {
    /// Gets a board by its handle.
    #[instrument(skip_all)]
    async fn board(&self, ctx: &Context<'_>, handle: String) -> GqlResult<Option<Board>> {
        ctx.board_persist().get_by_handle(&handle).await.extend()
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
}
