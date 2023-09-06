use async_graphql::MaybeUndefined;
use chrono::{DateTime, Utc};
use secrecy::{ExposeSecret as _, Secret, Zeroize};

use super::srql;

pub trait QueryValue {
    fn into_query_value(self, field: srql::Idiom) -> Option<srql::SetExprItem>;

    fn push_field(self, field: srql::Idiom, expr: &mut srql::SetExpr)
    where
        Self: Sized,
    {
        if let Some(v) = self.into_query_value(field) {
            expr.push(v);
        }
    }
}

impl QueryValue for String {
    fn into_query_value(self, field: srql::Idiom) -> Option<srql::SetExprItem> {
        Some((
            field,
            srql::Operator::Equal,
            srql::Value::Strand(self.into()),
        ))
    }
}

impl QueryValue for srql::Thing {
    fn into_query_value(self, field: srql::Idiom) -> Option<srql::SetExprItem> {
        Some((field, srql::Operator::Equal, srql::Value::Thing(self)))
    }
}

impl QueryValue for DateTime<Utc> {
    fn into_query_value(self, field: srql::Idiom) -> Option<srql::SetExprItem> {
        Some((
            field,
            srql::Operator::Equal,
            srql::Value::Datetime(srql::Datetime(self)),
        ))
    }
}

impl<T: QueryValue> QueryValue for Option<T> {
    fn into_query_value(self, field: srql::Idiom) -> Option<srql::SetExprItem> {
        self.and_then(|v| v.into_query_value(field))
    }
}

impl<T: QueryValue> QueryValue for MaybeUndefined<T> {
    fn into_query_value(self, field: srql::Idiom) -> Option<srql::SetExprItem> {
        use MaybeUndefined as E;
        match self {
            E::Value(v) => v.into_query_value(field),
            E::Null => Some((field, srql::Operator::Equal, srql::Value::None)),
            E::Undefined => None,
        }
    }
}

impl<T: Zeroize + ToOwned<Owned = O>, O: QueryValue> QueryValue for Secret<T> {
    fn into_query_value(self, field: srql::Idiom) -> Option<srql::SetExprItem> {
        self.expose_secret().to_owned().into_query_value(field)
    }
}
