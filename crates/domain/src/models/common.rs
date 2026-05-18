use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

/// Status of an item in the recycler
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrashStatus {
    Active,
    Trashed,
    Deleted,
}

/// A colour label for visual grouping
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColorLabel {
    pub name: String,
    pub hex: String,
}

/// A tag that can be attached to any item
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
}

impl Tag {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
        }
    }
}

/// Shared metadata across all vault items
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemMetadata {
    pub id: Uuid,
    pub created_at: i64,            // unix timestamp (seconds)
    pub updated_at: i64,
    pub tags: Vec<Tag>,
    pub color: Option<ColorLabel>,
    pub is_favorite: bool,
    pub trash_status: TrashStatus,
    pub position: f64, // Manual ordering position (lower = earlier in the list).
}


impl Default for ItemMetadata {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        Self {
            id: Uuid::now_v7(),
            created_at: now,
            updated_at: now,
            tags: vec![],
            color: None,
            is_favorite: false,
            trash_status: TrashStatus::Active,
            position: 0.0,
        }
    }
}