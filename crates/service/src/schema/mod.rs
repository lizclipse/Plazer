mod auth;

use self::auth::{AuthMutation, AuthQuery, AuthSubscription};

use async_graphql::{MergedObject, MergedSubscription, Schema};

#[derive(MergedObject, Default)]
pub struct Query(AuthQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(AuthMutation);

#[derive(MergedSubscription, Default)]
pub struct Subscription(AuthSubscription);

pub type ServiceSchema = Schema<Query, Mutation, Subscription>;

pub fn schema() -> ServiceSchema {
    Schema::build(
        Query::default(),
        Mutation::default(),
        Subscription::default(),
    )
    .finish()
}
