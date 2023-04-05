use std::{env, pin::Pin, sync::Arc};

use futures::{FutureExt as _, Stream};
use juniper::{graphql_object, graphql_subscription, RootNode};
use juniper_graphql_ws::ConnectionConfig;
use juniper_warp::{playground_filter, subscriptions::serve_graphql_ws};
use tokio::sync::{watch, RwLock};
use tracing::Level;
use warp::{http::Response, Filter};

struct ContextInner {
    counter: i32,
    counter_rx: watch::Receiver<i32>,
    counter_tx: watch::Sender<i32>,
}

struct Context(RwLock<ContextInner>);

impl Context {
    fn new() -> Self {
        let (counter_tx, counter_rx) = watch::channel(0);
        Self(RwLock::new(ContextInner {
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

    pub async fn counter_rx(&self) -> impl Stream<Item = i32> + Send + Sync + 'static {
        let mut rx = self.0.read().await.counter_rx.clone();
        async_stream::stream! {
            while rx.changed().await.is_ok() {
                let counter = {
                    let value = rx.borrow();
                    value.clone()
                };
                yield counter;
            }
        }
    }
}

impl juniper::Context for Context {}

struct Query;

#[graphql_object(context = Context)]
impl Query {
    async fn count<'db>(context: &'db Context) -> i32 {
        context.counter().await
    }
}

struct Mutation;

#[graphql_object(context = Context)]
impl Mutation {
    async fn increment<'db>(by: Option<i32>, context: &'db Context) -> i32 {
        context.increment(by).await
    }
}

type ChangesStream = Pin<Box<dyn Stream<Item = i32> + Send>>;

struct Subscription;

#[graphql_subscription(context = Context)]
impl Subscription {
    async fn changes<'db>(context: &'db Context) -> ChangesStream {
        let changes = context.counter_rx().await;

        Box::pin(changes)
    }
}

type Schema = RootNode<'static, Query, Mutation, Subscription>;

fn schema() -> Schema {
    Schema::new(Query, Mutation, Subscription)
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "warp_subscriptions");
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let log = warp::log("warp_subscriptions");

    let homepage = warp::path::end().map(|| {
        Response::builder()
            .header("content-type", "text/html")
            .body("<html><h1>juniper_subscriptions demo</h1><div>visit <a href=\"/playground\">graphql playground</a></html>")
    });

    let qm_schema = schema();
    let qm_state = warp::any().map(|| Context::new());
    let qm_graphql_filter = juniper_warp::make_graphql_filter(qm_schema, qm_state.boxed());

    let root_node = Arc::new(schema());

    log::info!("Listening on 127.0.0.1:8080");

    let routes = (warp::path!("api" / "subscriptions")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let root_node = root_node.clone();
            ws.on_upgrade(move |websocket| async move {
                serve_graphql_ws(websocket, root_node, ConnectionConfig::new(Context::new()))
                    .map(|r| {
                        if let Err(e) = r {
                            println!("Websocket error: {e}");
                        }
                    })
                    .await
            })
        }))
    .map(|reply| {
        // TODO#584: remove this workaround
        warp::reply::with_header(reply, "Sec-WebSocket-Protocol", "graphql-ws")
    })
    .or(warp::post()
        .and(warp::path!("api" / "graphql"))
        .and(qm_graphql_filter))
    .or(warp::get()
        .and(warp::path("playground"))
        .and(playground_filter("/api/graphql", Some("/api/subscriptions"))))
    .or(homepage)
    .with(log);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
