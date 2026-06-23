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

    match &page_config.delivery {
        page::PageDelivery::Composer => {
            validate_rfa_exists(state, &page_config)?;
            let rendered = render_rfa(state, req, &page_config, &resolved_page_config).await?;
            Ok(ok_response(&page_config, &resolved_page_config, rendered))
        }
        page::PageDelivery::UpstreamProxy { origin, markers } => {
            let rendered = render_proxy_page(
                state,
                req,
                &page_config,
                &resolved_page_config,
                origin,
                markers,
            )
            .await?;
            Ok(ok_response(&page_config, &resolved_page_config, rendered))
        }
    }
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

async fn render_proxy_page(
    state: &web::Data<AppState>,
    req: &HttpRequest,
    page_config: &page::PageConfig,
    resolved_page_config: &experiment::ResolvedPageConfig,
    origin: &str,
    markers: &[page::ProxyMarkerReplacement],
) -> Result<String, HttpResponse> {
    let upstream_body = fetch_upstream(origin, req)
        .await
        .map_err(|err| upstream_failed(&err.to_string()))?;

    if markers.is_empty() {
        return Ok(upstream_body);
    }

    let context = contextloader::build_context(&page_config.data, req);
    let mut rendered_body = upstream_body;

    for marker in markers {
        validate_marker_rfa_exists(state, marker)?;
        let replacement = state
            .render_pool
            .render(
                &marker.rfa,
                &context,
                &resolved_page_config.rfa_replacements,
            )
            .await
            .map_err(|err| rfa_execution_failed(&err.to_string()))?;

        rendered_body = replace_marker_region(&rendered_body, &marker.id, &replacement);
    }

    Ok(rendered_body)
}

async fn fetch_upstream(origin: &str, req: &HttpRequest) -> Result<String, reqwest::Error> {
    let target = upstream_target(origin, req);
    reqwest::get(target).await?.text().await
}

fn upstream_target(origin: &str, req: &HttpRequest) -> String {
    let origin = origin.trim_end_matches('/');
    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or_else(|| req.path());

    format!("{origin}{path_and_query}")
}

fn validate_marker_rfa_exists(
    state: &web::Data<AppState>,
    marker: &page::ProxyMarkerReplacement,
) -> Result<(), HttpResponse> {
    page::resolve_rfa(state, &marker.rfa)
        .map(|_| ())
        .ok_or_else(|| missing_rfa(&marker.rfa))
}

fn replace_marker_region(body: &str, marker_id: &str, replacement: &str) -> String {
    let start_marker = format!("<!-- protopipe:marker {marker_id} -->");
    let end_marker = format!("<!-- /protopipe:marker {marker_id} -->");

    let Some(start_index) = body.find(&start_marker) else {
        return body.to_string();
    };

    let content_start = start_index + start_marker.len();
    let Some(relative_end_index) = body[content_start..].find(&end_marker) else {
        return body.to_string();
    };

    let end_index = content_start + relative_end_index + end_marker.len();

    format!(
        "{}{}{}",
        &body[..start_index],
        replacement,
        &body[end_index..]
    )
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

fn upstream_failed(message: &str) -> HttpResponse {
    log::error!("Upstream proxy request failed: {}", message);
    internal_error("Upstream proxy request failed", message)
}

fn internal_error(prefix: &str, message: &str) -> HttpResponse {
    HttpResponse::InternalServerError()
        .content_type("text/plain; charset=utf-8")
        .body(format!("{prefix}: {message}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_region_is_replaced() {
        let body = "<h1>Checkout</h1><!-- protopipe:marker checkout.summary --><section>Legacy</section><!-- /protopipe:marker checkout.summary -->";

        let rendered =
            replace_marker_region(body, "checkout.summary", "<section>Composer</section>");

        assert!(rendered.contains("<section>Composer</section>"));
        assert!(!rendered.contains("<section>Legacy</section>"));
    }

    #[test]
    fn missing_marker_passes_body_through() {
        let body = "<h1>Checkout</h1>";

        assert_eq!(
            replace_marker_region(body, "checkout.summary", "<section>Composer</section>"),
            body
        );
    }
}
