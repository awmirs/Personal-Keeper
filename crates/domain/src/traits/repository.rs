use async_trait::async_trait;
use crate::error::CoreError;

#[async_trait]
pub trait Repository<T> {
    async fn save(&self, user_id: &str, item: &T) -> Result<(), CoreError>;
    async fn find_by_id(&self, user_id: &str, id: &str) -> Result<Option<T>, CoreError>;
    async fn find_all(&self, user_id: &str) -> Result<Vec<T>, CoreError>;
    async fn delete(&self, user_id: &str, id: &str) -> Result<(), CoreError>;
    async fn search(&self, user_id: &str, query: &str) -> Result<Vec<T>, CoreError>;
}