use std::rc::Rc;

use serde::{Deserialize, Serialize};
use sycamore::reactive::{RcSignal, ReadSignal};

#[derive(Debug, Clone)]
pub struct AccountApi(Rc<AccountInner>);

impl AccountApi {
    pub fn id(&self) -> &ReadSignal<Option<String>> {
        &self.0.id
    }

    pub fn name(&self) -> &ReadSignal<Option<String>> {
        &self.0.id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AccountInner {
    id: RcSignal<Option<String>>,
    name: RcSignal<Option<String>>,
}

impl Serialize for AccountApi {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AccountApi {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(AccountApi(Rc::new(AccountInner::deserialize(
            deserializer,
        )?)))
    }
}
