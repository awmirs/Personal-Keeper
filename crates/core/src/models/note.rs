use serde::{Deserialize, Serialize};
use super::common::ItemMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    #[serde(flatten)]
    pub meta: ItemMetadata,
    pub title: String,
    pub content: String,          // Markdown
    pub is_pinned: bool,
}