pub mod account;

use derive_more::From;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<T> {
    pub nonce: u64,
    pub payload: T,
}

#[derive(Debug, From, Serialize, Deserialize)]
pub enum Method<'a> {
    #[serde(borrow)]
    Account(account::Method<'a>),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_message_transcode() -> Result<(), bincode::Error> {
        let a = Message {
            nonce: 1,
            payload: vec![4u8, 3, 2, 1],
        };

        let data = bincode::serialize(&a)?;
        let b: Message<&[u8]> = bincode::deserialize(&data)?;

        assert_eq!(a.nonce, b.nonce);
        assert_eq!(a.payload, b.payload);

        Ok(())
    }
}
