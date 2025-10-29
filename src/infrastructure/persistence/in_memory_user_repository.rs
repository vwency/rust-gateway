use crate::domain::entities::user::User;
use crate::domain::repositories::user_repository::UserRepository;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct InMemoryUserRepository {
    users: RwLock<HashMap<String, User>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn create(&self, user: User) -> Result<User, String> {
        let mut users = self.users.write().await;
        users.insert(user.id.clone(), user.clone());
        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, String> {
        let users = self.users.read().await;
        Ok(users.values().find(|u| u.email == email).cloned())
    }

    async fn find_by_login(&self, login: &str) -> Result<Option<User>, String> {
        let users = self.users.read().await;
        Ok(users.values().find(|u| u.login == login).cloned())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<User>, String> {
        let users = self.users.read().await;
        Ok(users.get(id).cloned())
    }
}
