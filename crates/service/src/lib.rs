use std::future;

use async_graphql::{
    http::{GraphiQLSource, ALL_WEBSOCKET_PROTOCOLS},
    *,
};
use async_graphql_axum::{GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLWebSocket};
use axum::{
    extract::{State, WebSocketUpgrade},
    response::{self, IntoResponse},
    routing::{get, post},
    Router, Server,
};
use futures::Stream;
use tokio::sync::{watch, RwLock};
use tracing::Level;

struct DbInner {
    counter: i32,
    counter_rx: watch::Receiver<i32>,
    counter_tx: watch::Sender<i32>,
}

pub struct Db(RwLock<DbInner>);

impl Db {
    fn new() -> Self {
        let (counter_tx, counter_rx) = watch::channel(0);
        Self(RwLock::new(DbInner {
            counter: 0,
            counter_rx,
            counter_tx,
        }))
    }

    pub async fn increment(&self, by: Option<i32>) -> i32 {
        let mut inner = self.0.write().await;
        inner.counter += by.unwrap_or(1);
        let _ = inner.counter_tx.send(inner.counter);
        inner.counter
    }

    pub async fn counter(&self) -> i32 {
        self.0.read().await.counter
    }

    pub async fn counter_rx(&self) -> impl Stream<Item = i32> {
        let mut rx = self.0.read().await.counter_rx.clone();
        async_stream::stream! {
            while rx.changed().await.is_ok() {
                let counter = {
                    let value = rx.borrow();
                    *value
                };
                yield counter;
            }
        }
    }
}

pub struct Query;

#[Object]
impl Query {
    async fn count<'ctx>(&self, ctx: &Context<'ctx>) -> i32 {
        let db = ctx.data::<Db>().unwrap();
        db.counter().await
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn increment<'ctx>(&self, ctx: &Context<'ctx>, by: Option<i32>) -> i32 {
        let db = ctx.data::<Db>().unwrap();
        db.increment(by).await
    }
}

pub struct Subscription;

#[Subscription]
impl Subscription {
    async fn changes<'ctx>(&self, ctx: &Context<'ctx>) -> impl Stream<Item = i32> {
        let db = ctx.data::<Db>().unwrap();
        db.counter_rx().await
    }
}

pub fn schema() -> Schema<Query, Mutation, Subscription> {
    Schema::build(Query, Mutation, Subscription).finish()
}

async fn graphql_handler(
    schema: State<Schema<Query, Mutation, Subscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema
        .execute(req.into_inner().data(Db::new()))
        .await
        .into()
}

async fn graphql_ws_handler(
    schema: State<Schema<Query, Mutation, Subscription>>,
    protocol: GraphQLProtocol,
    upgrade: WebSocketUpgrade,
) -> impl IntoResponse {
    let schema = (*schema).clone();
    upgrade
        .protocols(ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| {
            GraphQLWebSocket::new(stream, schema, protocol)
                .on_connection_init(|_| {
                    future::ready(Ok({
                        let mut data = Data::default();
                        data.insert(Db::new());
                        data
                    }))
                })
                .serve()
        })
}

async fn graphiql() -> impl IntoResponse {
    response::Html(
        GraphiQLSource::build()
            .endpoint("/api/graphql")
            .subscription_endpoint("/api/graphql/ws")
            .finish(),
    )
}

pub async fn serve() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let schema = schema();

    let app = Router::new()
        .route("/", get(graphiql))
        .route("/api/graphql", post(graphql_handler))
        .route("/api/graphql/ws", get(graphql_ws_handler))
        .with_state(schema);

    println!("GraphiQL IDE: http://localhost:8080");

    Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
