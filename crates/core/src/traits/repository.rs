use async_trait::async_trait;
use crate::error::CoreError;

#[async_trait]
pub trait Repository<T> {
    async fn save(&self, item: &T) -> Result<(), CoreError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<T>, CoreError>;
    async fn find_all(&self) -> Result<Vec<T>, CoreError>;
    async fn delete(&self, id: &str) -> Result<(), CoreError>;
    async fn search(&self, query: &str) -> Result<Vec<T>, CoreError>;
}