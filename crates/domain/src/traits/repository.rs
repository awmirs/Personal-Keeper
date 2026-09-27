use async_trait::async_trait;
use crate::error::CoreError;

#[async_trait]
pub trait Repository<T: Send + Sync + 'static>: Send + Sync {
    async fn save(&self, user_id: &str, item: &T) -> Result<(), CoreError>;
    async fn find_by_id(&self, user_id: &str, id: &str) -> Result<Option<T>, CoreError>;
    async fn find_all(&self, user_id: &str) -> Result<Vec<T>, CoreError>;
    async fn delete(&self, user_id: &str, id: &str) -> Result<(), CoreError>;
    async fn search(&self, user_id: &str, query: &str) -> Result<Vec<T>, CoreError>;
    async fn get_next_position(&self, user_id: &str) -> Result<f64, CoreError>;
    async fn update_positions(
        &self,
        user_id: &str,
        positions: &[(uuid::Uuid, f64)],
    ) -> Result<(), CoreError>;
}
