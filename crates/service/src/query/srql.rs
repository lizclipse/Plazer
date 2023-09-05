use std::collections::BTreeMap;

pub use surrealdb::sql::{statements::*, *};
use ulid::Ulid;

#[inline]
pub fn query(statements: impl Into<Vec<Statement>>) -> Query {
    Query(Statements(statements.into()))
}

#[inline]
pub fn thing(thing: impl Into<Thing>) -> Values {
    Values(vec![thing.into().into()])
}

#[inline]
pub fn table(table: impl Into<String>) -> Values {
    Values(vec![Table(table.into()).into()])
}

#[inline]
pub fn field(field: impl Into<String>) -> Idiom {
    Idiom(vec![Part::Field(Ident(field.into()))])
}

#[inline]
pub fn array(array: impl Into<Vec<Value>>) -> Value {
    Value::Array(array.into().into())
}

#[inline]
pub fn object(obj: impl Into<BTreeMap<String, Value>>) -> Value {
    Value::Object(obj.into().into())
}

#[inline]
pub fn string(str: impl Into<String>) -> Strand {
    Strand(str.into())
}

pub type SetExpr = Vec<(Idiom, Operator, Value)>;

pub fn obj_create_query(table: &str, mut data: SetExpr) -> Query {
    data.push((field("updated_at"), Operator::Equal, time_now()));

    let thing = Thing {
        tb: table.into(),
        id: Ulid::new().to_string().into(),
    };

    query([Statement::Create(CreateStatement {
        what: Values(vec![thing.into()]),
        data: Data::SetExpression(data).into(),
        output: Output::After.into(),
        ..Default::default()
    })])
}

pub fn obj_update_query(thing: Thing, mut update: SetExpr) -> Option<Query> {
    if update.is_empty() {
        return None;
    }

    update.push((field("updated_at"), Operator::Equal, time_now()));

    query([Statement::Update(UpdateStatement {
        what: Values(vec![thing.clone().into()]),
        data: Data::UpdateExpression(update).into(),
        output: Output::After.into(),
        cond: Cond(
            Expression::Binary {
                l: field("id").into(),
                o: Operator::Equal,
                r: thing.into(),
            }
            .into(),
        )
        .into(),
        ..Default::default()
    })])
    .into()
}

pub fn define_uniq_index(
    index: impl Into<String>,
    table: &str,
    fields: impl Into<Vec<Idiom>>,
) -> Query {
    query([Statement::Define(DefineStatement::Index(
        DefineIndexStatement {
            name: index.into().into(),
            what: table.into(),
            cols: Idioms(fields.into()),
            index: Index::Uniq,
            ..Default::default()
        },
    ))])
}

#[inline]
pub fn time_now() -> Value {
    Value::Function(Box::new(Function::Normal("time::now".into(), vec![])))
}
