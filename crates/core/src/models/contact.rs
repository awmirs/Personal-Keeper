use serde::{Deserialize, Serialize};
use super::common::ItemMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    #[serde(flatten)]
    pub meta: ItemMetadata,
    pub name: String,
    pub phones: Vec<String>,
    pub emails: Vec<String>,
    pub addresses: Vec<String>,
    pub notes: String,
}