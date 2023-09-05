mod pagination;
pub mod srql;
mod value;

pub use pagination::*;
pub use value::*;

pub trait CreateObject {
    fn append(self, expr: &mut srql::SetExpr);
}

pub trait IntoUpdateQuery {
    fn into_update(self, thing: srql::Thing) -> Option<srql::Query>;
}
