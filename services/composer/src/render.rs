use crate::AppState;
use actix_web::{web, HttpRequest, HttpResponse};

pub async fn render_page(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    let path = req.path();
    
    log::debug!("Render request: {}", path);

    let pages = state.pages.lock().unwrap();
    
    match pages.get(path) {
        Some(page_config) => {
            // Minimal rendering: return simple HTML with page info
            let html = format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
</head>
<body>
    <h1>Page: {}</h1>
    <p>Template: {}</p>
    <p>RFA: {}</p>
    <pre>{}</pre>
</body>
</html>"#,
                page_config.page_id,
                page_config.page_id,
                page_config.template,
                page_config.rfa,
                serde_json::to_string_pretty(&page_config.defaults).unwrap_or_default()
            );
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
        None => {
            log::warn!("Page not found: {}", path);
            HttpResponse::NotFound()
                .content_type("text/html; charset=utf-8")
                .body("<h1>404 - Page not found</h1>")
        }
    }
}
