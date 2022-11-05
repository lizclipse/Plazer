use c11ity_common::api::account::{LoginReq, LoginRes, Method};
use serde::Deserialize;

use crate::{ClientInner, Result};

pub struct Account<'a> {
    client: &'a mut ClientInner,
}

impl<'a> Account<'a> {
    pub(crate) fn new(client: &'a mut ClientInner) -> Account<'a> {
        Self { client }
    }

    pub async fn login(&mut self, req: LoginReq) -> Result<LoginRes> {
        self.call(req.into()).await
    }

    async fn call<'de, T>(&mut self, req: Method) -> Result<T>
    where
        T: Deserialize<'de>,
    {
        self.client.call(req.into()).await
    }
}
