use async_graphql::connection::{Connection, Edge};
use indoc::indoc;
use tracing::instrument;

use crate::{
    account::CurrentAccount,
    persist::Persist,
    prelude::*,
    query::{values_table, OpaqueCursor, PaginationInput, PaginationOptions, ResultSlice},
};

use super::{Board, BoardCursor, CreateBoard, TABLE_NAME};

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
            .query("SELECT * FROM type::table($tbl) WHERE handle = $handle")
            .bind(("tbl", TABLE_NAME))
            .bind(("handle", handle))
            .await?
            .take(0)?;
        Ok(board)
    }

    #[instrument(skip_all)]
    pub async fn create(&self, board: CreateBoard) -> Result<Board> {
        // TODO: check config to see if anon users can create boards
        // TODO: check perms to see if authd user can create boards

        let handle = match board.handle {
            Some(handle) => handle,
            None => self
                .current
                .user_id()
                .map_err(|_| Error::MissingIdent)?
                .to_owned(),
        };

        let board = self
            .persist
            .db()
            .query(indoc! {"
                CREATE type::thing($tbl, rand::uuid::v7()) SET
                    creator_id = $creator_id,
                    handle = $handle,
                    name = $name,
                    description = $description,

                    updated_at = time::now()
            "})
            .bind(("tbl", TABLE_NAME))
            .bind((
                "creator_id",
                self.current.id().map(ToAccountThing::to_account_thing).ok(),
            ))
            .bind(("handle", handle))
            .bind(("name", board.name))
            .bind(("description", board.description))
            .await?
            .take(0)?;

        match board {
            Some(board) => Ok(board),
            None => Err(Error::UnavailableIdent),
        }
    }

    #[instrument(skip_all)]
    pub fn list(&self) -> BoardListRequest<'_> {
        BoardListRequest::new(self.persist)
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
            what: values_table(TABLE_NAME),
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
mod tests {
    use pretty_assertions::assert_eq;
    use std::{collections::VecDeque, pin::pin};

    use super::{testing::BoardTestData as _, *};
    use crate::{account::testing::*, query::testing::Paginator};

    #[tokio::test]
    async fn test_create() {
        let data = TestData::new().await;
        let board_persist = data.board();

        let board = CreateBoard {
            handle: Some("test".into()),
            name: Some("Test".into()),
            description: Some("Test".into()),
        };

        let res = board_persist.create(board).await;
        println!("{res:?}");
        assert!(res.is_ok());

        let res = res.unwrap();
        assert_eq!(res.handle, "test");
        assert_eq!(res.name, Some("Test".to_owned()));
        assert_eq!(res.description, Some("Test".to_owned()));
    }

    #[tokio::test]
    async fn test_get() {
        let data = TestData::new().await;
        let board_persist = data.board();
        let board = data.generate_board().await;

        let res = board_persist.get(&board.id.id.to_raw()).await;
        println!("{res:?}");
        assert!(res.is_ok());

        let res = res.unwrap();
        assert!(res.is_some());

        let res = res.unwrap();
        println!("{res:?}");
        assert_eq!(res.id, board.id);
        assert_eq!(res.creator_id, board.creator_id);
        assert_eq!(res.handle, board.handle);
        assert_eq!(res.name, board.name);
        assert_eq!(res.description, board.description);
    }

    #[tokio::test]
    async fn test_get_handle() {
        let data = TestData::new().await;
        let board_persist = data.board();
        let board = data.generate_board().await;

        let res = board_persist.get_by_handle(&board.handle).await;
        println!("{res:?}");
        assert!(res.is_ok());

        let res = res.unwrap();
        assert!(res.is_some());

        let res = res.unwrap();
        println!("{res:?}");
        assert_eq!(res.id, board.id);
        assert_eq!(res.creator_id, board.creator_id);
        assert_eq!(res.handle, board.handle);
        assert_eq!(res.name, board.name);
        assert_eq!(res.description, board.description);
    }

    #[tokio::test]
    async fn test_duplicate_handle() {
        let data = TestData::new().await;
        let board_persist = data.board();
        let board = data.generate_board().await;

        let create = CreateBoard {
            handle: Some(board.handle),
            name: Some("Test".into()),
            description: Some("Test".into()),
        };

        let res = board_persist.create(create).await;
        println!("{res:?}");
        assert!(res.is_err());

        let res = res.unwrap_err();
        println!("{res:?}");
        assert_eq!(res, Error::UnavailableIdent);
    }

    #[tokio::test]
    async fn test_anon_create_default_fail() {
        let data = TestData::new().await;
        let board_persist = data.board();

        let board = CreateBoard {
            handle: None,
            name: Some("Test".into()),
            description: Some("Test".into()),
        };

        let res = board_persist.create(board).await;
        println!("{res:?}");
        assert!(res.is_err());

        let res = res.unwrap_err();
        println!("{res:?}");
        assert_eq!(res, Error::MissingIdent);
    }

    #[tokio::test]
    async fn test_authed_create_default() {
        let (data, acc) = TestData::with_user().await;
        let board_persist = data.board();

        let board = CreateBoard {
            handle: None,
            name: Some("Test".into()),
            description: Some("Test".into()),
        };

        let res = board_persist.create(board).await;
        println!("{res:?}");
        assert!(res.is_ok());

        let res = res.unwrap();
        println!("{res:?}");
        assert_eq!(res.handle, acc.user_id);
        assert_eq!(res.creator_id, Some(acc.id));
    }

    #[tokio::test]
    async fn test_forward_pagination() {
        let (data, _) = TestData::with_user().await;
        let test_data = data.generate_boards(50).await;
        let board_persist = pin!(data.board());

        let mut results = vec![];
        let mut paginator = Paginator::new(|cursor| async {
            board_persist
                .list()
                .with_pagination(PaginationInput::new().forward(10).set_after(cursor))
                .execute()
                .await
        });

        while let Some(res) = paginator.next().await {
            assert!(res.is_ok());

            let res = res.unwrap();
            assert_eq!(res.len(), 10);

            results.extend(res);
        }

        assert_eq!(results.len(), test_data.len());
        assert_eq!(results, test_data);
    }

    #[tokio::test]
    async fn test_backward_pagination() {
        let (data, _) = TestData::with_user().await;
        let test_data = data.generate_boards(50).await;
        let board_persist = pin!(data.board());

        let mut results = VecDeque::new();
        let mut paginator = Paginator::new(|cursor| async {
            board_persist
                .list()
                .with_pagination(PaginationInput::new().backward(10).set_before(cursor))
                .execute()
                .await
        })
        .reversed();

        // Because reverse pagination still keeps the items in each page in
        // the same order as forward pagination, but the pages are in reverse
        // order, we need to collect the pages in reverse order to get the
        // correct order of items while keeping the item order.
        while let Some(res) = paginator.next().await {
            assert!(res.is_ok());

            let res = res.unwrap();
            assert_eq!(res.len(), 10);

            results.push_front(res);
        }

        let results: Vec<_> = results.into_iter().flatten().collect();
        assert_eq!(results.len(), test_data.len());
        assert_eq!(results, test_data);
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
