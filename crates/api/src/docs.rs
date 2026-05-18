// This entire module exists only when the "swagger" feature is enabled.
#[cfg(feature = "swagger")]
use utoipa::openapi::{
    self, Info, PathItem, PathItemType, RefOr, Response,
};
#[cfg(feature = "swagger")]
use utoipa::OpenApi;

#[cfg(feature = "swagger")]
/// Build the complete OpenAPI v3 specification for the Personal Keeper API.
fn build_openapi() -> openapi::OpenApi {
    // Convenience: create a JSON response with a schema reference.
    let json_response = |schema_ref: RefOr<openapi::Schema>| -> Response {
        Response::new("OK")
            .description("Successful response")
            .content("application/json", openapi::Content::new(schema_ref))
    };

    let json_array_response = |schema_ref: RefOr<openapi::Schema>| -> Response {
        Response::new("OK")
            .description("Successful response")
            .content(
                "application/json",
                openapi::Content::new(openapi::schema::Array::new(schema_ref)),
            )
    };

    let no_content_response = || {
        Response::new("No Content").description("Resource deleted successfully")
    };

    let error_response = || {
        Response::new("Internal Server Error")
            .description("An unexpected error occurred")
            .content(
                "application/json",
                openapi::Content::new(openapi::Object::new()),
            )
    };

    let mut paths = openapi::path::Paths::new();

    // ---------- Health ----------
    paths.insert(
        "/health".to_string(),
        PathItem::new(
            PathItemType::Get,
            openapi::Operation::new()
                .summary("Service health check")
                .response("200", json_response(openapi::schema::Object::new().into()))
                .tag("Health"),
        ),
    );

    // ---------- Auth ----------
    paths.insert(
        "/api/auth/register".to_string(),
        PathItem::new(
            PathItemType::Post,
            openapi::Operation::new()
                .summary("Register a new user")
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("RegisterRequest"))))
                    .build())
                .response("201", json_response(RefOr::Ref(openapi::Ref::from_schema_name("AuthResponse"))))
                .response("400", Response::new("Bad Request"))
                .response("409", Response::new("Username already exists"))
                .tag("Auth"),
        ),
    );

    paths.insert(
        "/api/auth/login".to_string(),
        PathItem::new(
            PathItemType::Post,
            openapi::Operation::new()
                .summary("Log in with username and password")
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("LoginRequest"))))
                    .build())
                .response("200", json_response(RefOr::Ref(openapi::Ref::from_schema_name("AuthResponse"))))
                .response("401", Response::new("Invalid credentials"))
                .tag("Auth"),
        ),
    );

    paths.insert(
        "/api/auth/refresh".to_string(),
        PathItem::new(
            PathItemType::Post,
            openapi::Operation::new()
                .summary("Refresh access token")
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("RefreshRequest"))))
                    .build())
                .response("200", json_response(RefOr::Ref(openapi::Ref::from_schema_name("AuthResponse"))))
                .response("401", Response::new("Invalid refresh token"))
                .tag("Auth"),
        ),
    );

    paths.insert(
        "/api/auth/me".to_string(),
        PathItem::new(
            PathItemType::Get,
            openapi::Operation::new()
                .summary("Get current user info")
                .response("200", json_response(openapi::Object::new().into()))
                .response("401", Response::new("Not authenticated"))
                .tag("Auth"),
        ),
    );

    // ---------- Notes ----------
    paths.insert(
        "/api/notes".to_string(),
        PathItem::new(
            PathItemType::Post,
            openapi::Operation::new()
                .summary("Create a new note")
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("CreateNoteRequest"))))
                    .build())
                .response("201", json_response(RefOr::Ref(openapi::Ref::from_schema_name("Note"))))
                .response("500", error_response())
                .tag("Notes"),
        ),
    );
    paths.entry("/api/notes".to_string()).or_default().operations.get_or_insert_with(|| openapi::path::Operations::new()).get = Some(
        openapi::Operation::new()
            .summary("List all notes")
            .response("200", json_array_response(RefOr::Ref(openapi::Ref::from_schema_name("Note"))))
            .response("500", error_response())
            .tag("Notes"),
    );

    paths.insert(
        "/api/notes/{id}".to_string(),
        PathItem::new(
            PathItemType::Put,
            openapi::Operation::new()
                .summary("Update a note")
                .parameter(openapi::path::PathParameter::new("id").schema(openapi::schema::Schema::String(Default::default())))
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("UpdateNoteRequest"))))
                    .build())
                .response("200", json_response(RefOr::Ref(openapi::Ref::from_schema_name("Note"))))
                .response("404", Response::new("Note not found"))
                .response("500", error_response())
                .tag("Notes"),
        ),
    );
    paths.entry("/api/notes/{id}".to_string()).or_default().operations.get_or_insert_with(|| openapi::path::Operations::new()).delete = Some(
        openapi::Operation::new()
            .summary("Delete a note")
            .parameter(openapi::path::PathParameter::new("id").schema(openapi::schema::Schema::String(Default::default())))
            .response("204", no_content_response())
            .response("500", error_response())
            .tag("Notes"),
    );

    paths.insert(
        "/api/notes/reorder".to_string(),
        PathItem::new(
            PathItemType::Put,
            openapi::Operation::new()
                .summary("Reorder notes")
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("ReorderRequest"))))
                    .build())
                .response("200", json_response(openapi::Object::new().into()))
                .response("500", error_response())
                .tag("Notes"),
        ),
    );

    // ---------- Clipboard ----------
    paths.insert(
        "/api/clipboard".to_string(),
        PathItem::new(
            PathItemType::Post,
            openapi::Operation::new()
                .summary("Create a clipboard item")
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("CreateClipboardRequest"))))
                    .build())
                .response("201", json_response(RefOr::Ref(openapi::Ref::from_schema_name("ClipboardItem"))))
                .response("500", error_response())
                .tag("Clipboard"),
        ),
    );
    paths.entry("/api/clipboard".to_string()).or_default().operations.get_or_insert_with(|| openapi::path::Operations::new()).get = Some(
        openapi::Operation::new()
            .summary("List clipboard items (optional search)")
            .parameter(openapi::query::QueryParameter::new("search").schema(openapi::Schema::String(Default::default())).required(false))
            .response("200", json_array_response(RefOr::Ref(openapi::Ref::from_schema_name("ClipboardItem"))))
            .response("500", error_response())
            .tag("Clipboard"),
    );

    paths.insert(
        "/api/clipboard/{id}".to_string(),
        PathItem::new(
            PathItemType::Delete,
            openapi::Operation::new()
                .summary("Delete a clipboard item")
                .parameter(openapi::path::PathParameter::new("id").schema(openapi::schema::Schema::String(Default::default())))
                .response("204", no_content_response())
                .response("500", error_response())
                .tag("Clipboard"),
        ),
    );
    paths.insert(
        "/api/clipboard/reorder".to_string(),
        PathItem::new(
            PathItemType::Put,
            openapi::Operation::new()
                .summary("Reorder clipboard items")
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("ReorderRequest"))))
                    .build())
                .response("200", json_response(openapi::Object::new().into()))
                .response("500", error_response())
                .tag("Clipboard"),
        ),
    );

    // ---------- Todos ----------
    paths.insert(
        "/api/todos".to_string(),
        PathItem::new(
            PathItemType::Post,
            openapi::Operation::new()
                .summary("Create a todo")
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("CreateTodoRequest"))))
                    .build())
                .response("201", json_response(RefOr::Ref(openapi::Ref::from_schema_name("Todo"))))
                .response("500", error_response())
                .tag("Todos"),
        ),
    );
    paths.entry("/api/todos".to_string()).or_default().operations.get_or_insert_with(|| openapi::path::Operations::new()).get = Some(
        openapi::Operation::new()
            .summary("List todos (optional search)")
            .parameter(openapi::query::QueryParameter::new("search").schema(openapi::Schema::String(Default::default())).required(false))
            .response("200", json_array_response(RefOr::Ref(openapi::Ref::from_schema_name("Todo"))))
            .response("500", error_response())
            .tag("Todos"),
    );

    paths.insert(
        "/api/todos/{id}".to_string(),
        PathItem::new(
            PathItemType::Put,
            openapi::Operation::new()
                .summary("Update a todo")
                .parameter(openapi::path::PathParameter::new("id").schema(openapi::schema::Schema::String(Default::default())))
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("UpdateTodoRequest"))))
                    .build())
                .response("200", json_response(RefOr::Ref(openapi::Ref::from_schema_name("Todo"))))
                .response("404", Response::new("Todo not found"))
                .response("500", error_response())
                .tag("Todos"),
        ),
    );
    paths.entry("/api/todos/{id}".to_string()).or_default().operations.get_or_insert_with(|| openapi::path::Operations::new()).delete = Some(
        openapi::Operation::new()
            .summary("Delete a todo")
            .parameter(openapi::path::PathParameter::new("id").schema(openapi::schema::Schema::String(Default::default())))
            .response("204", no_content_response())
            .response("500", error_response())
            .tag("Todos"),
    );

    paths.insert(
        "/api/todos/reorder".to_string(),
        PathItem::new(
            PathItemType::Put,
            openapi::Operation::new()
                .summary("Reorder todos")
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("ReorderRequest"))))
                    .build())
                .response("200", json_response(openapi::Object::new().into()))
                .response("500", error_response())
                .tag("Todos"),
        ),
    );

    // ---------- Bookmarks ----------
    paths.insert(
        "/api/bookmarks".to_string(),
        PathItem::new(
            PathItemType::Post,
            openapi::Operation::new()
                .summary("Create a bookmark")
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("CreateBookmarkRequest"))))
                    .build())
                .response("201", json_response(RefOr::Ref(openapi::Ref::from_schema_name("Bookmark"))))
                .response("500", error_response())
                .tag("Bookmarks"),
        ),
    );
    paths.entry("/api/bookmarks".to_string()).or_default().operations.get_or_insert_with(|| openapi::path::Operations::new()).get = Some(
        openapi::Operation::new()
            .summary("List bookmarks (optional search)")
            .parameter(openapi::query::QueryParameter::new("search").schema(openapi::Schema::String(Default::default())).required(false))
            .response("200", json_array_response(RefOr::Ref(openapi::Ref::from_schema_name("Bookmark"))))
            .response("500", error_response())
            .tag("Bookmarks"),
    );

    paths.insert(
        "/api/bookmarks/{id}".to_string(),
        PathItem::new(
            PathItemType::Put,
            openapi::Operation::new()
                .summary("Update a bookmark")
                .parameter(openapi::path::PathParameter::new("id").schema(openapi::schema::Schema::String(Default::default())))
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("UpdateBookmarkRequest"))))
                    .build())
                .response("200", json_response(RefOr::Ref(openapi::Ref::from_schema_name("Bookmark"))))
                .response("404", Response::new("Bookmark not found"))
                .response("500", error_response())
                .tag("Bookmarks"),
        ),
    );
    paths.entry("/api/bookmarks/{id}".to_string()).or_default().operations.get_or_insert_with(|| openapi::path::Operations::new()).delete = Some(
        openapi::Operation::new()
            .summary("Delete a bookmark")
            .parameter(openapi::path::PathParameter::new("id").schema(openapi::schema::Schema::String(Default::default())))
            .response("204", no_content_response())
            .response("500", error_response())
            .tag("Bookmarks"),
    );

    paths.insert(
        "/api/bookmarks/reorder".to_string(),
        PathItem::new(
            PathItemType::Put,
            openapi::Operation::new()
                .summary("Reorder bookmarks")
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("ReorderRequest"))))
                    .build())
                .response("200", json_response(openapi::Object::new().into()))
                .response("500", error_response())
                .tag("Bookmarks"),
        ),
    );

    // ---------- Contacts ----------
    paths.insert(
        "/api/contacts".to_string(),
        PathItem::new(
            PathItemType::Post,
            openapi::Operation::new()
                .summary("Create a contact")
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("CreateContactRequest"))))
                    .build())
                .response("201", json_response(RefOr::Ref(openapi::Ref::from_schema_name("Contact"))))
                .response("500", error_response())
                .tag("Contacts"),
        ),
    );
    paths.entry("/api/contacts".to_string()).or_default().operations.get_or_insert_with(|| openapi::path::Operations::new()).get = Some(
        openapi::Operation::new()
            .summary("List contacts (optional search)")
            .parameter(openapi::query::QueryParameter::new("search").schema(openapi::Schema::String(Default::default())).required(false))
            .response("200", json_array_response(RefOr::Ref(openapi::Ref::from_schema_name("Contact"))))
            .response("500", error_response())
            .tag("Contacts"),
    );

    paths.insert(
        "/api/contacts/{id}".to_string(),
        PathItem::new(
            PathItemType::Put,
            openapi::Operation::new()
                .summary("Update a contact")
                .parameter(openapi::path::PathParameter::new("id").schema(openapi::schema::Schema::String(Default::default())))
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("UpdateContactRequest"))))
                    .build())
                .response("200", json_response(RefOr::Ref(openapi::Ref::from_schema_name("Contact"))))
                .response("404", Response::new("Contact not found"))
                .response("500", error_response())
                .tag("Contacts"),
        ),
    );
    paths.entry("/api/contacts/{id}".to_string()).or_default().operations.get_or_insert_with(|| openapi::path::Operations::new()).delete = Some(
        openapi::Operation::new()
            .summary("Delete a contact")
            .parameter(openapi::path::PathParameter::new("id").schema(openapi::schema::Schema::String(Default::default())))
            .response("204", no_content_response())
            .response("500", error_response())
            .tag("Contacts"),
    );

    paths.insert(
        "/api/contacts/reorder".to_string(),
        PathItem::new(
            PathItemType::Put,
            openapi::Operation::new()
                .summary("Reorder contacts")
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("ReorderRequest"))))
                    .build())
                .response("200", json_response(openapi::Object::new().into()))
                .response("500", error_response())
                .tag("Contacts"),
        ),
    );

    // ---------- Credentials ----------
    paths.insert(
        "/api/credentials/status".to_string(),
        PathItem::new(
            PathItemType::Get,
            openapi::Operation::new()
                .summary("Check vault configuration status")
                .response("200", json_response(openapi::Object::new().into()))
                .response("500", error_response())
                .tag("Credentials"),
        ),
    );

    paths.insert(
        "/api/credentials/unlock".to_string(),
        PathItem::new(
            PathItemType::Post,
            openapi::Operation::new()
                .summary("Unlock vault or set master password")
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("UnlockRequest"))))
                    .build())
                .response("200", json_response(RefOr::Ref(openapi::Ref::from_schema_name("UnlockResponse"))))
                .response("401", Response::new("Invalid master password"))
                .response("500", error_response())
                .tag("Credentials"),
        ),
    );

    paths.insert(
        "/api/credentials/lock".to_string(),
        PathItem::new(
            PathItemType::Post,
            openapi::Operation::new()
                .summary("Lock the vault")
                .response("200", json_response(openapi::Object::new().into()))
                .response("500", error_response())
                .tag("Credentials"),
        ),
    );

    paths.insert(
        "/api/credentials".to_string(),
        PathItem::new(
            PathItemType::Post,
            openapi::Operation::new()
                .summary("Create a new credential (vault must be unlocked)")
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("CreateCredentialRequest"))))
                    .build())
                .response("201", json_response(RefOr::Ref(openapi::Ref::from_schema_name("Credential"))))
                .response("401", Response::new("Vault locked"))
                .response("500", error_response())
                .tag("Credentials"),
        ),
    );
    paths.entry("/api/credentials".to_string()).or_default().operations.get_or_insert_with(|| openapi::path::Operations::new()).get = Some(
        openapi::Operation::new()
            .summary("List all credentials (no decrypted secrets)")
            .response("200", json_array_response(RefOr::Ref(openapi::Ref::from_schema_name("Credential"))))
            .response("500", error_response())
            .tag("Credentials"),
    );

    paths.insert(
        "/api/credentials/{id}".to_string(),
        PathItem::new(
            PathItemType::Get,
            openapi::Operation::new()
                .summary("Get a single credential with decrypted fields (requires unlock)")
                .parameter(openapi::path::PathParameter::new("id").schema(openapi::schema::Schema::String(Default::default())))
                .response("200", json_response(openapi::Object::new().into()))
                .response("401", Response::new("Vault locked"))
                .response("404", Response::new("Credential not found"))
                .response("500", error_response())
                .tag("Credentials"),
        ),
    );
    paths.entry("/api/credentials/{id}".to_string()).or_default().operations.get_or_insert_with(|| openapi::path::Operations::new()).put = Some(
        openapi::Operation::new()
            .summary("Update a credential")
            .parameter(openapi::path::PathParameter::new("id").schema(openapi::schema::Schema::String(Default::default())))
            .request_body(openapi::request_body::RequestBody::new()
                .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("UpdateCredentialRequest"))))
                .build())
            .response("200", json_response(RefOr::Ref(openapi::Ref::from_schema_name("Credential"))))
            .response("401", Response::new("Vault locked"))
            .response("404", Response::new("Credential not found"))
            .response("500", error_response())
            .tag("Credentials"),
    );
    paths.entry("/api/credentials/{id}".to_string()).or_default().operations.get_or_insert_with(|| openapi::path::Operations::new()).delete = Some(
        openapi::Operation::new()
            .summary("Delete a credential")
            .parameter(openapi::path::PathParameter::new("id").schema(openapi::schema::Schema::String(Default::default())))
            .response("204", no_content_response())
            .response("401", Response::new("Vault locked"))
            .response("500", error_response())
            .tag("Credentials"),
    );

    paths.insert(
        "/api/credentials/reorder".to_string(),
        PathItem::new(
            PathItemType::Put,
            openapi::Operation::new()
                .summary("Reorder credentials")
                .request_body(openapi::request_body::RequestBody::new()
                    .content("application/json", openapi::Content::new(RefOr::Ref(openapi::Ref::from_schema_name("ReorderRequest"))))
                    .build())
                .response("200", json_response(openapi::Object::new().into()))
                .response("401", Response::new("Vault locked"))
                .response("500", error_response())
                .tag("Credentials"),
        ),
    );

    // Build OpenAPI object
    openapi::OpenApi::new()
        .info(
            Info::new("Personal Keeper API", "1.0.0")
                .description("Self-hosted personal knowledge base and vault"),
        )
        .paths(paths)
}

#[cfg(feature = "swagger")]
/// A dummy struct that implements `utoipa::OpenApi` to satisfy the Swagger UI binding.
/// The actual spec comes from `build_openapi()`.
#[derive(Clone)]
pub struct ApiDoc;

#[cfg(feature = "swagger")]
impl OpenApi for ApiDoc {
    fn openapi() -> openapi::OpenApi {
        build_openapi()
    }
}

#[cfg(feature = "swagger")]
/// Returns an actix-web `Scope` that serves the Swagger UI and OpenAPI JSON.
pub fn swagger_ui_service() -> actix_web::Scope {
    use utoipa_swagger_ui::SwaggerUi;

    actix_web::web::scope("/docs")
        .service(SwaggerUi::new("/docs/{_:.*}").url("/docs/openapi.json", ApiDoc::openapi()))
}