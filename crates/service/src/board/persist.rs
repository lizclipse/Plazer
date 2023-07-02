use indoc::indoc;
use tracing::instrument;

use crate::{
    account::CurrentAccount,
    error::{Error, Result},
    persist::Persist,
};

use super::{Board, CreateBoard, TABLE_NAME};

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
                CREATE type::table($tbl) SET
                    creator_id = $creator_id,
                    handle = $handle,
                    name = $name,
                    description = $description,

                    created_at = time::now(),
                    updated_at = time::now()
            "})
            .bind(("tbl", TABLE_NAME))
            .bind(("creator_id", self.current.user_id().ok()))
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{account::testing::*, board::persist::testing::BoardTestData as _};

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
    }
}
