use actix_web::{web, HttpResponse};
use domain::traits::repository::Repository;
use crate::AppState;
use crate::error::ApiError;
use crate::middleware::auth::AuthUser;

#[derive(serde::Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(serde::Serialize)]
pub struct SearchResultDto {
    pub vault: String,
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub url: Option<String>,
}

pub async fn search(
    user: AuthUser,
    data: web::Data<AppState>,
    query: web::Query<SearchQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = match &query.q {
        Some(term) if !term.trim().is_empty() => term.trim(),
        _ => return Ok(HttpResponse::Ok().json(Vec::<SearchResultDto>::new())),
    };

    let (notes, clipboard, todos, bookmarks, contacts) = tokio::try_join!(
        data.notes_repo.search(&user.user_id, q),
        data.clipboard_repo.search(&user.user_id, q),
        data.todo_repo.search(&user.user_id, q),
        data.bookmark_repo.search(&user.user_id, q),
        data.contact_repo.search(&user.user_id, q),
    )?;

    let mut results = Vec::new();

    for n in notes {
        let subtitle = if n.content.len() > 100 {
            let mut end = 100;
            while !n.content.is_char_boundary(end) && end > 0 {
                end -= 1;
            }
            format!("{}…", &n.content[..end])
        } else {
            n.content
        };
        results.push(SearchResultDto {
            vault: "Notes".to_string(),
            id: n.meta.id.to_string(),
            title: n.title,
            subtitle,
            url: None,
        });
    }

    for c in clipboard {
        let first_line = c.content.lines().next().unwrap_or("").trim();
        let title = if first_line.len() > 80 {
            let mut end = 80;
            while !first_line.is_char_boundary(end) && end > 0 {
                end -= 1;
            }
            format!("{}…", &first_line[..end])
        } else {
            first_line.to_string()
        };
        results.push(SearchResultDto {
            vault: "Clipboard".to_string(),
            id: c.meta.id.to_string(),
            title,
            subtitle: String::new(),
            url: None,
        });
    }

    for t in todos {
        results.push(SearchResultDto {
            vault: "Todos".to_string(),
            id: t.meta.id.to_string(),
            title: t.title,
            subtitle: if t.completed { "Completed".to_string() } else { "Pending".to_string() },
            url: None,
        });
    }

    for b in bookmarks {
        let title = if b.title.trim().is_empty() {
            b.url.clone()
        } else {
            b.title
        };
        results.push(SearchResultDto {
            vault: "Bookmarks".to_string(),
            id: b.meta.id.to_string(),
            title,
            subtitle: b.url.clone(),
            url: Some(b.url),
        });
    }

    for c in contacts {
        let mut summary = Vec::new();
        if !c.phones.is_empty() {
            summary.push(c.phones.join(", "));
        }
        if !c.emails.is_empty() {
            summary.push(c.emails.join(", "));
        }
        results.push(SearchResultDto {
            vault: "Contacts".to_string(),
            id: c.meta.id.to_string(),
            title: c.name,
            subtitle: summary.join(" · "),
            url: None,
        });
    }

    Ok(HttpResponse::Ok().json(results))
}
