use async_graphql::{EmptySubscription, MergedObject, Schema, SchemaBuilder};

use crate::{
    account::{AccountMutation, AccountQuery},
    board::{BoardMutation, BoardQuery},
};

#[derive(MergedObject, Default)]
pub struct Query(AccountQuery, BoardQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(AccountMutation, BoardMutation);

pub type ServiceSchema = Schema<Query, Mutation, EmptySubscription>;

type ServiceSchemaBuilder = SchemaBuilder<Query, Mutation, EmptySubscription>;
pub fn schema<F>(adjust: F) -> ServiceSchema
where
    F: FnOnce(ServiceSchemaBuilder) -> ServiceSchemaBuilder,
{
    adjust(Schema::build(
        Query::default(),
        Mutation::default(),
        EmptySubscription,
    ))
    .finish()
}
