use async_graphql::InputObject;

#[derive(InputObject, Clone)]
pub struct RegisterInput {
    pub email: String,
    pub username: String,
    pub password: String,
    pub geo_location: Option<String>,
}

#[derive(InputObject, Clone)]
pub struct LoginInput {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: String,
}
