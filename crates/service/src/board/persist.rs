#[cfg(test)]
mod tests;

use async_graphql::connection::{Connection, Edge};
use tracing::instrument;

use super::{Board, BoardCursor, CreateBoard, UpdateBoard, TABLE_NAME};
use crate::{
    account::CurrentAccount,
    persist::Persist,
    prelude::*,
    query::{OpaqueCursor, PaginationInput, PaginationOptions, ResultSlice},
};

pub struct BoardPersist<'a> {
    persist: &'a Persist,
    current: &'a CurrentAccount,
}

impl<'a> BoardPersist<'a> {
    pub fn new(persist: &'a Persist, current: &'a CurrentAccount) -> Self {
        Self { persist, current }
    }

    #[instrument(skip_all)]
    pub async fn get(&self, id: &str) -> Result<Option<Board>> {
        Ok(self.persist.db().select((TABLE_NAME, id)).await?)
    }

    #[instrument(skip_all)]
    pub async fn get_by_handle(&self, handle: &str) -> Result<Option<Board>> {
        let board = self
            .persist
            .db()
            .query(srql::SelectStatement {
                expr: srql::Fields::all(),
                what: srql::table(TABLE_NAME),
                cond: srql::Cond(
                    srql::Expression::Binary {
                        l: srql::field("handle").into(),
                        o: srql::Operator::Equal,
                        r: srql::string(handle).into(),
                    }
                    .into(),
                )
                .into(),
                ..Default::default()
            })
            .await?
            .take(0)?;
        Ok(board)
    }

    #[instrument(skip_all)]
    pub fn list(&self) -> BoardListRequest<'_> {
        BoardListRequest::new(self.persist)
    }

    #[instrument(skip_all)]
    pub async fn create(&self, mut board: CreateBoard) -> Result<Board> {
        // TODO: check config to see if anon users can create boards
        // TODO: check perms to see if authd user can create boards

        if board.handle.is_none() {
            board.handle = self
                .current
                .user_id()
                .map_err(|_| Error::MissingIdent)?
                .to_owned()
                .into();
        }

        let board = self
            .persist
            .db()
            .query(Board::create(
                self.current.id().map(ToAccountThing::to_account_thing).ok(),
                board,
            ))
            .await?
            .take(0)?;

        match board {
            Some(board) => Ok(board),
            None => Err(Error::UnavailableIdent),
        }
    }

    #[instrument(skip_all)]
    pub async fn update(&self, id: &str, update: UpdateBoard) -> Result<Option<Board>> {
        // TODO: check config to see if anon users can update boards
        // TODO: check perms to see if authd user can update boards

        let board = if let Some(update) = update.into_update((TABLE_NAME, id).into()) {
            self.persist.db().query(update).await?.take(0)?
        } else {
            self.get(id).await?
        };

        Ok(board)
    }

    #[instrument(skip_all)]
    pub async fn delete(&self, id: &str) -> Result<Option<Board>> {
        let board = self.persist.db().delete((TABLE_NAME, id)).await?;
        Ok(board)
    }
}

pub struct BoardListRequest<'a> {
    persist: &'a Persist,
    pagination: Option<PaginationInput<OpaqueCursor<String>>>,
}

impl<'a> BoardListRequest<'a> {
    fn new(persist: &'a Persist) -> Self {
        Self {
            persist,
            pagination: None,
        }
    }

    pub fn with_pagination(
        mut self,
        args: impl Into<PaginationInput<OpaqueCursor<String>>>,
    ) -> Self {
        self.pagination = Some(args.into());
        self
    }

    #[instrument(skip_all)]
    pub async fn execute(self) -> Result<Connection<BoardCursor, Board>> {
        let PaginationOptions {
            cond,
            order,
            limit,
            result_slice_opts,
        } = (self.pagination, TABLE_NAME).into();

        let query = srql::SelectStatement {
            expr: srql::Fields::all(),
            what: srql::table(TABLE_NAME),
            order: srql::Orders(order.into_iter().collect()).into(),
            cond,
            limit,
            ..Default::default()
        };

        let boards: Vec<Board> = self.persist.db().query(query).await?.take(0)?;
        let ResultSlice {
            results: boards,
            has_previous_page,
            has_next_page,
        } = ResultSlice::new(boards, result_slice_opts);

        let mut connection = Connection::new(has_previous_page, has_next_page);
        connection.edges = boards
            .into_iter()
            .map(|board| Edge::new(OpaqueCursor(board.id.to_gql_id().0), board))
            .collect();

        Ok(connection)
    }
}

#[cfg(test)]
mod testing {
    use async_trait::async_trait;

    use crate::{
        account::testing::TestData,
        board::{Board, CreateBoard},
    };

    use super::BoardPersist;

    #[async_trait]
    pub trait BoardTestData {
        fn board(&self) -> BoardPersist<'_>;
        async fn generate_board(&self) -> Board;
        async fn generate_boards(&self, count: usize) -> Vec<Board>;
    }

    #[async_trait]
    impl BoardTestData for TestData {
        fn board(&self) -> BoardPersist<'_> {
            BoardPersist::new(&self.persist, &self.current)
        }

        async fn generate_board(&self) -> Board {
            let board_persist = self.board();
            board_persist
                .create(CreateBoard {
                    handle: Some("test".into()),
                    name: Some("Test".into()),
                    description: Some("Test".into()),
                })
                .await
                .unwrap()
        }

        async fn generate_boards(&self, count: usize) -> Vec<Board> {
            let board_persist = self.board();
            let mut boards = Vec::with_capacity(count);
            for i in 0..count {
                let board = CreateBoard {
                    handle: Some(format!("test-{i}")),
                    name: Some(format!("Test {i}")),
                    description: Some(format!("Test {i}")),
                };

                let res = board_persist.create(board).await;
                assert!(res.is_ok());
                boards.push(res.unwrap());
            }
            boards.sort_by(|a, b| b.id.cmp(&a.id));
            boards
        }
    }
}
