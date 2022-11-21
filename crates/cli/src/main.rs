mod api;

use std::net::SocketAddr;

use api::Db;
use axum::{
    extract::ws::{self, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
    Extension, Router,
};
use c11ity_common::api::{Message, Method};

#[tokio::main]
async fn main() {
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

async fn handle_socket(mut socket: WebSocket, db: Db) {
    let db = db.client();
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // Client disconnected
            return;
        };

        let msg = match msg {
            ws::Message::Binary(msg) => msg,
            msg => {
                tracing::warn!("Unhandled message type {:?}", msg);
                continue;
            }
        };

        let Message { nonce, payload }: Message<Method> = match bincode::deserialize(&msg) {
            Ok(msg) => msg,
            Err(err) => {
                tracing::warn!("Failed to parse message {:?}", err);
                continue;
            }
        };

        let res = match db.dispatch(nonce, payload).await {
            Ok(res) => res,
            Err(err) => {
                tracing::warn!("Failed to encode message {:?}", err);
                continue;
            }
        };

        if socket.send(ws::Message::Binary(res)).await.is_err() {
            // Client disconnected
            return;
        }
    }
}
