mod migration;
mod models;
mod persist;
mod schema;

pub use migration::*;
pub use models::*;
pub use persist::*;
pub use schema::*;

pub static BOARD_TABLE_NAME: &str = "board";
