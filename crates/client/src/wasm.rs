use std::collections::HashMap;

use c11ity_common::api::Message;
use futures::{channel::oneshot, select, stream::FusedStream, Sink, SinkExt, Stream, StreamExt};
use getrandom::getrandom;
use gloo_net::websocket::{self, futures::WebSocket};
use wasm_bindgen_futures::spawn_local;

use crate::{ChannelResponse, Client, ClientError, Request};

/// Creates a new client backed by a WASM-based WebSocket.
pub fn client(ws: WebSocket) -> Client {
    let (cl, rx) = Client::new();
    start(ws, rx);
    cl
}

/// Creates a new client backed by a WASM-based WebSocket with a given
/// size internal message buffer.
pub fn client_sized(ws: WebSocket, buffer: usize) -> Client {
    let (cl, rx) = Client::new_sized(buffer);
    start(ws, rx);
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
    Unary(oneshot::Sender<ChannelResponse>),
}

#[derive(Debug)]
struct WsClient<W> {
    ws: W,
    pending: HashMap<u64, Req>,
}

fn start(ws: WebSocket, mut reqs: impl Stream<Item = Request> + FusedStream + Unpin + 'static) {
    spawn_local(async move {
        let (write, read) = ws.split();
        let mut read = read.fuse();

        let mut client = WsClient::new(write);

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

impl<W> WsClient<W>
where
    W: Sink<websocket::Message, Error = websocket::WebSocketError> + Unpin,
{
    fn new(ws: W) -> Self {
        Self {
            ws,
            pending: HashMap::new(),
        }
    }

    async fn process_req(&mut self, req: Request) {
        match req {
            Request::Unary(data, tx) => match self.req_unary(data).await {
                Ok(nonce) => {
                    self.pending.insert(nonce, Req::Unary(tx));
                }
                Err(err) => {
                    let _ = tx.send(Err(err));
                }
            },
        }
    }

    async fn req_unary(&mut self, data: Vec<u8>) -> crate::Result<u64> {
        let nonce = rand_u64()?;
        

        Ok(0)
    }

    async fn process_msg(&mut self, msg: WsMessage) {
        todo!()
    }
}

fn rand_u64() -> Result<u64, getrandom::Error> {
    let mut buf = [0u8; 8];
    getrandom(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}
