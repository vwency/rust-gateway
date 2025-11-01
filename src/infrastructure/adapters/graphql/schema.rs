use crate::application::graphql::mutations::login_mutation::LoginMutation;
use crate::application::graphql::mutations::register_mutation::RegisterMutation;
use crate::application::graphql::queries::health_query::HealthQuery;
use crate::infrastructure::adapters::kratos::kratos_client::KratosClient;
use async_graphql::{EmptySubscription, MergedObject, Schema};

#[derive(MergedObject, Default)]
pub struct QueryRoot(HealthQuery);

#[derive(MergedObject, Default)]
pub struct MutationRoot(RegisterMutation, LoginMutation);

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema(jwt_secret: String) -> AppSchema {
    let kratos_client = KratosClient::new(
        "http://localhost:4434".to_string(), // admin URL
        "http://localhost:4433".to_string(), // public URL
    );

    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(jwt_secret)
    .data(kratos_client) // Add KratosClient to schema data
    .finish()
}
