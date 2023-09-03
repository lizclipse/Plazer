mod auth;
mod migration;
mod models;
mod persist;
mod schema;

pub use auth::*;
pub use migration::*;
pub use models::*;
pub use persist::*;
pub use schema::*;

static TABLE_NAME: &str = "account";
