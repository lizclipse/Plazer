use async_graphql::MaybeUndefined;
use pretty_assertions::assert_eq;
use std::collections::VecDeque;

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
    assert_eq!(res.updated_at, board.updated_at);
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
    let board_persist = data.board();

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
    let board_persist = data.board();

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

#[tokio::test]
async fn test_empty_update() {
    let data = TestData::new().await;
    let board_persist = data.board();
    let board = data.generate_board().await;

    let update = UpdateBoard::default();

    let res = board_persist.update(&board.id.id.to_raw(), update).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    println!("{res:?}");
    assert!(res.is_some());

    let res = res.unwrap();
    println!("{res:?}");
    assert_eq!(res.id, board.id);
    assert_eq!(res.creator_id, board.creator_id);
    assert_eq!(res.handle, board.handle);
    assert_eq!(res.name, board.name);
    assert_eq!(res.description, board.description);
    assert_eq!(res.updated_at, board.updated_at);
}

#[tokio::test]
async fn test_update_handle() {
    let data = TestData::new().await;
    let board_persist = data.board();
    let board = data.generate_board().await;

    let update = UpdateBoard {
        handle: Some("test".into()),
        ..Default::default()
    };

    let res = board_persist.update(&board.id.id.to_raw(), update).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    println!("{res:?}");
    assert!(res.is_some());

    let res = res.unwrap();
    println!("{res:?}");
    assert_eq!(res.id, board.id);
    assert_eq!(res.creator_id, board.creator_id);
    assert_eq!(res.handle, "test");
    assert_eq!(res.name, board.name);
    assert_eq!(res.description, board.description);
    assert_ne!(res.updated_at, board.updated_at);
}

#[tokio::test]
async fn test_update_name() {
    let data = TestData::new().await;
    let board_persist = data.board();
    let board = data.generate_board().await;

    let update = UpdateBoard {
        name: MaybeUndefined::Value("Test".into()),
        ..Default::default()
    };

    let res = board_persist.update(&board.id.id.to_raw(), update).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    println!("{res:?}");
    assert!(res.is_some());

    let res = res.unwrap();
    println!("{res:?}");
    assert_eq!(res.id, board.id);
    assert_eq!(res.creator_id, board.creator_id);
    assert_eq!(res.handle, board.handle);
    assert_eq!(res.name, Some("Test".to_owned()));
    assert_eq!(res.description, board.description);
    assert_ne!(res.updated_at, board.updated_at);
}

#[tokio::test]
async fn test_update_name_null() {
    let data = TestData::new().await;
    let board_persist = data.board();
    let board = data.generate_board().await;

    let update = UpdateBoard {
        name: MaybeUndefined::Null,
        ..Default::default()
    };

    let res = board_persist.update(&board.id.id.to_raw(), update).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    println!("{res:?}");
    assert!(res.is_some());

    let res = res.unwrap();
    println!("{res:?}");
    assert_eq!(res.id, board.id);
    assert_eq!(res.creator_id, board.creator_id);
    assert_eq!(res.handle, board.handle);
    assert_eq!(res.name, None);
    assert_eq!(res.description, board.description);
    assert_ne!(res.updated_at, board.updated_at);
}

#[tokio::test]
async fn test_update_description() {
    let data = TestData::new().await;
    let board_persist = data.board();
    let board = data.generate_board().await;

    let update = UpdateBoard {
        description: MaybeUndefined::Value("Test".into()),
        ..Default::default()
    };

    let res = board_persist.update(&board.id.id.to_raw(), update).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    println!("{res:?}");
    assert!(res.is_some());

    let res = res.unwrap();
    println!("{res:?}");
    assert_eq!(res.id, board.id);
    assert_eq!(res.creator_id, board.creator_id);
    assert_eq!(res.handle, board.handle);
    assert_eq!(res.name, board.name);
    assert_eq!(res.description, Some("Test".to_owned()));
    assert_ne!(res.updated_at, board.updated_at);
}

#[tokio::test]
async fn test_update_description_null() {
    let data = TestData::new().await;
    let board_persist = data.board();
    let board = data.generate_board().await;

    let update = UpdateBoard {
        description: MaybeUndefined::Null,
        ..Default::default()
    };

    let res = board_persist.update(&board.id.id.to_raw(), update).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    println!("{res:?}");
    assert!(res.is_some());

    let res = res.unwrap();
    println!("{res:?}");
    assert_eq!(res.id, board.id);
    assert_eq!(res.creator_id, board.creator_id);
    assert_eq!(res.handle, board.handle);
    assert_eq!(res.name, board.name);
    assert_eq!(res.description, None);
    assert_ne!(res.updated_at, board.updated_at);
}

#[tokio::test]
async fn test_update_all() {
    let data = TestData::new().await;
    let board_persist = data.board();
    let board = data.generate_board().await;

    let update = UpdateBoard {
        handle: Some("test".into()),
        name: MaybeUndefined::Value("Test".into()),
        description: MaybeUndefined::Value("Test".into()),
    };

    let res = board_persist.update(&board.id.id.to_raw(), update).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    println!("{res:?}");
    assert!(res.is_some());

    let res = res.unwrap();
    println!("{res:?}");
    assert_eq!(res.id, board.id);
    assert_eq!(res.creator_id, board.creator_id);
    assert_eq!(res.handle, "test");
    assert_eq!(res.name, Some("Test".to_owned()));
    assert_eq!(res.description, Some("Test".to_owned()));
    assert_ne!(res.updated_at, board.updated_at);
}

#[tokio::test]
async fn test_update_nonexistent() {
    let data = TestData::new().await;
    let board_persist = data.board();

    let update = UpdateBoard {
        handle: Some("test".into()),
        name: MaybeUndefined::Value("Test".into()),
        description: MaybeUndefined::Value("Test".into()),
    };

    let res = board_persist.update("test", update).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    println!("{res:?}");
    assert!(res.is_none());

    let res = board_persist.get("test").await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    assert!(res.is_none());
}

#[tokio::test]
async fn test_delete() {
    let data = TestData::new().await;
    let board_persist = data.board();
    let board = data.generate_board().await;

    let res = board_persist.delete(&board.id.id.to_raw()).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    println!("{res:?}");
    assert!(res.is_some());

    let res = res.unwrap();
    assert_eq!(res.id, board.id);
    assert_eq!(res.creator_id, board.creator_id);
    assert_eq!(res.handle, board.handle);
    assert_eq!(res.name, board.name);
    assert_eq!(res.description, board.description);
    assert_eq!(res.updated_at, board.updated_at);

    let res = board_persist.get(&board.id.id.to_raw()).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    assert!(res.is_none());
}

#[tokio::test]
async fn test_delete_nonexistent() {
    let data = TestData::new().await;
    let board_persist = data.board();

    let res = board_persist.delete("test").await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    assert!(res.is_none());

    let res = board_persist.get("test").await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    assert!(res.is_none());
}
