// crates/api/src/routes/dto.rs
// DTOs shared across every vault route.

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct ReorderRequest {
    pub positions: Vec<PositionEntry>,
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct PositionEntry {
    pub id: String,
    pub position: f64,
}


/// Pagination query parameters shared by every list endpoint.
/// `limit` is clamped to `Pagination::MAX_LIMIT`; `cursor` is the opaque
/// token returned in `next_cursor` from the previous page.
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct ListQuery {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
