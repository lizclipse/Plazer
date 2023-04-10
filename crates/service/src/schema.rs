use crate::account::{AccountMutation, AccountQuery};

use async_graphql::{EmptySubscription, MergedObject, Schema};

#[derive(MergedObject, Default)]
pub struct Query(AccountQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(AccountMutation);

pub type ServiceSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn schema() -> ServiceSchema {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription).finish()
}
