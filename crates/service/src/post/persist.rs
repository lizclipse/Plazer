#[cfg(test)]
mod tests;

use async_graphql::connection::{Connection, Edge};
use tracing::instrument;

use super::{CreatePost, Post, PostCursor, UpdatePost, CONTAINS_TABLE_NAME, POST_TABLE_NAME};
use crate::{
    account::CurrentAccount,
    persist::Persist,
    prelude::*,
    query::{OpaqueCursor, PaginationInput, PaginationOptions, ResultSlice},
};

pub struct PostPersist<'a> {
    persist: &'a Persist,
    current: &'a CurrentAccount,
}

impl<'a> PostPersist<'a> {
    pub fn new(persist: &'a Persist, current: &'a CurrentAccount) -> Self {
        Self { persist, current }
    }

    #[instrument(skip_all)]
    pub async fn get(&self, id: &str) -> Result<Option<Post>> {
        Ok(self.persist.db().select((POST_TABLE_NAME, id)).await?)
    }

    #[instrument(skip_all)]
    pub fn list(&self) -> PostListRequest<'_> {
        PostListRequest::new(self.persist)
    }

    #[instrument(skip_all)]
    pub async fn create(&self, post: CreatePost) -> Result<Post> {
        // TODO: check config to see if anon users can create posts on this board
        // TODO: check perms to see if authd user can create posts on this board

        let (ids, create) = Post::create(
            self.current.id().map(ToAccountThing::to_account_thing).ok(),
            post,
        );

        let query = if let Some((board_id, post_id)) = ids {
            vec![
                srql::trans_begin(),
                srql::Statement::Create(create),
                srql::Statement::Relate(srql::RelateStatement {
                    kind: srql::Table(CONTAINS_TABLE_NAME.to_owned()).into(),
                    from: board_id.into(),
                    with: srql::Thing::from((POST_TABLE_NAME.to_owned(), post_id)).into(),
                    ..Default::default()
                }),
                srql::trans_end(),
            ]
        } else {
            vec![srql::Statement::Create(create)]
        };

        let post = self.persist.db().query(query).await?.take(0)?;

        match post {
            Some(post) => Ok(post),
            None => Err(Error::UnavailableIdent),
        }
    }

    #[instrument(skip_all)]
    pub async fn update(&self, id: &str, update: UpdatePost) -> Result<Option<Post>> {
        // TODO: check config to see if anon users can update posts
        // TODO: check perms to see if authd user can update posts

        let post = if let Some(update) = update.into_update((POST_TABLE_NAME, id).into()) {
            self.persist.db().query(update).await?.take(0)?
        } else {
            self.get(id).await?
        };

        Ok(post)
    }

    #[instrument(skip_all)]
    pub async fn delete(&self, id: &str) -> Result<Option<Post>> {
        // TODO: check config to see if anon users can delete posts
        // TODO: check perms to see if authd user can delete posts

        let post = self.persist.db().delete((POST_TABLE_NAME, id)).await?;
        Ok(post)
    }
}

pub struct PostListRequest<'a> {
    persist: &'a Persist,
    pagination: Option<PaginationInput<OpaqueCursor<String>>>,
}

impl<'a> PostListRequest<'a> {
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
    pub async fn execute(self) -> Result<Connection<PostCursor, Post>> {
        let PaginationOptions {
            cond,
            order,
            limit,
            result_slice_opts,
        } = (self.pagination, POST_TABLE_NAME).into();

        let query = srql::SelectStatement {
            expr: srql::Fields::all(),
            what: srql::table(POST_TABLE_NAME),
            order: srql::Orders(order.into_iter().collect()).into(),
            cond,
            limit,
            ..Default::default()
        };

        let posts: Vec<Post> = self.persist.db().query(query).await?.take(0)?;
        let ResultSlice {
            results: posts,
            has_previous_page,
            has_next_page,
        } = ResultSlice::new(posts, result_slice_opts);

        let mut connection = Connection::new(has_previous_page, has_next_page);
        connection.edges = posts
            .into_iter()
            .map(|post| Edge::new(OpaqueCursor(post.id.to_gql_id().0), post))
            .collect();

        Ok(connection)
    }
}

#[cfg(test)]
pub mod testing {
    use async_trait::async_trait;

    use crate::{
        account::testing::TestData,
        post::{CreatePost, Post},
        prelude::*,
    };

    use super::PostPersist;

    #[async_trait]
    pub trait PostTestData {
        fn post(&self) -> PostPersist<'_>;

        async fn generate_post(&self) -> Post {
            let post_persist = self.post();
            post_persist
                .create(CreatePost {
                    content: Some("Test".into()),
                    ..Default::default()
                })
                .await
                .unwrap()
        }

        async fn generate_post_in(&self, board_id: &srql::Thing) -> Post {
            let post_persist = self.post();
            post_persist
                .create(CreatePost {
                    board_id: Some(board_id.to_gql_id()),
                    content: Some("Test".into()),
                    ..Default::default()
                })
                .await
                .unwrap()
        }

        async fn generate_posts(&self, count: usize) -> Vec<Post> {
            let post_persist = self.post();
            let mut posts = Vec::with_capacity(count);
            for i in 0..count {
                let post = CreatePost {
                    content: Some(format!("Test {i}")),
                    ..Default::default()
                };

                let res = post_persist.create(post).await;
                assert!(res.is_ok());
                posts.push(res.unwrap());
            }
            posts.sort_by(|a, b| b.id.cmp(&a.id));
            posts
        }
    }

    #[async_trait]
    impl PostTestData for TestData {
        fn post(&self) -> PostPersist<'_> {
            PostPersist::new(&self.persist, &self.current)
        }
    }
}
