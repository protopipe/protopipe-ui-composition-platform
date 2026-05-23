use actix_web::{web, HttpRequest, HttpResponse};

use crate::{contextloader, experiment, page, AppState};

pub async fn render_page(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    render_request(state, req).await
}

pub async fn reset_config(state: web::Data<AppState>) {
    state.render_pool.reset_rfas();
}

async fn render_request(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let path = req.path();
    log::debug!("Render request: {}", path);

    let resolved_page_config = match resolve_page_config(&state, &req) {
        Some(resolved) => resolved,
        None => return page_not_found(path),
    };

    match render_resolved_page(&state, &req, resolved_page_config).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

fn resolve_page_config(
    state: &web::Data<AppState>,
    req: &HttpRequest,
) -> Option<experiment::ResolvedPageConfig> {
    experiment::resolve_page_config(state, req)
}

async fn render_resolved_page(
    state: &web::Data<AppState>,
    req: &HttpRequest,
    resolved_page_config: experiment::ResolvedPageConfig,
) -> Result<HttpResponse, HttpResponse> {
    let page_config = resolved_page_config.page_config.clone();

    validate_page(&page_config)?;
    validate_rfa_exists(state, &page_config)?;

    let rendered = render_rfa(state, req, &page_config, &resolved_page_config).await?;
    Ok(ok_response(&page_config, &resolved_page_config, rendered))
}

fn validate_page(page_config: &page::PageConfig) -> Result<(), HttpResponse> {
    page::validate_page_config(page_config).map_err(|message| {
        log::error!("Invalid page config for {}: {}", page_config.path, message);
        internal_error("Invalid page config", message)
    })
}

fn validate_rfa_exists(
    state: &web::Data<AppState>,
    page_config: &page::PageConfig,
) -> Result<(), HttpResponse> {
    page::resolve_rfa(state, &page_config.rfa)
        .map(|_| ())
        .ok_or_else(|| missing_rfa(&page_config.rfa))
}

async fn render_rfa(
    state: &web::Data<AppState>,
    req: &HttpRequest,
    page_config: &page::PageConfig,
    resolved_page_config: &experiment::ResolvedPageConfig,
) -> Result<String, HttpResponse> {
    let context = contextloader::build_context(&page_config.data, req);
    state
        .render_pool
        .render(
            &page_config.rfa,
            &context,
            &resolved_page_config.rfa_replacements,
        )
        .await
        .map_err(|err| rfa_execution_failed(&err.to_string()))
}

fn ok_response(
    page_config: &page::PageConfig,
    resolved_page_config: &experiment::ResolvedPageConfig,
    rendered: String,
) -> HttpResponse {
    let mut response = HttpResponse::Ok();
    response.content_type(page_config.content_type.clone());
    add_assignment_cookie(&mut response, resolved_page_config);
    response.body(rendered)
}

fn add_assignment_cookie(
    response: &mut actix_web::HttpResponseBuilder,
    resolved_page_config: &experiment::ResolvedPageConfig,
) {
    if let Some(cookie) = resolved_page_config.assignment_cookie.clone() {
        response.cookie(cookie);
    }
}

fn page_not_found(path: &str) -> HttpResponse {
    log::warn!("Page not found: {}", path);
    HttpResponse::NotFound()
        .content_type("text/html; charset=utf-8")
        .body("<h1>404 - Page not found</h1>")
}

fn missing_rfa(rfa_id: &str) -> HttpResponse {
    log::error!("RFA not found: {}", rfa_id);
    internal_error("RFA not found", rfa_id)
}

fn rfa_execution_failed(message: &str) -> HttpResponse {
    log::error!("RFA execution failed: {}", message);
    internal_error("RFA execution failed", message)
}

fn internal_error(prefix: &str, message: &str) -> HttpResponse {
    HttpResponse::InternalServerError()
        .content_type("text/plain; charset=utf-8")
        .body(format!("{prefix}: {message}"))
}
