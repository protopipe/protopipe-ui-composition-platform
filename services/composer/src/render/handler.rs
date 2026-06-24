use std::io;

use actix_web::{web, HttpRequest, HttpResponse};
use futures::{Stream, StreamExt};
use tokio::task::JoinHandle;
use tokio_stream::wrappers::ReceiverStream;

use crate::{contextloader, experiment, page, AppState};

pub async fn render_page(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    render_request(state, req, body).await
}

pub async fn reset_config(state: web::Data<AppState>) {
    state.render_pool.reset_rfas();
}

async fn render_request(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    let path = req.path();
    log::debug!("Render request: {}", path);

    let resolved_route_config = match resolve_route_config(&state, &req) {
        Some(resolved) => resolved,
        None => return page_not_found(path),
    };

    match resolved_route_config {
        experiment::ResolvedRouteConfig::Page(resolved_page_config) => {
            render_get_page(&state, &req, resolved_page_config).await
        }
        experiment::ResolvedRouteConfig::Submit(resolved_submit_route_config) => {
            render_post_page(&state, &body, resolved_submit_route_config).await
        }
    }
}

async fn render_get_page(
    state: &web::Data<AppState>,
    req: &HttpRequest,
    resolved_page_config: experiment::ResolvedPageConfig,
) -> HttpResponse {
    match render_resolved_page(state, req, resolved_page_config).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn render_post_page(
    state: &web::Data<AppState>,
    body: &[u8],
    resolved_submit_route_config: experiment::ResolvedSubmitRouteConfig,
) -> HttpResponse {
    let submit_route_config = resolved_submit_route_config.submit_route_config;
    let acknowledgement =
        contextloader::resolve_submit_service(state, &submit_route_config.post_service, body).await;
    let location = redirect_location(
        &submit_route_config.post_service.redirect.path,
        &acknowledgement,
    );

    let mut response = HttpResponse::SeeOther();
    response.append_header(("Location", location));
    add_assignment_cookie(
        &mut response,
        &resolved_submit_route_config.assignment_cookie,
    );
    response.finish()
}

fn resolve_route_config(
    state: &web::Data<AppState>,
    req: &HttpRequest,
) -> Option<experiment::ResolvedRouteConfig> {
    experiment::resolve_route_config(state, req)
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
            render_proxy_page_response(
                state,
                req,
                &page_config,
                &resolved_page_config,
                origin,
                markers,
            )
            .await
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
    let context = contextloader::build_context(state, &page_config.data, req).await;

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

async fn render_proxy_page_response(
    state: &web::Data<AppState>,
    req: &HttpRequest,
    page_config: &page::PageConfig,
    resolved_page_config: &experiment::ResolvedPageConfig,
    origin: &str,
    markers: &[page::ProxyMarkerReplacement],
) -> Result<HttpResponse, HttpResponse> {
    let replacement_tasks =
        start_marker_replacements(state, req, page_config, resolved_page_config, markers).await?;

    let upstream_response = fetch_upstream(origin, req)
        .await
        .map_err(|err| upstream_failed(&err.to_string()))?;

    let body_stream =
        stream_marker_replacements(upstream_response.bytes_stream(), replacement_tasks);

    let mut response = HttpResponse::Ok();
    response.content_type(page_config.content_type.clone());
    add_assignment_cookie(&mut response, &resolved_page_config.assignment_cookie);
    Ok(response.streaming(body_stream))
}

async fn start_marker_replacements(
    state: &web::Data<AppState>,
    req: &HttpRequest,
    page_config: &page::PageConfig,
    resolved_page_config: &experiment::ResolvedPageConfig,
    markers: &[page::ProxyMarkerReplacement],
) -> Result<Vec<ActiveMarkerReplacement>, HttpResponse> {
    let context = contextloader::build_context(state, &page_config.data, req).await;
    let mut replacements = Vec::new();

    for marker in markers {
        validate_marker_rfa_exists(state, marker)?;
        let render_pool = state.render_pool.clone();
        let rfa = marker.rfa.clone();
        let context = context.clone();
        let rfa_replacements = resolved_page_config.rfa_replacements.clone();

        replacements.push(ActiveMarkerReplacement {
            id: marker.id.clone(),
            task: tokio::spawn(async move {
                render_pool
                    .render(&rfa, &context, &rfa_replacements)
                    .await
                    .map_err(|err| err.to_string())
            }),
        });
    }

    Ok(replacements)
}

async fn fetch_upstream(
    origin: &str,
    req: &HttpRequest,
) -> Result<reqwest::Response, reqwest::Error> {
    let target = upstream_target(origin, req);
    reqwest::get(target).await
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

struct ActiveMarkerReplacement {
    id: String,
    task: JoinHandle<Result<String, String>>,
}

struct StreamMarkerReplacement {
    start_marker: String,
    end_marker: String,
    task: Option<JoinHandle<Result<String, String>>>,
}

fn stream_marker_replacements<S, E>(
    upstream: S,
    replacements: Vec<ActiveMarkerReplacement>,
) -> ReceiverStream<Result<web::Bytes, io::Error>>
where
    S: Stream<Item = Result<web::Bytes, E>> + Send + 'static,
    E: std::fmt::Display + Send + 'static,
{
    let (sender, receiver) = tokio::sync::mpsc::channel(16);

    tokio::spawn(async move {
        let mut upstream = Box::pin(upstream);
        let mut replacements = replacements
            .into_iter()
            .map(|replacement| StreamMarkerReplacement {
                start_marker: marker_start(&replacement.id),
                end_marker: marker_end(&replacement.id),
                task: Some(replacement.task),
            })
            .collect::<Vec<_>>();
        let mut buffer = String::new();

        while let Some(chunk) = upstream.next().await {
            match chunk {
                Ok(bytes) => buffer.push_str(&String::from_utf8_lossy(&bytes)),
                Err(err) => {
                    send_stream_error(&sender, err.to_string()).await;
                    return;
                }
            }

            if !flush_available_proxy_content(&sender, &mut buffer, &mut replacements).await {
                return;
            }
        }

        if !buffer.is_empty() {
            let _ = sender.send(Ok(web::Bytes::from(buffer))).await;
        }
    });

    ReceiverStream::new(receiver)
}

async fn flush_available_proxy_content(
    sender: &tokio::sync::mpsc::Sender<Result<web::Bytes, io::Error>>,
    buffer: &mut String,
    replacements: &mut [StreamMarkerReplacement],
) -> bool {
    loop {
        if let Some(marker_match) = find_next_marker(buffer, replacements) {
            if marker_match.start_index > 0 {
                let prefix = buffer[..marker_match.start_index].to_string();
                buffer.drain(..marker_match.start_index);
                if sender.send(Ok(web::Bytes::from(prefix))).await.is_err() {
                    return false;
                }
            }

            let marker = &mut replacements[marker_match.replacement_index];
            let content_start = marker.start_marker.len();
            let Some(relative_end_index) = buffer[content_start..].find(&marker.end_marker) else {
                return true;
            };

            let replacement_end = content_start + relative_end_index + marker.end_marker.len();
            let fallback_region = buffer[..replacement_end].to_string();
            let task = marker.task.take();

            let replacement = match task {
                Some(task) => match task.await {
                    Ok(Ok(rendered)) => rendered,
                    Ok(Err(err)) => {
                        log::warn!("Marker replacement failed: {}", err);
                        fallback_region.clone()
                    }
                    Err(err) => {
                        log::warn!("Marker replacement task failed: {}", err);
                        fallback_region.clone()
                    }
                },
                None => fallback_region.clone(),
            };

            buffer.drain(..replacement_end);
            if sender
                .send(Ok(web::Bytes::from(replacement)))
                .await
                .is_err()
            {
                return false;
            }

            continue;
        }

        let safe_len = safe_flush_len(buffer, replacements);
        if safe_len > 0 {
            let prefix = buffer[..safe_len].to_string();
            buffer.drain(..safe_len);
            if sender.send(Ok(web::Bytes::from(prefix))).await.is_err() {
                return false;
            }
        }

        return true;
    }
}

struct MarkerMatch {
    replacement_index: usize,
    start_index: usize,
}

fn find_next_marker(buffer: &str, replacements: &[StreamMarkerReplacement]) -> Option<MarkerMatch> {
    replacements
        .iter()
        .enumerate()
        .filter(|(_, replacement)| replacement.task.is_some())
        .filter_map(|(index, replacement)| {
            buffer
                .find(&replacement.start_marker)
                .map(|start_index| MarkerMatch {
                    replacement_index: index,
                    start_index,
                })
        })
        .min_by_key(|marker_match| marker_match.start_index)
}

fn safe_flush_len(buffer: &str, replacements: &[StreamMarkerReplacement]) -> usize {
    let keep_len = replacements
        .iter()
        .filter(|replacement| replacement.task.is_some())
        .map(|replacement| replacement.start_marker.len().saturating_sub(1))
        .max()
        .unwrap_or(0);

    if buffer.len() <= keep_len {
        return 0;
    }

    previous_char_boundary(buffer, buffer.len() - keep_len)
}

fn previous_char_boundary(value: &str, index: usize) -> usize {
    let mut index = index.min(value.len());
    while index > 0 && !value.is_char_boundary(index) {
        index -= 1;
    }
    index
}

async fn send_stream_error(
    sender: &tokio::sync::mpsc::Sender<Result<web::Bytes, io::Error>>,
    message: String,
) {
    let _ = sender
        .send(Err(io::Error::new(io::ErrorKind::Other, message)))
        .await;
}

fn marker_start(marker_id: &str) -> String {
    format!("<!-- protopipe:marker {marker_id} -->")
}

fn marker_end(marker_id: &str) -> String {
    format!("<!-- /protopipe:marker {marker_id} -->")
}

fn ok_response(
    page_config: &page::PageConfig,
    resolved_page_config: &experiment::ResolvedPageConfig,
    rendered: String,
) -> HttpResponse {
    let mut response = HttpResponse::Ok();
    response.content_type(page_config.content_type.clone());
    add_assignment_cookie_from_resolved_page(&mut response, resolved_page_config);
    response.body(rendered)
}

fn add_assignment_cookie_from_resolved_page(
    response: &mut actix_web::HttpResponseBuilder,
    resolved_page_config: &experiment::ResolvedPageConfig,
) {
    add_assignment_cookie(response, &resolved_page_config.assignment_cookie);
}

fn add_assignment_cookie(
    response: &mut actix_web::HttpResponseBuilder,
    assignment_cookie: &Option<actix_web::cookie::Cookie<'static>>,
) {
    if let Some(cookie) = assignment_cookie.clone() {
        response.cookie(cookie);
    }
}

fn redirect_location(base_path: &str, acknowledgement: &serde_json::Value) -> String {
    let query_pairs = ["stream", "businessKey", "partitionKey", "version"]
        .iter()
        .filter_map(|field| ack_field(acknowledgement, field).map(|value| (*field, value)))
        .map(|(field, value)| format!("{}={}", percent_encode(field), percent_encode(&value)))
        .collect::<Vec<_>>();

    if query_pairs.is_empty() {
        base_path.to_string()
    } else {
        format!("{}?{}", base_path, query_pairs.join("&"))
    }
}

fn ack_field(acknowledgement: &serde_json::Value, field: &str) -> Option<String> {
    acknowledgement.get(field).and_then(|value| match value {
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        _ => None,
    })
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
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
    use std::time::Duration;

    use futures::stream;
    use tokio::sync::oneshot;

    #[actix_rt::test]
    async fn proxy_stream_flushes_prefix_before_waiting_for_marker_replacement() {
        let (release_replacement, wait_for_release) = oneshot::channel();
        let replacement_task = tokio::spawn(async move {
            wait_for_release.await.unwrap();
            Ok("replacement".to_string())
        });
        let upstream = stream::iter(vec![Ok::<_, io::Error>(web::Bytes::from(
            "prefix <!-- protopipe:marker checkout.summary -->legacy<!-- /protopipe:marker checkout.summary --> suffix",
        ))]);

        let mut stream = stream_marker_replacements(
            upstream,
            vec![ActiveMarkerReplacement {
                id: "checkout.summary".to_string(),
                task: replacement_task,
            }],
        );

        assert_eq!(next_chunk(&mut stream).await, "prefix ");
        assert!(
            tokio::time::timeout(Duration::from_millis(50), stream.next())
                .await
                .is_err(),
            "stream should wait at the marker while replacement rendering is pending"
        );

        release_replacement.send(()).unwrap();

        assert_eq!(next_chunk(&mut stream).await, "replacement");
        assert_eq!(next_chunk(&mut stream).await, " suffix");
        assert!(stream.next().await.is_none());
    }

    #[actix_rt::test]
    async fn proxy_stream_preserves_marker_split_across_upstream_chunks() {
        let replacement_task = tokio::spawn(async { Ok("replacement".to_string()) });
        let upstream = stream::iter(vec![
            Ok::<_, io::Error>(web::Bytes::from("prefix <!-- protopipe:mar")),
            Ok::<_, io::Error>(web::Bytes::from(
                "ker checkout.summary -->legacy<!-- /protopipe:marker checkout.summary --> suffix",
            )),
        ]);

        let mut stream = stream_marker_replacements(
            upstream,
            vec![ActiveMarkerReplacement {
                id: "checkout.summary".to_string(),
                task: replacement_task,
            }],
        );

        let mut rendered = String::new();
        while let Some(chunk) = stream.next().await {
            rendered.push_str(std::str::from_utf8(&chunk.unwrap()).unwrap());
        }

        assert_eq!(rendered, "prefix replacement suffix");
    }

    #[actix_rt::test]
    async fn proxy_stream_passes_through_when_no_replacement_is_active() {
        let upstream = stream::iter(vec![Ok::<_, io::Error>(web::Bytes::from(
            "<h1>Checkout</h1>",
        ))]);

        let mut stream = stream_marker_replacements(upstream, Vec::new());

        assert_eq!(next_chunk(&mut stream).await, "<h1>Checkout</h1>");
        assert!(stream.next().await.is_none());
    }

    async fn next_chunk(stream: &mut ReceiverStream<Result<web::Bytes, io::Error>>) -> String {
        let chunk = stream.next().await.unwrap().unwrap();
        std::str::from_utf8(&chunk).unwrap().to_string()
    }
}
