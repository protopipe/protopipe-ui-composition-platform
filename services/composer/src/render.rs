use crate::{AppState, DataValue, StaticData};
use actix_web::{web, HttpRequest, HttpResponse};

fn get_currency(data: &std::collections::HashMap<String, DataValue>) -> Option<String> {
    match data.get("currency") {
        Some(DataValue::Static(StaticData { value })) => {
            value.as_str().map(|s| s.to_string())
        }
        _ => None,
    }
}

pub async fn render_page(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    let path = req.path();

    log::debug!("Render request: {}", path);

    let pages = state.pages.lock().unwrap();

    match pages.get(path) {
        Some(page_config) => {
            let currency = get_currency(&page_config.data)
                .unwrap_or_else(|| "N/A".to_string());
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
                currency 
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