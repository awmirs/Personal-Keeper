use serde::{Deserialize, Serialize};
use super::common::ItemMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    #[serde(flatten)]
    pub meta: ItemMetadata,
    pub url: String,
    pub title: String,
    pub description: String,
    pub favicon: Option<Vec<u8>>,          // stored binary
    pub thumbnail: Option<Vec<u8>>,        // Open Graph image
}