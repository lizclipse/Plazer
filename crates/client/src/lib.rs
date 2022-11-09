mod account;
#[cfg(feature = "wasm")]
pub mod wasm;

use std::future::Future;
use std::marker::PhantomData;
use std::mem;
use std::pin::Pin;
use std::task::{ready, Poll};
use std::{fmt::Display, task::Context};

use account::Account;
use c11ity_common::{api, Container};
use futures::channel::{mpsc, oneshot};
use serde::{Deserialize, Serialize};
use thiserror::Error;

type Result<T> = core::result::Result<T, ClientError>;

#[derive(Debug)]
pub struct Client {
    inner: ClientInner,
}

impl<'a> Client {
    pub fn new() -> (Self, mpsc::Receiver<Request>) {
        // A good default for now.
        Self::new_sized(32)
    }

    pub fn new_sized(buffer: usize) -> (Self, mpsc::Receiver<Request>) {
        let (tx, rx) = mpsc::channel(buffer);
        (
            Self {
                inner: ClientInner { chl: tx },
            },
            rx,
        )
    }

    pub fn account(&self) -> Account {
        Account::new(&self.inner)
    }
}

#[macro_export]
macro_rules! unary {
    ($name: ident, $method: ident, $internal_method: ident, $input: ident, $output: ident) => {
        pub fn $name<'input>(
            &self,
            req: $input<'input>,
        ) -> impl std::future::Future<
            Output = $crate::Result<c11ity_common::Container<$crate::RawResponse, $output>>,
        > + 'input {
            self.client.unary(
                c11ity_common::api::Method::$method(Method::$internal_method(req)),
                $crate::transform,
            )
        }
    };
}

fn transform<Res>(data: RawResponse) -> bincode::Result<Container<RawResponse, Res>>
where
    Res: Deserialize<'static>,
{
    bincode::deserialize(data.payload).map(|value| (data, value).into())
}

#[derive(Debug)]
struct ClientInner {
    chl: mpsc::Sender<Request>,
}

impl ClientInner {
    fn unary<Req, Res, Trans>(&self, inp: Req, trans: Trans) -> Unary<Req, Res, Trans>
    where
        Req: Serialize + Unpin,
        Res: Unpin,
        Trans: FnOnce(RawResponse) -> bincode::Result<Res>,
    {
        Unary::new(self.chl.clone(), inp, trans)
    }
}

#[derive(Debug)]
pub enum Request {
    Unary(Vec<u8>, oneshot::Sender<RawResponse>),
}

type RawResponse = Container<Vec<u8>, api::Message<'static>>;

#[derive(Debug, Error)]
pub enum ClientError {
    Closed,
    InvalidRequest(bincode::Error),
    InvalidResponse(bincode::Error),
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Closed => write!(f, "channel closed"),
            ClientError::InvalidRequest(_) => write!(f, "given an invalid request"),
            ClientError::InvalidResponse(_) => write!(f, "received an invalid response"),
        }
    }
}

#[derive(Debug)]
pub struct Unary<Req, Res, Trans> {
    state: UnaryState<Req, Trans>,
    _res: PhantomData<Res>,
}

impl<Req, Res, Trans> Unary<Req, Res, Trans> {
    fn new(chl: mpsc::Sender<Request>, req: Req, trans: Trans) -> Self {
        Self {
            state: UnaryState::Initial { chl, req, trans },
            _res: Default::default(),
        }
    }
}

impl<Req, Res, Trans> Future for Unary<Req, Res, Trans>
where
    Req: Serialize + Unpin,
    Res: Unpin,
    Trans: (FnOnce(RawResponse) -> bincode::Result<Res>) + Unpin,
{
    type Output = Result<Res>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let output = loop {
            match &mut self.state {
                UnaryState::Initial { req, .. } => {
                    let req = match bincode::serialize(&req) {
                        Ok(req) => req,
                        Err(err) => break Poll::Ready(Err(ClientError::InvalidRequest(err))),
                    };

                    let (tx, rx) = oneshot::channel();

                    // This is a workaround to the fact we don't actually own self here,
                    // so we instead move the memory around so we can own the fields.
                    self.state = match std::mem::replace(&mut self.state, UnaryState::Ended) {
                        UnaryState::Initial { chl, trans, .. } => UnaryState::Sending {
                            chl,
                            req,
                            tx,
                            rx,
                            trans,
                        },
                        _ => unreachable!(),
                    };
                }
                UnaryState::Sending { .. } => {
                    // Sending requires owned data, so pull out now and put back in later.
                    let (mut chl, req, tx, rx, trans) =
                        match mem::replace(&mut self.state, UnaryState::Ended) {
                            UnaryState::Sending {
                                chl,
                                req,
                                tx,
                                rx,
                                trans,
                            } => (chl, req, tx, rx, trans),
                            _ => unreachable!(),
                        };

                    match chl.try_send(Request::Unary(req, tx)) {
                        Ok(_) => {
                            self.state = UnaryState::Receiving { rx, trans };
                        }
                        Err(err) => match err.is_full() {
                            true => {
                                let (req, tx) = match err.into_inner() {
                                    Request::Unary(req, tx) => (req, tx),
                                };

                                // We need to do this to queue up the waker, but might as well
                                // handle the result when the state is back to normal.
                                let poll = chl.poll_ready(cx);
                                self.state = UnaryState::Sending {
                                    chl,
                                    req,
                                    tx,
                                    rx,
                                    trans,
                                };
                                match ready!(poll) {
                                    // If it's ready, we're already set up to try again, so let it loop.
                                    Ok(_) => (),
                                    // poll_ready only errs on receiver drop.
                                    Err(_) => break Poll::Ready(Err(ClientError::Closed)),
                                }
                            }
                            false => break Poll::Ready(Err(ClientError::Closed)),
                        },
                    };
                }
                UnaryState::Receiving { rx, .. } => {
                    let data = match ready!(Pin::new(rx).poll(cx)) {
                        Ok(data) => data,
                        Err(_) => break Poll::Ready(Err(ClientError::Closed)),
                    };

                    let trans = match mem::replace(&mut self.state, UnaryState::Ended) {
                        UnaryState::Receiving { trans, .. } => trans,
                        _ => unreachable!(),
                    };

                    let res = trans(data);
                    break match res {
                        Ok(res) => Poll::Ready(Ok(res)),
                        Err(err) => Poll::Ready(Err(ClientError::InvalidResponse(err))),
                    };
                }
                UnaryState::Ended => panic!("unary future cannot be called when done"),
            };
        };

        if output.is_ready() {
            self.state = UnaryState::Ended;
        }
        output
    }
}

#[derive(Debug)]
enum UnaryState<Req, Trans> {
    Initial {
        chl: mpsc::Sender<Request>,
        req: Req,
        trans: Trans,
    },
    Sending {
        chl: mpsc::Sender<Request>,
        req: Vec<u8>,
        tx: oneshot::Sender<RawResponse>,
        rx: oneshot::Receiver<RawResponse>,
        trans: Trans,
    },
    Receiving {
        rx: oneshot::Receiver<RawResponse>,
        trans: Trans,
    },
    Ended,
}
