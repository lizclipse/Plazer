use c11ity_common::api::account::{LoginReq, LoginRes, Method};

use crate::{unary, ClientInner};

pub struct Account<'a> {
    client: &'a ClientInner,
}

impl<'a> Account<'a> {
    pub(crate) fn new(client: &'a ClientInner) -> Account<'a> {
        Self { client }
    }

    unary!(login, Account, Login, LoginReq, LoginRes);
}
