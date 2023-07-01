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
            .bind(("creator_id", self.current.user_id()?))
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
