// This entire module exists only when the "swagger" feature is enabled.
#[cfg(feature = "swagger")]
use utoipa::openapi::{
    self, RefOr,
    path::{OperationBuilder, PathItemBuilder, PathsBuilder, PathItemType, ParameterBuilder, ParameterIn},
    request_body::RequestBodyBuilder,
    response::{ResponseBuilder, ResponsesBuilder},
    schema::{ObjectBuilder, ArrayBuilder, SchemaType},
    OpenApiBuilder, InfoBuilder, ContentBuilder,
};
#[cfg(feature = "swagger")]
use utoipa::OpenApi;

#[cfg(feature = "swagger")]
#[derive(utoipa::OpenApi)]
#[openapi(
    components(
        schemas(
            // Auth
            crate::routes::auth::RegisterRequest,
            crate::routes::auth::AuthResponse,
            crate::routes::auth::LoginRequest,
            crate::routes::auth::RefreshRequest,

            // Reorder & Position (we only need one instance since they are identical across routes)
            crate::routes::notes::ReorderRequest,
            crate::routes::notes::PositionEntry,

            // Notes
            crate::routes::notes::CreateNoteRequest,
            crate::routes::notes::UpdateNoteRequest,

            // Clipboard
            crate::routes::clipboard::CreateClipboardRequest,

            // Todos
            crate::routes::todos::CreateTodoRequest,
            crate::routes::todos::UpdateTodoRequest,

            // Bookmarks
            crate::routes::bookmarks::CreateBookmarkRequest,
            crate::routes::bookmarks::UpdateBookmarkRequest,

            // Contacts
            crate::routes::contacts::CreateContactRequest,
            crate::routes::contacts::UpdateContactRequest,

            // Credentials
            crate::routes::credentials::UnlockRequest,
            crate::routes::credentials::UnlockResponse,
            crate::routes::credentials::CreateCredentialRequest,
            crate::routes::credentials::UpdateCredentialRequest,

            // Domain Models
            domain::models::note::Note,
            domain::models::clipboard::ClipboardItem,
            domain::models::todo::Todo,
            domain::models::bookmark::Bookmark,
            domain::models::contact::Contact,
            domain::models::credential::Credential,
            domain::models::credential::EncryptedData,
            domain::models::common::TrashStatus,
            domain::models::common::ColorLabel,
            domain::models::common::Tag,
            domain::models::common::ItemMetadata,
        )
    )
)]
/// A dummy struct used specifically to auto-generate the schemas map.
struct ApiSchemas;

#[cfg(feature = "swagger")]
/// Build the complete OpenAPI v3 specification for the Personal Keeper API.
fn build_openapi() -> openapi::OpenApi {
    // Convenience: create a JSON response with a schema reference.
    let json_response = |schema_ref: RefOr<openapi::schema::Schema>| -> openapi::response::Response {
        ResponseBuilder::new()
            .description("Successful response")
            .content("application/json", ContentBuilder::new().schema(schema_ref).build())
            .build()
    };

    let json_array_response = |schema_ref: RefOr<openapi::schema::Schema>| -> openapi::response::Response {
        ResponseBuilder::new()
            .description("Successful response")
            .content(
                "application/json",
                ContentBuilder::new()
                    .schema(ArrayBuilder::new().items(schema_ref).build())
                    .build(),
            )
            .build()
    };

    let no_content_response = || -> openapi::response::Response {
        ResponseBuilder::new()
            .description("Resource deleted successfully")
            .build()
    };

    let error_response = || -> openapi::response::Response {
        ResponseBuilder::new()
            .description("An unexpected error occurred")
            .content(
                "application/json",
                ContentBuilder::new()
                    .schema(ObjectBuilder::new().build())
                    .build(),
            )
            .build()
    };

    let string_schema = || -> openapi::schema::Schema {
        ObjectBuilder::new().schema_type(SchemaType::String).build().into()
    };

    let path_param_id = || {
        ParameterBuilder::new()
            .name("id")
            .parameter_in(ParameterIn::Path)
            .required(openapi::Required::True)
            .schema(Some(string_schema()))
            .build()
    };

    let query_param_search = || {
        ParameterBuilder::new()
            .name("search")
            .parameter_in(ParameterIn::Query)
            .required(openapi::Required::False)
            .schema(Some(string_schema()))
            .build()
    };

    let paths = PathsBuilder::new()
        // ---------- Health ----------
        .path(
            "/health",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Get,
                    OperationBuilder::new()
                        .summary(Some("Service health check".to_string()))
                        .tag("Health")
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(ObjectBuilder::new().build().into()))
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        // ---------- Auth ----------
        .path(
            "/api/auth/register",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Post,
                    OperationBuilder::new()
                        .summary(Some("Register a new user".to_string()))
                        .tag("Auth")
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("RegisterRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("201", json_response(openapi::Ref::from_schema_name("AuthResponse").into()))
                                .response("400", ResponseBuilder::new().description("Bad Request").build())
                                .response("409", ResponseBuilder::new().description("Username already exists").build())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .path(
            "/api/auth/login",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Post,
                    OperationBuilder::new()
                        .summary(Some("Log in with username and password".to_string()))
                        .tag("Auth")
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("LoginRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(openapi::Ref::from_schema_name("AuthResponse").into()))
                                .response("401", ResponseBuilder::new().description("Invalid credentials").build())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .path(
            "/api/auth/refresh",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Post,
                    OperationBuilder::new()
                        .summary(Some("Refresh access token".to_string()))
                        .tag("Auth")
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("RefreshRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(openapi::Ref::from_schema_name("AuthResponse").into()))
                                .response("401", ResponseBuilder::new().description("Invalid refresh token").build())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .path(
            "/api/auth/me",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Get,
                    OperationBuilder::new()
                        .summary(Some("Get current user info".to_string()))
                        .tag("Auth")
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(ObjectBuilder::new().build().into()))
                                .response("401", ResponseBuilder::new().description("Not authenticated").build())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        // ---------- Notes ----------
        .path(
            "/api/notes",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Post,
                    OperationBuilder::new()
                        .summary(Some("Create a new note".to_string()))
                        .tag("Notes")
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("CreateNoteRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("201", json_response(openapi::Ref::from_schema_name("Note").into()))
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .operation(
                    PathItemType::Get,
                    OperationBuilder::new()
                        .summary(Some("List all notes".to_string()))
                        .tag("Notes")
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_array_response(openapi::Ref::from_schema_name("Note").into()))
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .path(
            "/api/notes/reorder",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Put,
                    OperationBuilder::new()
                        .summary(Some("Reorder notes".to_string()))
                        .tag("Notes")
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("ReorderRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(ObjectBuilder::new().build().into()))
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .path(
            "/api/notes/{id}",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Put,
                    OperationBuilder::new()
                        .summary(Some("Update a note".to_string()))
                        .tag("Notes")
                        .parameter(path_param_id())
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("UpdateNoteRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(openapi::Ref::from_schema_name("Note").into()))
                                .response("404", ResponseBuilder::new().description("Note not found").build())
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .operation(
                    PathItemType::Delete,
                    OperationBuilder::new()
                        .summary(Some("Delete a note".to_string()))
                        .tag("Notes")
                        .parameter(path_param_id())
                        .responses(
                            ResponsesBuilder::new()
                                .response("204", no_content_response())
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        // ---------- Clipboard ----------
        .path(
            "/api/clipboard",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Post,
                    OperationBuilder::new()
                        .summary(Some("Create a clipboard item".to_string()))
                        .tag("Clipboard")
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("CreateClipboardRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("201", json_response(openapi::Ref::from_schema_name("ClipboardItem").into()))
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .operation(
                    PathItemType::Get,
                    OperationBuilder::new()
                        .summary(Some("List clipboard items (optional search)".to_string()))
                        .tag("Clipboard")
                        .parameter(query_param_search())
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_array_response(openapi::Ref::from_schema_name("ClipboardItem").into()))
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .path(
            "/api/clipboard/reorder",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Put,
                    OperationBuilder::new()
                        .summary(Some("Reorder clipboard items".to_string()))
                        .tag("Clipboard")
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("ReorderRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(ObjectBuilder::new().build().into()))
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .path(
            "/api/clipboard/{id}",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Delete,
                    OperationBuilder::new()
                        .summary(Some("Delete a clipboard item".to_string()))
                        .tag("Clipboard")
                        .parameter(path_param_id())
                        .responses(
                            ResponsesBuilder::new()
                                .response("204", no_content_response())
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        // ---------- Todos ----------
        .path(
            "/api/todos",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Post,
                    OperationBuilder::new()
                        .summary(Some("Create a todo".to_string()))
                        .tag("Todos")
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("CreateTodoRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("201", json_response(openapi::Ref::from_schema_name("Todo").into()))
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .operation(
                    PathItemType::Get,
                    OperationBuilder::new()
                        .summary(Some("List todos (optional search)".to_string()))
                        .tag("Todos")
                        .parameter(query_param_search())
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_array_response(openapi::Ref::from_schema_name("Todo").into()))
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .path(
            "/api/todos/reorder",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Put,
                    OperationBuilder::new()
                        .summary(Some("Reorder todos".to_string()))
                        .tag("Todos")
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("ReorderRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(ObjectBuilder::new().build().into()))
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .path(
            "/api/todos/{id}",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Put,
                    OperationBuilder::new()
                        .summary(Some("Update a todo".to_string()))
                        .tag("Todos")
                        .parameter(path_param_id())
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("UpdateTodoRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(openapi::Ref::from_schema_name("Todo").into()))
                                .response("404", ResponseBuilder::new().description("Todo not found").build())
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .operation(
                    PathItemType::Delete,
                    OperationBuilder::new()
                        .summary(Some("Delete a todo".to_string()))
                        .tag("Todos")
                        .parameter(path_param_id())
                        .responses(
                            ResponsesBuilder::new()
                                .response("204", no_content_response())
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        // ---------- Bookmarks ----------
        .path(
            "/api/bookmarks",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Post,
                    OperationBuilder::new()
                        .summary(Some("Create a bookmark".to_string()))
                        .tag("Bookmarks")
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("CreateBookmarkRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("201", json_response(openapi::Ref::from_schema_name("Bookmark").into()))
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .operation(
                    PathItemType::Get,
                    OperationBuilder::new()
                        .summary(Some("List bookmarks (optional search)".to_string()))
                        .tag("Bookmarks")
                        .parameter(query_param_search())
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_array_response(openapi::Ref::from_schema_name("Bookmark").into()))
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .path(
            "/api/bookmarks/reorder",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Put,
                    OperationBuilder::new()
                        .summary(Some("Reorder bookmarks".to_string()))
                        .tag("Bookmarks")
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("ReorderRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(ObjectBuilder::new().build().into()))
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .path(
            "/api/bookmarks/{id}",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Put,
                    OperationBuilder::new()
                        .summary(Some("Update a bookmark".to_string()))
                        .tag("Bookmarks")
                        .parameter(path_param_id())
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("UpdateBookmarkRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(openapi::Ref::from_schema_name("Bookmark").into()))
                                .response("404", ResponseBuilder::new().description("Bookmark not found").build())
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .operation(
                    PathItemType::Delete,
                    OperationBuilder::new()
                        .summary(Some("Delete a bookmark".to_string()))
                        .tag("Bookmarks")
                        .parameter(path_param_id())
                        .responses(
                            ResponsesBuilder::new()
                                .response("204", no_content_response())
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        // ---------- Contacts ----------
        .path(
            "/api/contacts",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Post,
                    OperationBuilder::new()
                        .summary(Some("Create a contact".to_string()))
                        .tag("Contacts")
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("CreateContactRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("201", json_response(openapi::Ref::from_schema_name("Contact").into()))
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .operation(
                    PathItemType::Get,
                    OperationBuilder::new()
                        .summary(Some("List contacts (optional search)".to_string()))
                        .tag("Contacts")
                        .parameter(query_param_search())
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_array_response(openapi::Ref::from_schema_name("Contact").into()))
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .path(
            "/api/contacts/reorder",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Put,
                    OperationBuilder::new()
                        .summary(Some("Reorder contacts".to_string()))
                        .tag("Contacts")
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("ReorderRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(ObjectBuilder::new().build().into()))
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .path(
            "/api/contacts/{id}",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Put,
                    OperationBuilder::new()
                        .summary(Some("Update a contact".to_string()))
                        .tag("Contacts")
                        .parameter(path_param_id())
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("UpdateContactRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(openapi::Ref::from_schema_name("Contact").into()))
                                .response("404", ResponseBuilder::new().description("Contact not found").build())
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .operation(
                    PathItemType::Delete,
                    OperationBuilder::new()
                        .summary(Some("Delete a contact".to_string()))
                        .tag("Contacts")
                        .parameter(path_param_id())
                        .responses(
                            ResponsesBuilder::new()
                                .response("204", no_content_response())
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        // ---------- Credentials ----------
        .path(
            "/api/credentials/status",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Get,
                    OperationBuilder::new()
                        .summary(Some("Check vault configuration status".to_string()))
                        .tag("Credentials")
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(ObjectBuilder::new().build().into()))
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .path(
            "/api/credentials/unlock",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Post,
                    OperationBuilder::new()
                        .summary(Some("Unlock vault or set master password".to_string()))
                        .tag("Credentials")
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("UnlockRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(openapi::Ref::from_schema_name("UnlockResponse").into()))
                                .response("401", ResponseBuilder::new().description("Invalid master password").build())
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .path(
            "/api/credentials/lock",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Post,
                    OperationBuilder::new()
                        .summary(Some("Lock the vault".to_string()))
                        .tag("Credentials")
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(ObjectBuilder::new().build().into()))
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .path(
            "/api/credentials",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Post,
                    OperationBuilder::new()
                        .summary(Some("Create a new credential (vault must be unlocked)".to_string()))
                        .tag("Credentials")
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("CreateCredentialRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("201", json_response(openapi::Ref::from_schema_name("Credential").into()))
                                .response("401", ResponseBuilder::new().description("Vault locked").build())
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .operation(
                    PathItemType::Get,
                    OperationBuilder::new()
                        .summary(Some("List all credentials (no decrypted secrets)".to_string()))
                        .tag("Credentials")
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_array_response(openapi::Ref::from_schema_name("Credential").into()))
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .path(
            "/api/credentials/reorder",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Put,
                    OperationBuilder::new()
                        .summary(Some("Reorder credentials".to_string()))
                        .tag("Credentials")
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("ReorderRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(ObjectBuilder::new().build().into()))
                                .response("401", ResponseBuilder::new().description("Vault locked").build())
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .path(
            "/api/credentials/{id}",
            PathItemBuilder::new()
                .operation(
                    PathItemType::Get,
                    OperationBuilder::new()
                        .summary(Some("Get a single credential with decrypted fields (requires unlock)".to_string()))
                        .tag("Credentials")
                        .parameter(path_param_id())
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(ObjectBuilder::new().build().into()))
                                .response("401", ResponseBuilder::new().description("Vault locked").build())
                                .response("404", ResponseBuilder::new().description("Credential not found").build())
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .operation(
                    PathItemType::Put,
                    OperationBuilder::new()
                        .summary(Some("Update a credential".to_string()))
                        .tag("Credentials")
                        .parameter(path_param_id())
                        .request_body(Some(
                            RequestBodyBuilder::new()
                                .content(
                                    "application/json",
                                    ContentBuilder::new()
                                        .schema(openapi::Ref::from_schema_name("UpdateCredentialRequest"))
                                        .build(),
                                )
                                .build(),
                        ))
                        .responses(
                            ResponsesBuilder::new()
                                .response("200", json_response(openapi::Ref::from_schema_name("Credential").into()))
                                .response("401", ResponseBuilder::new().description("Vault locked").build())
                                .response("404", ResponseBuilder::new().description("Credential not found").build())
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .operation(
                    PathItemType::Delete,
                    OperationBuilder::new()
                        .summary(Some("Delete a credential".to_string()))
                        .tag("Credentials")
                        .parameter(path_param_id())
                        .responses(
                            ResponsesBuilder::new()
                                .response("204", no_content_response())
                                .response("401", ResponseBuilder::new().description("Vault locked").build())
                                .response("500", error_response())
                                .build()
                        )
                        .build(),
                )
                .build(),
        )
        .build();

    // Build the manually constructed OpenAPI object
    let mut api = OpenApiBuilder::new()
        .info(
            InfoBuilder::new()
                .title("Personal Keeper API")
                .version("1.0.0")
                .description(Some("Self-hosted personal knowledge base and vault".to_string()))
                .build()
        )
        .paths(paths)
        .build();

    // Dynamically pull and assign all automatically derived components (schemas) from ApiSchemas
    let generated_schemas = ApiSchemas::openapi();
    api.components = generated_schemas.components;

    api
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
/// Returns the Swagger UI service.
pub fn swagger_ui_service() -> utoipa_swagger_ui::SwaggerUi {
    use utoipa_swagger_ui::SwaggerUi;

    SwaggerUi::new("/docs/{_:.*}")
        .url("/docs/openapi.json", ApiDoc::openapi())
}