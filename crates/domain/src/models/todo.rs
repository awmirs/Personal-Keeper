use serde::{Deserialize, Serialize};
use super::common::ItemMetadata;

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    #[serde(flatten)]
    pub meta: ItemMetadata,
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub due_date: Option<i64>,
}