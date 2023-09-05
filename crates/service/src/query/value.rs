use async_graphql::MaybeUndefined;
use chrono::{DateTime, Utc};
use secrecy::{ExposeSecret as _, Secret, Zeroize};

use super::srql;

pub trait QueryValue {
    fn push_field(self, field: srql::Idiom, expr: &mut srql::SetExpr);
}

impl QueryValue for String {
    fn push_field(self, field: srql::Idiom, expr: &mut srql::SetExpr) {
        expr.push((
            field,
            srql::Operator::Equal,
            srql::Value::Strand(self.into()),
        ));
    }
}

impl QueryValue for srql::Thing {
    fn push_field(self, field: srql::Idiom, expr: &mut srql::SetExpr) {
        expr.push((field, srql::Operator::Equal, srql::Value::Thing(self)));
    }
}

impl QueryValue for DateTime<Utc> {
    fn push_field(self, field: srql::Idiom, expr: &mut srql::SetExpr) {
        expr.push((
            field,
            srql::Operator::Equal,
            srql::Value::Datetime(srql::Datetime(self)),
        ));
    }
}

impl<T: QueryValue> QueryValue for Option<T> {
    fn push_field(self, field: srql::Idiom, expr: &mut srql::SetExpr) {
        if let Some(v) = self {
            v.push_field(field, expr);
        }
    }
}

impl<T: QueryValue> QueryValue for MaybeUndefined<T> {
    fn push_field(self, field: srql::Idiom, expr: &mut srql::SetExpr) {
        use MaybeUndefined as E;
        match self {
            E::Value(v) => v.push_field(field, expr),
            E::Null => expr.push((field, srql::Operator::Equal, srql::Value::None)),
            E::Undefined => (),
        }
    }
}

impl<T: Zeroize + ToOwned<Owned = O>, O: QueryValue> QueryValue for Secret<T> {
    fn push_field(self, field: srql::Idiom, expr: &mut srql::SetExpr) {
        self.expose_secret().to_owned().push_field(field, expr);
    }
}
