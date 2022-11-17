use std::{collections::HashMap, mem};

use async_trait::async_trait;
use c11ity_common::{api, Container};
use futures::{
    channel::{mpsc, oneshot},
    select,
    stream::FusedStream,
    Sink, SinkExt, Stream, StreamExt,
};
use gloo_net::websocket::{self, futures::WebSocket};
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;

use crate::{rand_u64, Account, Client, ClientError, Result};

/// Creates a new client backed by a WASM-based WebSocket.
pub fn client(ws: WebSocket) -> impl Client {
    let (cl, rx) = WsClient::new();
    start(ws, rx);
    cl
}

/// Creates a new client backed by a WASM-based WebSocket with a given
/// size internal message buffer.
pub fn client_sized(ws: WebSocket, buffer: usize) -> impl Client {
    let (cl, rx) = WsClient::new_sized(buffer);
    start(ws, rx);
    cl
}

#[derive(Debug)]
struct WsClient {
    inner: ClientInner,
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
            },
            rx,
        )
    }
}

impl Client for WsClient {
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
    type LoginRes = RequestOutput<api::account::LoginRes<'static>>;
    async fn login<'b>(&self, req: api::account::LoginReq<'b>) -> Result<Self::LoginRes> {
        self.inner
            .unary(api::Method::Account(api::account::Method::Login(req)))
            .await
    }
}

#[derive(Debug)]
struct ClientInner {
    chl: mpsc::Sender<Request>,
}

impl ClientInner {
    async fn unary<'a, Res>(&self, req: api::Method<'a>) -> Result<RequestOutput<Res>>
    where
        Res: Deserialize<'static>,
    {
        let mut chl = self.chl.clone();

        // Prepare the request
        let nonce = rand_u64()?;
        let req: api::Message<api::Method> = api::Message {
            nonce,
            payload: req,
        };
        let req = bincode::serialize(&req).map_err(ClientError::InvalidRequest)?;

        // Send it down the channel
        let (tx, rx) = oneshot::channel();
        chl.send(Request::Unary(nonce, req, tx)).await?;

        // Wait and handle response
        let data = rx.await??;
        let res = bincode::deserialize(data.payload).map_err(ClientError::InvalidResponse)?;
        Ok((data, res).into())
    }
}

#[derive(Debug)]
pub enum Request {
    Unary(u64, Vec<u8>, oneshot::Sender<ChannelResponse>),
}

type ChannelData = Container<Vec<u8>, api::Message<&'static [u8]>>;
type ChannelResponse = Result<ChannelData>;
type RequestOutput<T> = Container<ChannelData, T>;
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
}

fn start(ws: WebSocket, mut reqs: impl Stream<Item = Request> + FusedStream + Unpin + 'static) {
    spawn_local(async move {
        let (write, read) = ws.split();
        let mut read = read.fuse();

        let mut handle = WsHandle::new(write);

        loop {
            let item = select! {
                msg = read.next() => msg.map(Item::Msg),
                req = reqs.next() => req.map(Item::Req),
                complete => break,
            };

            if let Some(item) = item {
                match item {
                    Item::Req(req) => handle.process_req(req).await,
                    Item::Msg(msg) => handle.process_msg(msg).await,
                }
            }
        }
    });
}

impl<W> WsHandle<W>
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
            Request::Unary(nonce, data, tx) => {
                match self.ws.send(websocket::Message::Bytes(data)).await {
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

    async fn process_msg(&mut self, msg: WsMessage) {
        todo!()
    }
}

impl<W> Drop for WsHandle<W> {
    fn drop(&mut self) {
        // We need to own the senders, so just replace in an empty map.
        for (_, req) in mem::replace(&mut self.pending, HashMap::new()) {
            match req {
                Req::Unary(tx) => {
                    // Again, nothing we can do if this fails.
                    let _ = tx.send(Err(ClientError::Closed));
                }
            }
        }
    }
}
