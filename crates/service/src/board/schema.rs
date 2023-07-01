use async_graphql::{Context, Object, Result, ResultExt};
use tracing::instrument;

use crate::persist::PersistExt;

use super::{Board, CreateBoard};

#[derive(Default)]
pub struct BoardQuery;

#[Object]
impl BoardQuery {
    /// Gets a board by its handle.
    #[instrument(skip_all)]
    async fn get_board(&self, ctx: &Context<'_>, handle: String) -> Result<Option<Board>> {
        ctx.board_persist().get_by_handle(&handle).await.extend()
    }
}

#[derive(Default)]
pub struct BoardMutation;

#[Object]
impl BoardMutation {
    /// Creates a new board.
    #[instrument(skip_all)]
    async fn create_board(&self, ctx: &Context<'_>, create: CreateBoard) -> Result<Board> {
        ctx.board_persist().create(create).await.extend()
    }
}
