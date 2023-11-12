mod models;
mod persist;
mod schema;

pub use models::*;
pub use persist::*;
pub use schema::*;

static POST_TABLE_NAME: &str = "post";
static CONTAINS_TABLE_NAME: &str = "contains_post";
