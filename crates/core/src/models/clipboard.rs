use serde::{Deserialize, Serialize};
use super::common::ItemMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    #[serde(flatten)]
    pub meta: ItemMetadata,
    pub content: String,
    #[serde(default)]
    pub persist_to_disk: bool,   // true = save to DB, false = memory only (but still sync meta)
}