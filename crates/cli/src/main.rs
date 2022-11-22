mod db;

use std::net::SocketAddr;

use axum::{
    extract::ws::{self, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
    Extension, Router,
};
use c11ity_common::api::{Message, Method};
use db::Db;
use tracing::{instrument, Level};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let db = Db::new();

    let app = Router::new()
        .route("/api/v1/rpc", get(handler))
        .layer(Extension(db));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(ws: WebSocketUpgrade, Extension(db): Extension<Db>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, db))
}

enum Transport {
    Binary,
    Text,
}

#[instrument(skip(socket))]
async fn handle_socket(mut socket: WebSocket, db: Db) {
    let db = db.client();
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // Client disconnected
            return;
        };

        let (Message { nonce, payload }, transport): (Message<Method>, Transport) = match &msg {
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

        let res = db.dispatch(nonce, payload).await;

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
