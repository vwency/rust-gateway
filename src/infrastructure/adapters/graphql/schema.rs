use crate::infrastructure::adapters::graphql::mutations::login_mutation::LoginMutation;
use crate::infrastructure::adapters::graphql::mutations::register_mutation::RegisterMutation;
use crate::infrastructure::adapters::graphql::queries::health_query::HealthQuery;
use async_graphql::{EmptySubscription, MergedObject, Schema};

#[derive(MergedObject, Default)]
pub struct QueryRoot(HealthQuery);

#[derive(MergedObject, Default)]
pub struct MutationRoot(RegisterMutation, LoginMutation);

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema(jwt_secret: String) -> AppSchema {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(jwt_secret)
    .finish()
}
