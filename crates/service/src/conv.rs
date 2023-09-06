use async_graphql::ID;
use surrealdb::sql::{Id as SrlId, Thing};

pub trait IntoGqlId {
    fn into_gql_id(self) -> ID;
}

impl IntoGqlId for ID {
    fn into_gql_id(self) -> ID {
        self
    }
}

impl IntoGqlId for String {
    fn into_gql_id(self) -> ID {
        ID(self)
    }
}

impl IntoGqlId for &str {
    fn into_gql_id(self) -> ID {
        ID(self.to_owned())
    }
}

impl IntoGqlId for SrlId {
    fn into_gql_id(self) -> ID {
        match self {
            SrlId::String(s) => ID(s),
            id => ID(id.to_raw()),
        }
    }
}

impl IntoGqlId for Thing {
    fn into_gql_id(self) -> ID {
        self.id.into_gql_id()
    }
}

pub trait ToGqlId {
    fn to_gql_id(&self) -> ID;
}

impl ToGqlId for ID {
    fn to_gql_id(&self) -> ID {
        self.clone()
    }
}

impl ToGqlId for String {
    fn to_gql_id(&self) -> ID {
        ID(self.clone())
    }
}

impl ToGqlId for str {
    fn to_gql_id(&self) -> ID {
        ID(self.to_owned())
    }
}

impl ToGqlId for SrlId {
    fn to_gql_id(&self) -> ID {
        match self {
            SrlId::String(s) => ID(s.clone()),
            id => ID(id.to_raw()),
        }
    }
}

impl ToGqlId for Thing {
    fn to_gql_id(&self) -> ID {
        self.id.to_gql_id()
    }
}

pub trait AsMaybeStr {
    fn as_maybe_str(&self) -> Option<&str>;
}

impl AsMaybeStr for ID {
    fn as_maybe_str(&self) -> Option<&str> {
        Some(&self.0)
    }
}

impl AsMaybeStr for String {
    fn as_maybe_str(&self) -> Option<&str> {
        Some(self)
    }
}

impl AsMaybeStr for str {
    fn as_maybe_str(&self) -> Option<&str> {
        Some(self)
    }
}

impl AsMaybeStr for SrlId {
    fn as_maybe_str(&self) -> Option<&str> {
        match self {
            SrlId::String(s) => Some(s),
            _ => None,
        }
    }
}

impl AsMaybeStr for Thing {
    fn as_maybe_str(&self) -> Option<&str> {
        self.id.as_maybe_str()
    }
}
