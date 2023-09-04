pub use surrealdb::sql::{statements::*, *};

pub fn values_table(table: impl Into<String>) -> Values {
    Values(vec![Table(table.into()).into()])
}

pub fn field(field: impl Into<String>) -> Idiom {
    Idiom(vec![Part::Field(Ident(field.into()))])
}

// pub fn param(param: impl Into<String>) -> srql::Param {
//     srql::Param(srql::Ident(param.into()))
// }

// pub fn string(str: impl Into<String>) -> srql::Strand {
//     srql::Strand(str.into())
// }
