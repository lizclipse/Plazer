use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use async_trait::async_trait;
use c11ity_common::{api, DEBUG_PREFIX};
use const_format::concatcp;
use futures::{
    channel::{mpsc, oneshot},
    select,
    stream::FusedStream,
    Sink, SinkExt, Stream, StreamExt,
};
use gloo_net::websocket::{self, futures::WebSocket};
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen_futures::spawn_local;

use crate::{rand_u64, Account, Client, ClientError, Result};

/// Creates a new client backed by a WASM-based WebSocket.
pub fn client(ws: WebSocket) -> impl Client {
    let (cl, rx) = WsClient::new();
    start(ws, cl.connected.clone(), rx);
    cl
}

/// Creates a new client backed by a WASM-based WebSocket with a given
/// size internal message buffer.
pub fn client_sized(ws: WebSocket, buffer: usize) -> impl Client {
    let (cl, rx) = WsClient::new_sized(buffer);
    start(ws, cl.connected.clone(), rx);
    cl
}

#[derive(Debug)]
struct WsClient {
    inner: ClientInner,
    connected: Arc<AtomicBool>,
}

impl WsClient {
    fn new() -> (Self, mpsc::Receiver<Request>) {
        // A good default for now.
        Self::new_sized(32)
    }

    fn new_sized(buffer: usize) -> (Self, mpsc::Receiver<Request>) {
        let (tx, rx) = mpsc::channel(buffer);
        (
            Self {
                inner: ClientInner { chl: tx },
                connected: Arc::new(AtomicBool::new(true)),
            },
            rx,
        )
    }
}

impl Client for WsClient {
    fn connected(&self) -> bool {
        self.connected.load(Ordering::Acquire)
    }

    type Account<'a> = WsAccount<'a>;
    fn account<'a>(&'a self) -> Self::Account<'a> {
        WsAccount { inner: &self.inner }
    }
}

struct WsAccount<'a> {
    inner: &'a ClientInner,
}

#[async_trait]
impl<'a> Account for WsAccount<'a> {
    async fn login<'b>(&self, req: api::account::LoginReq<'b>) -> Result<api::account::LoginRes> {
        self.inner
            .unary(api::Method::Account(api::account::Method::Login(req)))
            .await
            .and_then(|res| match res {
                api::Response::Account(api::account::Response::Login(res)) => Ok(res),
                _ => Err(ClientError::ResponseMismatch),
            })
    }
}

#[derive(Debug)]
struct ClientInner {
    chl: mpsc::Sender<Request>,
}

const JSON_TRANSPORT: &str = concatcp!(DEBUG_PREFIX, "json-transport");

impl ClientInner {
    async fn unary(&self, req: api::Method<'_>) -> Result<api::Response> {
        let mut chl = self.chl.clone();

        // Prepare the request
        let nonce = rand_u64()?;
        let req: api::Message<api::Method> = api::Message {
            nonce,
            payload: req,
        };

        let req = if LocalStorage::get(JSON_TRANSPORT).unwrap_or(false) {
            websocket::Message::Text(
                serde_json::to_string(&req).map_err(|_| ClientError::InvalidRequest)?,
            )
        } else {
            websocket::Message::Bytes(
                bincode::serialize(&req).map_err(|_| ClientError::InvalidRequest)?,
            )
        };

        // Send it down the channel
        let (tx, rx) = oneshot::channel();
        chl.send(Request::Unary(nonce, req, tx)).await?;

        // Wait and handle response
        rx.await?
    }
}

#[derive(Debug)]
enum Request {
    Unary(u64, websocket::Message, oneshot::Sender<ChannelResponse>),
}

type ChannelData = api::Message<api::Response>;

type ChannelResponse = Result<api::Response>;
type WsMessage = core::result::Result<websocket::Message, websocket::WebSocketError>;

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
struct WsHandle<W> {
    ws: W,
    pending: HashMap<u64, Req>,
    connected: Arc<AtomicBool>,
}

fn start(
    ws: WebSocket,
    connected: Arc<AtomicBool>,
    mut reqs: impl Stream<Item = Request> + FusedStream + Unpin + 'static,
) {
    spawn_local(async move {
        let (write, read) = ws.split();
        let mut read = read.fuse();

        let mut handle = WsHandle::new(write, connected);

        while handle.connected.load(Ordering::Acquire) {
            let item = select! {
                msg = read.next() => msg.map(Item::Msg),
                req = reqs.next() => req.map(Item::Req),
                complete => break,
            };

            if let Some(item) = item {
                match item {
                    Item::Req(req) => handle.process_req(req).await,
                    Item::Msg(msg) => handle.process_msg(msg),
                }
            }
        }
    });
}

impl<W> WsHandle<W>
where
    W: Sink<websocket::Message, Error = websocket::WebSocketError> + Unpin,
{
    fn new(ws: W, connected: Arc<AtomicBool>) -> Self {
        Self {
            ws,
            pending: HashMap::new(),
            connected,
        }
    }

    async fn process_req(&mut self, req: Request) {
        match req {
            Request::Unary(nonce, data, tx) => {
                match self.ws.send(data).await {
                    Ok(_) => {
                        self.pending.insert(nonce, Req::Unary(tx));
                    }
                    Err(err) => {
                        // Nothing we can do about a failed send.
                        // If it fails it means the receiver was dropped anyway, so it doesn't
                        // actually matter.
                        let _ = tx.send(Err(err.into()));
                    }
                }
            }
        }
    }

    fn process_msg(&mut self, msg: WsMessage) {
        let msg = match msg {
            Ok(msg) => msg,
            Err(err) => {
                log::warn!("WebSocket closed {:?}", err);
                self.connected.store(false, Ordering::Release);
                return;
            }
        };

        let msg: ChannelData = match msg {
            websocket::Message::Text(data) => match serde_json::from_str(&data) {
                Ok(msg) => msg,
                Err(err) => {
                    log::error!("Unable to parse message {:?}", err);
                    return;
                }
            },
            websocket::Message::Bytes(data) => match bincode::deserialize(&data) {
                Ok(msg) => msg,
                Err(err) => {
                    log::error!("Unable to deserialise message {:?}", err);
                    return;
                }
            },
        };

        let nonce = msg.nonce;
        let req = match self.pending.remove(&nonce) {
            Some(req) => req,
            None => {
                log::error!("Received unknown message ID {}", nonce);
                return;
            }
        };

        match req {
            Req::Unary(tx) => {
                // Nothing we can do about a failed send
                let _ = tx.send(Ok(msg.payload));
            }
        };
    }
}
