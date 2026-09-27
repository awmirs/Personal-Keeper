// crates/api/src/routes/import_common.rs
// Shared helpers used by every vault's bulk /import endpoint: the
// ImportStrategy / ImportSummary DTOs and the per-item normalizer that
// repairs partially-sourced data (hand-written CSV rows, JSON-text arrays,
// string booleans, blank timestamps, and so on) so that any exported item
// can be round-tripped back through the create path.

use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImportStrategy {
    Skip,
    Replace,
    Copy,
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(Serialize)]
pub struct ImportSummary {
    pub created: usize,
    pub replaced: usize,
    pub skipped: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

impl ImportSummary {
    pub fn new() -> Self {
        Self {
            created: 0,
            replaced: 0,
            skipped: 0,
            failed: 0,
            errors: Vec::new(),
        }
    }
}

impl Default for ImportSummary {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-vault configuration for `fill_import_defaults`. Everything not
/// listed here (`id`, `created_at`, `updated_at`, `trash_status`,
/// `position`, `tags`, `color`, `is_favorite`) is handled identically for
/// all vaults by the shared normalizer.
pub struct ImportSpec {
    /// String fields that default to "" when missing.
    pub string_defaults: &'static [&'static str],
    /// Boolean fields that default to `false` when missing and are coerced
    /// from the usual "true"/"1"/"yes" spellings. The `bool` is the default
    /// used when the field is absent.
    pub bool_fields: &'static [(&'static str, bool)],
    /// Fields holding a list of plain strings (contacts' phones/emails/
    /// addresses); arrays, JSON text and comma-separated strings are all
    /// normalized.
    pub string_list_fields: &'static [&'static str],
    /// When true, `due_date` is normalized to `Option<i64>`.
    pub has_due_date: bool,
    /// When true, `favicon` and `thumbnail` binary blobs are normalized.
    pub has_binary_fields: bool,
}

/// Normalizes a raw imported item so it always deserializes into the vault
/// model: fills server-side defaults for missing metadata and repairs
/// partially-sourced values — blank, non-string or invalid ids; string
/// booleans; comma-separated or JSON-text tags; blank or unparsable
/// timestamps, positions and colours; and other hand-written-file quirks.
pub fn fill_import_defaults(
    mut value: serde_json::Value,
    next_pos: &mut f64,
    spec: &ImportSpec,
) -> serde_json::Value {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default();
    if let serde_json::Value::Object(ref mut map) = value {
        // Ids: missing, blank, non-string or invalid values get a fresh UUID
        // so hand-written rows never collide or fail UUID parsing.
        let id_valid = match map.get("id") {
            Some(serde_json::Value::String(s)) => uuid::Uuid::parse_str(s.trim()).is_ok(),
            _ => false,
        };
        if !id_valid {
            map.insert(
                "id".to_string(),
                serde_json::json!(uuid::Uuid::now_v7().to_string()),
            );
        }
        // Timestamps: blank or unparsable values fall back to "now".
        let created_raw = map.remove("created_at").unwrap_or(serde_json::json!(now));
        map.insert(
            "created_at".to_string(),
            serde_json::json!(coerce_epoch(created_raw, now)),
        );
        let updated_raw = map.remove("updated_at").unwrap_or(serde_json::json!(now));
        map.insert(
            "updated_at".to_string(),
            serde_json::json!(coerce_epoch(updated_raw, now)),
        );
        // Trash status: blank or non-string values reset to "Active".
        let trash_raw = map
            .remove("trash_status")
            .unwrap_or(serde_json::json!("Active"));
        let trash_status = match trash_raw {
            serde_json::Value::String(s) if !s.trim().is_empty() => {
                serde_json::json!(s.trim())
            }
            _ => serde_json::json!("Active"),
        };
        map.insert("trash_status".to_string(), trash_status);
        // Per-vault string defaults.
        for key in spec.string_defaults {
            map.entry((*key).to_string())
                .or_insert_with(|| serde_json::json!(""));
        }
        // is_favorite is common to every vault.
        {
            let raw = map
                .remove("is_favorite")
                .unwrap_or(serde_json::json!(false));
            map.insert(
                "is_favorite".to_string(),
                serde_json::json!(coerce_bool(raw).unwrap_or(false)),
            );
        }
        // Per-vault boolean fields with explicit defaults.
        for &(key, default) in spec.bool_fields {
            let raw = map
                .remove(key)
                .unwrap_or_else(|| serde_json::json!(default));
            map.insert(
                key.to_string(),
                serde_json::json!(coerce_bool(raw).unwrap_or(default)),
            );
        }
        // Position: blank or unparsable values get the next free slot so the
        // imported ordering stays consistent.
        let position_raw = map.remove("position");
        let position = match position_raw {
            Some(serde_json::Value::Number(n)) => n.as_f64(),
            Some(serde_json::Value::String(s)) => s.trim().parse::<f64>().ok(),
            _ => None,
        };
        let position = match position {
            Some(position) => position,
            None => {
                let position = *next_pos;
                *next_pos += 1.0;
                position
            }
        };
        map.insert("position".to_string(), serde_json::json!(position));
        // Tags: arrays, JSON text, comma-separated or bare strings all become
        // a list of well-formed tag objects with valid ids.
        let tags = map.remove("tags").unwrap_or(serde_json::json!([]));
        map.insert("tags".to_string(), normalize_tag_list(tags));
        // Per-vault plain string lists (contacts).
        for key in spec.string_list_fields {
            let raw = map.remove(*key).unwrap_or(serde_json::json!([]));
            map.insert((*key).to_string(), normalize_string_list(raw));
        }
        // Optional due date (todos).
        if spec.has_due_date {
            let due_raw = map.remove("due_date").unwrap_or(serde_json::Value::Null);
            map.insert(
                "due_date".to_string(),
                coerce_opt_epoch(due_raw)
                    .map(|v| serde_json::json!(v))
                    .unwrap_or(serde_json::Value::Null),
            );
        }
        // Binary blobs (bookmarks): JSON-text arrays round-trip; anything
        // else is dropped rather than failing the item.
        if spec.has_binary_fields {
            for key in ["favicon", "thumbnail"] {
                let raw = map.remove(key).unwrap_or(serde_json::Value::Null);
                let normalized = match raw {
                    serde_json::Value::Array(_) => raw,
                    serde_json::Value::String(text) => {
                        let trimmed = text.trim();
                        if trimmed.is_empty() {
                            serde_json::Value::Null
                        } else {
                            serde_json::from_str::<serde_json::Value>(trimmed)
                                .ok()
                                .filter(|parsed| parsed.is_array())
                                .unwrap_or(serde_json::Value::Null)
                        }
                    }
                    _ => serde_json::Value::Null,
                };
                map.insert(key.to_string(), normalized);
            }
        }
        // Colour: blank becomes null, bare names become full labels and JSON
        // text/objects are repaired into well-formed { name, hex } labels.
        let color = map.remove("color").unwrap_or(serde_json::Value::Null);
        map.insert("color".to_string(), normalize_color(color));
    }
    value
}

/// Splits a bare comma-separated string into JSON string values.
pub fn split_csv_like(text: &str) -> Vec<serde_json::Value> {
    text.split(',')
        .map(|part| serde_json::json!(part.trim()))
        .collect()
}

/// Converts a raw tags value (array, JSON text, comma-separated or bare
/// string) into a list of well-formed tag objects, generating ids where
/// they are missing or invalid.
pub fn normalize_tag_list(value: serde_json::Value) -> serde_json::Value {
    let entries: Vec<serde_json::Value> = match value {
        serde_json::Value::Array(entries) => entries,
        serde_json::Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                Vec::new()
            } else {
                if trimmed.starts_with('[') {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        if parsed.is_array() {
                            return normalize_tag_list(parsed);
                        }
                    }
                }
                split_csv_like(trimmed)
            }
        }
        _ => Vec::new(),
    };
    let tags = entries
        .into_iter()
        .filter_map(|entry| match entry {
            serde_json::Value::String(name) => {
                let name = name.trim();
                if name.is_empty() {
                    None
                } else {
                    Some(serde_json::json!({
                        "id": uuid::Uuid::now_v7().to_string(),
                        "name": name,
                    }))
                }
            }
            serde_json::Value::Object(mut obj) => {
                let name = obj
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(str::trim)
                    .unwrap_or("")
                    .to_string();
                if name.is_empty() {
                    None
                } else {
                    let id_valid = obj
                        .get("id")
                        .and_then(|v| v.as_str())
                        .map(|s| uuid::Uuid::parse_str(s).is_ok())
                        .unwrap_or(false);
                    if !id_valid {
                        obj.insert(
                            "id".to_string(),
                            serde_json::json!(uuid::Uuid::now_v7().to_string()),
                        );
                    }
                    obj.insert("name".to_string(), serde_json::json!(name));
                    Some(serde_json::Value::Object(obj))
                }
            }
            _ => None,
        })
        .collect::<Vec<serde_json::Value>>();
    serde_json::Value::Array(tags)
}

/// Converts a raw list value (array, JSON text, comma-separated or bare
/// string) into an array of trimmed, non-empty strings.
pub fn normalize_string_list(value: serde_json::Value) -> serde_json::Value {
    let entries: Vec<serde_json::Value> = match value {
        serde_json::Value::Array(entries) => entries,
        serde_json::Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                Vec::new()
            } else {
                if trimmed.starts_with('[') {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        if parsed.is_array() {
                            return normalize_string_list(parsed);
                        }
                    }
                }
                split_csv_like(trimmed)
            }
        }
        other => vec![other],
    };
    let list = entries
        .into_iter()
        .filter_map(|entry| match entry {
            serde_json::Value::String(text) => {
                let text = text.trim();
                if text.is_empty() {
                    None
                } else {
                    Some(serde_json::Value::String(text.to_string()))
                }
            }
            serde_json::Value::Null => None,
            other => Some(serde_json::Value::String(other.to_string())),
        })
        .collect::<Vec<serde_json::Value>>();
    serde_json::Value::Array(list)
}

/// Coerces "true"/"false"/"1"/"0"/"" and JSON booleans/numbers to a bool.
pub fn coerce_bool(value: serde_json::Value) -> Option<bool> {
    match value {
        serde_json::Value::Bool(parsed) => Some(parsed),
        serde_json::Value::Number(n) => Some(n.as_f64().unwrap_or(0.0) != 0.0),
        serde_json::Value::String(text) => match text.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" => Some(true),
            "false" | "0" | "no" | "" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

/// Coerces a raw timestamp value (number, numeric or blank string) to
/// epoch seconds, falling back to `fallback` when unparsable.
pub fn coerce_epoch(value: serde_json::Value, fallback: i64) -> i64 {
    match value {
        serde_json::Value::Number(n) => n.as_f64().map(|f| f as i64).unwrap_or(fallback),
        serde_json::Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                fallback
            } else {
                trimmed.parse::<i64>().unwrap_or(fallback)
            }
        }
        _ => fallback,
    }
}

/// Coerces a raw optional timestamp to epoch seconds; unparsable values
/// become None instead of failing the item.
pub fn coerce_opt_epoch(value: serde_json::Value) -> Option<i64> {
    match value {
        serde_json::Value::Number(n) => n.as_f64().map(|f| f as i64),
        serde_json::Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                trimmed.parse::<i64>().ok()
            }
        }
        _ => None,
    }
}

/// Blank colours become null; bare colour names become full labels; JSON
/// text and objects are repaired into well-formed { name, hex } labels.
pub fn normalize_color(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Null => serde_json::Value::Null,
        serde_json::Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                serde_json::Value::Null
            } else if trimmed.starts_with('{') {
                normalize_color(
                    serde_json::from_str::<serde_json::Value>(trimmed)
                        .unwrap_or(serde_json::Value::Null),
                )
            } else {
                serde_json::json!({ "name": trimmed, "hex": "" })
            }
        }
        serde_json::Value::Object(obj) => {
            let name = obj
                .get("name")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .unwrap_or("");
            let hex = obj
                .get("hex")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .unwrap_or("");
            serde_json::json!({ "name": name, "hex": hex })
        }
        _ => serde_json::Value::Null,
    }
}
