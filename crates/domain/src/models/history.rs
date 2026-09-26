use serde::{Deserialize, Serialize};

/// Item types that participate in version history.
pub const HISTORY_ITEM_TYPES: &[&str] = &[
    "note",
    "clipboard",
    "todo",
    "bookmark",
    "contact",
    "credential",
];

/// Returns true when the given item type participates in version history.
pub fn is_valid_item_type(item_type: &str) -> bool {
    HISTORY_ITEM_TYPES.contains(&item_type)
}

/// A single recorded version (full snapshot) of a vault item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemVersion {
    /// Unique id of the version row itself.
    pub id: String,
    /// One of [`HISTORY_ITEM_TYPES`].
    pub item_type: String,
    /// Id of the versioned item.
    pub item_id: String,
    /// Owning user (multi-tenant isolation).
    pub user_id: String,
    /// Monotonically increasing per-item version number (1, 2, 3, ...).
    pub version: i64,
    /// One of: `created`, `updated`, `deleted`, `restored`.
    pub operation: String,
    /// Full JSON snapshot of the item row at this point in time.
    pub data: String,
    /// Unix timestamp (seconds) when this version was recorded.
    pub created_at: i64,
}
