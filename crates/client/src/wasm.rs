use std::collections::HashMap;

use futures::{channel::oneshot, select, stream::FusedStream, Sink, SinkExt, Stream, StreamExt};
use gloo_net::websocket::{self, futures::WebSocket};
use wasm_bindgen_futures::spawn_local;

use crate::{Client, RawResponse, Request};

/// Creates a new client backed by a WASM-based WebSocket.
pub fn client(ws: WebSocket) -> Client {
    let (cl, rx) = Client::new();
    WsClient::start(ws, rx);
    cl
}

/// Creates a new client backed by a WASM-based WebSocket with a given
/// size internal message buffer.
pub fn client_sized(ws: WebSocket, buffer: usize) -> Client {
    let (cl, rx) = Client::new_sized(buffer);
    WsClient::start(ws, rx);
    cl
}

type WsMessage = Result<websocket::Message, websocket::WebSocketError>;

#[derive(Debug)]
enum Item {
    Req(Request),
    Msg(WsMessage),
}

#[derive(Debug)]
enum Req {
    Unary(oneshot::Sender<RawResponse>),
}

#[derive(Debug, Default)]
struct WsClient {
    pending: HashMap<u64, Req>,
}

impl WsClient {
    fn start(ws: WebSocket, mut reqs: impl Stream<Item = Request> + FusedStream + Unpin + 'static) {
        spawn_local(async move {
            let (write, read) = ws.split();
            let mut read = read.fuse();

            let mut client = Self::default();

            loop {
                let item = select! {
                    msg = read.next() => msg.map(Item::Msg),
                    req = reqs.next() => req.map(Item::Req),
                    complete => break,
                };

                if let Some(item) = item {
                    match item {
                        Item::Req(req) => client.process_req(req).await,
                        Item::Msg(msg) => client.process_msg(msg).await,
                    }
                }
            }
        });
    }

    async fn process_req(&mut self, req: Request) {
        todo!()
    }

    async fn process_msg(&mut self, msg: WsMessage) {
        todo!()
    }
}
