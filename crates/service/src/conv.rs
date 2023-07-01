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
        ID(self.to_owned())
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
            SrlId::String(s) => ID(s.to_owned()),
            id => ID(id.to_raw()),
        }
    }
}

impl ToGqlId for Thing {
    fn to_gql_id(&self) -> ID {
        self.id.to_gql_id()
    }
}
