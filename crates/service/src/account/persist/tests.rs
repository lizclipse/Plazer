use super::*;
use crate::account::{create_refresh_token, testing::*};

#[tokio::test]
async fn test_create() {
    let data = TestData::new().await;
    let acc_persist = data.account();

    let acc = CreateAccount {
        user_id: "test".into(),
        pword: "test".to_owned().into(),
        invite: None,
    };

    let res = acc_persist.create(acc).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    assert_eq!(res.account.user_id, "test");
}

#[tokio::test]
async fn test_get() {
    let data = TestData::new().await;
    let acc_persist = data.account();
    let AccData { user_id, acc, .. } = acc_persist.create_test_user().await;

    let res = acc_persist.get(&acc.id.into_gql_id()).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    assert!(res.is_some());

    let res = res.unwrap();
    assert_eq!(res.user_id, user_id);
}

#[tokio::test]
async fn test_get_user_id() {
    let data = TestData::new().await;
    let acc_persist = data.account();
    let AccData { user_id, acc, .. } = acc_persist.create_test_user().await;

    let res = acc_persist.get_by_user_id(&user_id).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    assert!(res.is_some());

    let res = res.unwrap();
    assert_eq!(res.id, acc.id);
}

#[tokio::test]
async fn test_duplicate_user_id() {
    let data = TestData::new().await;
    let acc_persist = data.account();
    let AccData { user_id, .. } = acc_persist.create_test_user().await;

    let acc = CreateAccount {
        user_id,
        pword: "test2".to_owned().into(),
        invite: None,
    };

    let res = acc_persist.create(acc).await;
    println!("{res:?}");
    assert!(res.is_err());

    let res = res.unwrap_err();
    assert_eq!(res, Error::UnavailableIdent);
}

#[tokio::test]
async fn test_login() {
    let data = TestData::new().await;
    let acc_persist = data.account();
    let AccData { user_id, pword, .. } = acc_persist.create_test_user().await;

    let res = acc_persist
        .login(AuthCreds {
            user_id: user_id.clone(),
            pword,
        })
        .await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    assert_eq!(res.account.user_id, user_id);
}

#[tokio::test]
async fn test_login_fail() {
    let data = TestData::new().await;
    let acc_persist = data.account();
    let AccData { user_id, .. } = acc_persist.create_test_user().await;

    let res = acc_persist
        .login(AuthCreds {
            user_id,
            pword: "bad password".to_owned().into(),
        })
        .await;
    println!("{res:?}");
    assert!(res.is_err());

    let res = res.unwrap_err();
    assert_eq!(res, Error::CredentialsInvalid);
}

#[tokio::test]
async fn test_refresh() {
    let data = TestData::new().await;
    let acc_persist = data.account();
    let AccData { user_id, acc, .. } = acc_persist.create_test_user().await;
    let refresh_token = create_refresh_token(acc.id.to_gql_id(), &data.jwt_enc_key).unwrap();

    let res = acc_persist.refresh(refresh_token).await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = res.unwrap();
    assert_eq!(res.account.id, acc.id);
    assert_eq!(res.account.user_id, user_id);
}

#[tokio::test]
async fn test_access_token_fail() {
    let data = TestData::new().await;
    let acc_persist = data.account();

    let res = acc_persist.refresh("invalid.refresh.token".into()).await;
    println!("{res:?}");
    assert!(res.is_err());

    let res = res.unwrap_err();
    assert_eq!(res, Error::CredentialsInvalid);
}

#[tokio::test]
async fn test_revoke_tokens() {
    let (data, AccData { acc, .. }) = TestData::with_user().await;
    let acc_persist = data.account();
    let refresh_token = create_refresh_token(acc.id.into_gql_id(), &data.jwt_enc_key).unwrap();

    let res = acc_persist.revoke_tokens().await;
    println!("{res:?}");
    assert!(res.is_ok());

    let res = acc_persist.refresh(refresh_token).await;
    println!("{res:?}");
    assert!(res.is_err());

    let res = res.unwrap_err();
    assert_eq!(res, Error::CredentialsInvalid);
}

#[tokio::test]
async fn test_revoke_tokens_fail() {
    let data = TestData::new().await;
    let acc_persist = data.account();

    let res = acc_persist.revoke_tokens().await;
    println!("{res:?}");
    assert!(res.is_err());

    let res = res.unwrap_err();
    assert_eq!(res, Error::Unauthenticated);
}
