use async_graphql::InputObject;

#[derive(InputObject, Clone)]
pub struct RegisterInput {
    pub email: String,
    pub login: String,
    pub password: String,
}

#[derive(InputObject, Clone)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}
