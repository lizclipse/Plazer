mod db;

use std::net::SocketAddr;

use axum::{
    extract::{
        ws::{self, WebSocket, WebSocketUpgrade},
        RawQuery, State,
    },
    http::{StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router, debug_handler,
};
use c11ity_common::api::Message;
use c11ity_ui::render;
use db::Db;
use tracing::{instrument, Level};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let db = Db::new();

    let app = Router::new()
        .route("/api/v1/rpc", get(rpc_handle))
        .fallback(ui_render)
        .with_state(db);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn rpc_handle(ws: WebSocketUpgrade, State(db): State<Db>) -> Response {
    ws.on_upgrade(|socket| rpc(socket, db))
}

#[debug_handler]
async fn ui_render() -> impl IntoResponse {
    // Html(render(path.unwrap_or_default()).await)
    // let ui = render("".to_owned()).await;
    Html("")
}

enum Transport {
    Binary,
    Text,
}

#[instrument(skip(socket))]
async fn rpc(mut socket: WebSocket, db: Db) {
    let db = db.client();
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // Client disconnected
            return;
        };

        let (Message { nonce, payload }, transport) = match &msg {
            ws::Message::Binary(data) => match bincode::deserialize(data) {
                Ok(msg) => (msg, Transport::Binary),
                Err(err) => {
                    tracing::warn!("Failed to decode message {:?}", err);
                    continue;
                }
            },
            ws::Message::Text(data) => match serde_json::from_str(data) {
                Ok(msg) => (msg, Transport::Text),
                Err(err) => {
                    tracing::warn!("Failed to parse message {:?}", err);
                    continue;
                }
            },
            msg => {
                tracing::warn!("Unhandled message type {:?}", msg);
                continue;
            }
        };

        let res = Message {
            nonce,
            payload: db.dispatch(payload).await,
        };

        let res = match transport {
            Transport::Binary => match bincode::serialize(&res) {
                Ok(res) => socket.send(ws::Message::Binary(res)).await,
                Err(err) => {
                    tracing::warn!("Failed to encode message {:?}", err);
                    continue;
                }
            },
            Transport::Text => match serde_json::to_string(&res) {
                Ok(res) => socket.send(ws::Message::Text(res)).await,
                Err(err) => {
                    tracing::warn!("Failed to encode message {:?}", err);
                    continue;
                }
            },
        };

        if res.is_err() {
            // Client disconnected
            return;
        }
    }
}
