mod migration;
mod models;
mod persist;
mod schema;

pub use migration::*;
pub use models::*;
pub use persist::*;
pub use schema::*;

static TABLE_NAME: &str = "board";
