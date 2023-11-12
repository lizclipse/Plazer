use std::collections::VecDeque;

use async_graphql::MaybeUndefined;

use super::{testing::PostTestData as _, *};
use crate::{account::testing::*, board::testing::BoardTestData as _, query::testing::Paginator};

#[tokio::test]
async fn test_create_no_board() {
    let data = TestData::new().await;
    let post_persist = data.post();

    let post = CreatePost {
        title: Some("Test".into()),
        content: Some("Test".into()),
        ..Default::default()
    };

    let res = post_persist.create(post).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    assert!(res.board_id.is_none());
    assert_eq!(res.title, Some("Test".to_owned()));
    assert_eq!(res.content, Some("Test".to_owned()));

    let relations: Option<i32> = data
        .persist
        .db()
        .query(format!(
            "SELECT count() as count FROM {CONTAINS_TABLE_NAME}"
        ))
        .await
        .unwrap()
        .take("count")
        .unwrap();

    assert_eq!(relations, None);
}

#[tokio::test]
async fn test_create_with_board() {
    let data = TestData::new().await;
    let post_persist = data.post();

    let board = data.generate_board().await;

    let post = CreatePost {
        board_id: Some(board.id.to_gql_id()),
        title: Some("Test".into()),
        content: Some("Test".into()),
    };

    let res = post_persist.create(post).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    assert_eq!(res.board_id, Some(board.id));
    assert_eq!(res.title, Some("Test".to_owned()));
    assert_eq!(res.content, Some("Test".to_owned()));

    let relations: Option<i32> = data
        .persist
        .db()
        .query(format!(
            "SELECT count() as count FROM {CONTAINS_TABLE_NAME}"
        ))
        .await
        .unwrap()
        .take("count")
        .unwrap();

    assert_eq!(relations, Some(1));
}

#[tokio::test]
async fn test_get() {
    let data = TestData::new().await;
    let post_persist = data.post();

    let post = data.generate_post().await;

    let res = post_persist.get(&post.id.to_gql_id()).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    println!("{res:?}");
    assert!(res.is_some());

    let res = res.unwrap();
    assert_eq!(res.id, post.id);
    assert_eq!(res.board_id, post.board_id);
    assert_eq!(res.title, post.title);
    assert_eq!(res.content, post.content);
}

#[tokio::test]
async fn test_forward_pagination() {
    let (data, _) = TestData::with_user().await;
    let test_data = data.generate_posts(50).await;
    let post_persist = data.post();

    let mut results = vec![];
    let mut paginator = Paginator::new(|cursor| async {
        post_persist
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
    let test_data = data.generate_posts(50).await;
    let post_persist = data.post();

    let mut results = VecDeque::new();
    let mut paginator = Paginator::new(|cursor| async {
        post_persist
            .list()
            .with_pagination(PaginationInput::new().backward(10).set_before(cursor))
            .execute()
            .await
    })
    .reversed();

    // See `crate::board::persist::tests::test_backward_pagination` for why we
    // use a `VecDeque` here.
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

#[tokio::test]
async fn test_empty_update() {
    let data = TestData::new().await;
    let post_persist = data.post();

    let post = data.generate_post().await;

    let res = post_persist
        .update(&post.id.to_gql_id(), UpdatePost::default())
        .await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    println!("{res:?}");

    let res = res.unwrap();
    assert_eq!(res.id, post.id);
    assert_eq!(res.board_id, post.board_id);
    assert_eq!(res.title, post.title);
    assert_eq!(res.content, post.content);
}

#[tokio::test]
async fn test_update_title() {
    let data = TestData::new().await;
    let post_persist = data.post();

    let post = data.generate_post().await;

    let res = post_persist
        .update(
            &post.id.to_gql_id(),
            UpdatePost {
                title: MaybeUndefined::Value("Test".into()),
                ..Default::default()
            },
        )
        .await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    println!("{res:?}");

    let res = res.unwrap();
    assert_eq!(res.id, post.id);
    assert_eq!(res.board_id, post.board_id);
    assert_eq!(res.title, Some("Test".to_owned()));
    assert_eq!(res.content, post.content);
}

#[tokio::test]
async fn test_update_content() {
    let data = TestData::new().await;
    let post_persist = data.post();

    let post = data.generate_post().await;

    let res = post_persist
        .update(
            &post.id.to_gql_id(),
            UpdatePost {
                content: MaybeUndefined::Value("Test".into()),
                ..Default::default()
            },
        )
        .await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    println!("{res:?}");

    let res = res.unwrap();
    assert_eq!(res.id, post.id);
    assert_eq!(res.board_id, post.board_id);
    assert_eq!(res.title, post.title);
    assert_eq!(res.content, Some("Test".to_owned()));
}

#[tokio::test]
async fn test_update_title_null() {
    let data = TestData::new().await;
    let post_persist = data.post();

    let post = data.generate_post().await;

    let res = post_persist
        .update(
            &post.id.to_gql_id(),
            UpdatePost {
                title: MaybeUndefined::Null,
                ..Default::default()
            },
        )
        .await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    println!("{res:?}");

    let res = res.unwrap();
    assert_eq!(res.id, post.id);
    assert_eq!(res.board_id, post.board_id);
    assert_eq!(res.title, None);
    assert_eq!(res.content, post.content);
}

#[tokio::test]
async fn test_update_nonexistent() {
    let data = TestData::new().await;
    let post_persist = data.post();

    let res = post_persist
        .update(
            "test",
            UpdatePost {
                title: MaybeUndefined::Value("Test".into()),
                ..Default::default()
            },
        )
        .await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    println!("{res:?}");
    assert!(res.is_none());
}

#[tokio::test]
async fn test_delete() {
    let data = TestData::new().await;
    let post_persist = data.post();

    let post = data.generate_post().await;

    let res = post_persist.delete(&post.id.to_gql_id()).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    println!("{res:?}");
    assert!(res.is_some());

    let res = post_persist.get(&post.id.to_gql_id()).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    println!("{res:?}");
    assert!(res.is_none());
}

#[tokio::test]
async fn test_delete_nonexistent() {
    let data = TestData::new().await;
    let post_persist = data.post();

    let res = post_persist.delete("test").await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    println!("{res:?}");
    assert!(res.is_none());
}
