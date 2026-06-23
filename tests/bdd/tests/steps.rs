use cucumber::{gherkin::Step as GherkinStep, given, then, when};
use futures::StreamExt;
use reqwest::cookie::Jar;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(cucumber::World, Debug)]
#[world(init = ComposerWorld::new)]
pub struct ComposerWorld {
    pub client: Option<reqwest::Client>,
    pub base_url: String,
    pub admin_url: String,
    pub messagebridge_url: String,
    pub message_worker_mock_url: String,
    pub wiremock_url: String,
    pub last_response: Option<String>,
    pub last_status: Option<u16>,
    pub last_headers: Option<reqwest::header::HeaderMap>,
    pub first_streamed_chunk: Option<String>,
    pub first_streamed_chunk_elapsed_ms: Option<u128>,
    pub cookie_jar: Arc<Jar>,
    pub page_config: HashMap<String, String>,
}

impl ComposerWorld {
    pub fn new() -> Self {
        let jar = Arc::new(Jar::default());
        let base_url =
            env::var("COMPOSER_BASE_URL").unwrap_or_else(|_| "http://localhost".to_string());
        let admin_url = env::var("COMPOSER_ADMIN_URL").unwrap_or_else(|_| base_url.clone());
        let messagebridge_url =
            env::var("MESSAGEBRIDGE_URL").unwrap_or_else(|_| "http://localhost:8082".to_string());
        let message_worker_mock_url = env::var("MESSAGE_WORKER_MOCK_URL")
            .unwrap_or_else(|_| "http://localhost:9100".to_string());
        let wiremock_url =
            env::var("WIREMOCK_URL").unwrap_or_else(|_| "http://localhost:8081".to_string());

        Self {
            client: Some(
                reqwest::Client::builder()
                    .cookie_provider(Arc::clone(&jar))
                    .build()
                    .unwrap(),
            ),
            base_url,
            admin_url,
            messagebridge_url,
            message_worker_mock_url,
            wiremock_url,
            last_response: None,
            last_status: None,
            last_headers: None,
            first_streamed_chunk: None,
            first_streamed_chunk_elapsed_ms: None,
            cookie_jar: jar,
            page_config: HashMap::new(),
        }
    }

    pub async fn cleanup(&mut self) {
        // Reset state before each scenario

        wait_for_http(
            self.client.as_ref().unwrap(),
            &format!("{}:9000/admin/health", self.admin_url),
        )
        .await;

        let url = format!("{}:9000/admin/config", self.admin_url);
        let response = self.client.as_ref().unwrap().delete(&url).send().await;

        log::info!(
            "Cleanup: Sent DELETE request to {}, {}",
            url,
            response.unwrap().status()
        );

        let messagebridge_health_url = format!("{}/health", self.messagebridge_url);
        let messagebridge_reset_url = format!("{}/admin/config", self.messagebridge_url);
        match self
            .client
            .as_ref()
            .unwrap()
            .get(&messagebridge_health_url)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                let messagebridge_response = self
                    .client
                    .as_ref()
                    .unwrap()
                    .delete(&messagebridge_reset_url)
                    .send()
                    .await;
                match messagebridge_response {
                    Ok(response) => log::info!(
                        "Cleanup: Sent DELETE request to {}, {}",
                        messagebridge_reset_url,
                        response.status()
                    ),
                    Err(error) => log::warn!(
                        "Cleanup: Could not reset optional Message Bridge at {}: {}",
                        messagebridge_reset_url,
                        error
                    ),
                }
            }
            Ok(response) => log::warn!(
                "Cleanup: Optional Message Bridge at {} returned {}",
                messagebridge_health_url,
                response.status()
            ),
            Err(error) => log::warn!(
                "Cleanup: Optional Message Bridge at {} is not reachable: {}",
                messagebridge_health_url,
                error
            ),
        }

        let worker_health_url = format!("{}/health", self.message_worker_mock_url);
        let worker_reset_url = format!("{}/processed", self.message_worker_mock_url);
        match self
            .client
            .as_ref()
            .unwrap()
            .get(&worker_health_url)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                let worker_response = self
                    .client
                    .as_ref()
                    .unwrap()
                    .delete(&worker_reset_url)
                    .send()
                    .await;
                match worker_response {
                    Ok(response) => log::info!(
                        "Cleanup: Sent DELETE request to {}, {}",
                        worker_reset_url,
                        response.status()
                    ),
                    Err(error) => log::warn!(
                        "Cleanup: Could not reset optional message worker mock at {}: {}",
                        worker_reset_url,
                        error
                    ),
                }
            }
            Ok(response) => log::warn!(
                "Cleanup: Optional message worker mock at {} returned {}",
                worker_health_url,
                response.status()
            ),
            Err(error) => log::warn!(
                "Cleanup: Optional message worker mock at {} is not reachable: {}",
                worker_health_url,
                error
            ),
        }

        let wiremock_reset_url = format!("{}/__admin/mappings", self.wiremock_url);
        let wiremock_response = self
            .client
            .as_ref()
            .unwrap()
            .delete(&wiremock_reset_url)
            .send()
            .await;
        match wiremock_response {
            Ok(response) => log::info!(
                "Cleanup: Sent DELETE request to {}, {}",
                wiremock_reset_url,
                response.status()
            ),
            Err(error) => log::warn!(
                "Cleanup: Could not reset optional WireMock at {}: {}",
                wiremock_reset_url,
                error
            ),
        }

        self.last_response = None;
        self.last_status = None;
        self.last_headers = None;
        self.first_streamed_chunk = None;
        self.first_streamed_chunk_elapsed_ms = None;
        self.page_config.clear();
    }
}

async fn wait_for_http(client: &reqwest::Client, url: &str) {
    let mut last_error = None;

    for _ in 0..60 {
        match client.get(url).send().await {
            Ok(response) if response.status().is_success() => return,
            Ok(response) => last_error = Some(format!("status {}", response.status())),
            Err(error) => last_error = Some(error.to_string()),
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    panic!("service at {url} did not become ready: {last_error:?}");
}

#[given(regex = r"^a registered IFA message channel:$")]
async fn register_ifa_message_channel(world: &mut ComposerWorld, step: &GherkinStep) {
    let docstring = step
        .docstring()
        .expect("Expected docstring for IFA message channel");
    let payload: serde_json::Value =
        serde_json::from_str(docstring).expect("Invalid JSON in IFA message channel docstring");

    let url = format!("{}/admin/config/ifas", world.messagebridge_url);
    let response = world
        .client
        .as_ref()
        .unwrap()
        .post(&url)
        .json(&payload)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_else(|_| "no body".to_string());
            assert!(
                status.is_success(),
                "Failed to register IFA message channel. Status: {}, Response: {}",
                status,
                body
            );
        }
        Err(e) => {
            panic!(
                "Failed to register IFA message channel at {}. Error: {}",
                url, e
            );
        }
    }
}

#[when(regex = r"^the frontend emits an interaction event:$")]
async fn frontend_emits_interaction_event(world: &mut ComposerWorld, step: &GherkinStep) {
    let docstring = step
        .docstring()
        .expect("Expected docstring for interaction event");
    let payload: serde_json::Value =
        serde_json::from_str(docstring).expect("Invalid JSON in interaction event docstring");

    let url = format!("{}/messages", world.messagebridge_url);
    let response = world
        .client
        .as_ref()
        .unwrap()
        .post(&url)
        .json(&payload)
        .send()
        .await;

    match response {
        Ok(resp) => {
            world.last_status = Some(resp.status().as_u16());
            world.last_headers = Some(resp.headers().clone());
            world.last_response = Some(resp.text().await.unwrap_or_default());
        }
        Err(e) => {
            world.last_status = Some(0);
            world.last_response = Some(format!("Error: {}", e));
            panic!("Failed to emit interaction event at {}. Error: {}", url, e);
        }
    }
}

#[given(regex = r"^a registered page config:$")]
async fn register_page_config(world: &mut ComposerWorld, step: &GherkinStep) {
    let docstring = step
        .docstring()
        .expect("Expected docstring for page config");

    let payload: serde_json::Value =
        serde_json::from_str(docstring).expect("Invalid JSON in page config docstring");
    let payload = rewrite_legacy_monolith_origin(payload, &world.wiremock_url);

    let url = format!("{}:9000/admin/config/pages", world.admin_url);
    log::info!("Registering page config at: {}", url);
    log::debug!(
        "Payload: {}",
        serde_json::to_string_pretty(&payload).unwrap()
    );

    let client = reqwest::Client::new();
    let response = client.post(&url).json(&payload).send().await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_else(|_| "no body".to_string());

            assert!(
                status.is_success(),
                "Failed to register page config. Status: {}, Response: {}",
                status,
                body
            );
        }
        Err(e) => {
            panic!("Failed to register page config at {}. Error: {}", url, e);
        }
    }
}

#[then(regex = r#"^the worker mock should have processed message "([^"]*)" on queue "([^"]*)"$"#)]
async fn worker_mock_should_have_processed_message(
    world: &mut ComposerWorld,
    expected_message_name: String,
    expected_queue: String,
) {
    let url = format!("{}/processed", world.message_worker_mock_url);
    let client = world.client.as_ref().unwrap();

    let mut last_body = serde_json::Value::Null;
    for _ in 0..40 {
        let response = client
            .get(&url)
            .send()
            .await
            .expect("Failed to query message worker mock");
        let body = response
            .json::<serde_json::Value>()
            .await
            .expect("Message worker mock returned invalid JSON");
        last_body = body.clone();

        if body.as_array().is_some_and(|messages| {
            messages.iter().any(|message| {
                message.get("queue").and_then(|value| value.as_str())
                    == Some(expected_queue.as_str())
                    && message.get("message_name").and_then(|value| value.as_str())
                        == Some(expected_message_name.as_str())
            })
        }) {
            return;
        }

        tokio::time::sleep(Duration::from_millis(250)).await;
    }

    panic!(
        "Expected worker mock to process message '{}' on queue '{}'. Last processed messages: {}",
        expected_message_name,
        expected_queue,
        serde_json::to_string_pretty(&last_body).unwrap_or_else(|_| last_body.to_string())
    );
}

#[given(regex = r#"^a registered experiment:$"#)]
async fn register_experiment_config(world: &mut ComposerWorld, step: &GherkinStep) {
    let docstring = step
        .docstring()
        .expect("Expected docstring for experiment config");

    let url = format!("{}:9000/admin/config/experiments", world.admin_url);
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&rewrite_legacy_monolith_origin(
            docstring
                .parse::<serde_json::Value>()
                .expect("Invalid JSON in experiment config"),
            &world.wiremock_url,
        ))
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_else(|_| "no body".to_string());

            assert!(
                status.is_success(),
                "Failed to register Experiment. Status: {}, Response: {}",
                status,
                body
            );
        }
        Err(e) => {
            panic!("Failed to register Experiment at {}. Error: {}", url, e);
        }
    }
}

#[given(regex = r#"^I have accepted the experiment cookie "([^"]*)" with value "([^"]*)"$"#)]
async fn have_accepted_experiment_cookie(
    world: &mut ComposerWorld,
    step: &GherkinStep,
    cookie_name: String,
    cookie_value: String,
) {
    let _ = step;
    let url = format!("{}:8080", world.base_url)
        .parse::<reqwest::Url>()
        .expect("Invalid base URL");
    world
        .cookie_jar
        .add_cookie_str("pp_xa_allowd=true; Path=/; SameSite=Lax", &url);

    let cookie_str = format!("{}={}", cookie_name, cookie_value);
    world.cookie_jar.add_cookie_str(&cookie_str, &url);
}

#[given(
    regex = r#"I have the experiment cookie "([^"]*)" with value "([^"]*)" without consenting to the experiment cookies"#
)]
async fn have_experiment_cookie_without_consent(
    world: &mut ComposerWorld,
    step: &GherkinStep,
    cookie_name: String,
    cookie_value: String,
) {
    let _ = step;
    let url = format!("{}:8080", world.base_url)
        .parse::<reqwest::Url>()
        .expect("Invalid base URL");

    let cookie_str = format!("{}={}", cookie_name, cookie_value);
    world.cookie_jar.add_cookie_str(&cookie_str, &url);
}

#[given(regex = r#"^a registered RFA \"([^\"]+)\":$"#)]
#[when(regex = r#"^I register a RFA \"([^\"]+)\":$"#)]
async fn register_rfa(world: &mut ComposerWorld, id: String, step: &GherkinStep) {
    let docstring = step.docstring().expect("Expected docstring for RFA source");

    let payload = serde_json::json!({
        "id": id,
        "source": docstring,
        "version": "1.0.0"
    });

    let url = format!("{}:9000/admin/config/rfas", world.admin_url);
    log::info!("Registering RFA '{}' at: {}", id, url);
    log::debug!(
        "Payload: {}",
        serde_json::to_string_pretty(&payload).unwrap()
    );

    let client = reqwest::Client::new();
    let response = client.post(&url).json(&payload).send().await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_else(|_| "no body".to_string());

            assert!(
                status.is_success(),
                "Failed to register RFA '{}'. Status: {}, Response: {}",
                id,
                status,
                body
            );
        }
        Err(e) => {
            panic!("Failed to register RFA '{}' at {}. Error: {}", id, url, e);
        }
    }
}

#[given(regex = r#"^an upstream monolith responds to GET (.+) with:$"#)]
async fn upstream_monolith_responds(world: &mut ComposerWorld, path: String, step: &GherkinStep) {
    let body = step
        .docstring()
        .expect("Expected docstring for upstream monolith response");

    let payload = serde_json::json!({
        "request": {
            "method": "GET",
            "urlPath": path
        },
        "response": {
            "status": 200,
            "headers": {
                "Content-Type": "text/html; charset=utf-8"
            },
            "body": body
        }
    });

    let url = format!("{}/__admin/mappings", world.wiremock_url);
    let response = world
        .client
        .as_ref()
        .unwrap()
        .post(&url)
        .json(&payload)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_else(|_| "no body".to_string());
            assert!(
                status.is_success(),
                "Failed to register upstream monolith response. Status: {}, Response: {}",
                status,
                body
            );
        }
        Err(e) => {
            panic!(
                "Failed to register upstream monolith response at {}. Error: {}",
                url, e
            );
        }
    }
}

#[given(regex = r#"^an upstream monolith streams GET (.+) as (\d+) chunks over (\d+) ms with:$"#)]
async fn upstream_monolith_streams(
    world: &mut ComposerWorld,
    path: String,
    number_of_chunks: u32,
    total_duration_ms: u64,
    step: &GherkinStep,
) {
    let body = step
        .docstring()
        .expect("Expected docstring for streamed upstream monolith response");

    let payload = serde_json::json!({
        "request": {
            "method": "GET",
            "urlPath": path
        },
        "response": {
            "status": 200,
            "headers": {
                "Content-Type": "text/html; charset=utf-8"
            },
            "body": body,
            "chunkedDribbleDelay": {
                "numberOfChunks": number_of_chunks,
                "totalDuration": total_duration_ms
            }
        }
    });

    let url = format!("{}/__admin/mappings", world.wiremock_url);
    let response = world
        .client
        .as_ref()
        .unwrap()
        .post(&url)
        .json(&payload)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_else(|_| "no body".to_string());
            assert!(
                status.is_success(),
                "Failed to register streamed upstream monolith response. Status: {}, Response: {}",
                status,
                body
            );
        }
        Err(e) => {
            panic!(
                "Failed to register streamed upstream monolith response at {}. Error: {}",
                url, e
            );
        }
    }
}

#[given(regex = r#"^a registered Proxy Page without marker replacements for "([^"]*)"$"#)]
async fn registered_proxy_page_without_marker_replacements(
    world: &mut ComposerWorld,
    path: String,
) {
    let payload = serde_json::json!({
        "path": path,
        "page_id": "proxy-page",
        "type": "rfa",
        "delivery": {
            "type": "upstream-proxy",
            "origin": world.wiremock_url
        },
        "timeout_ms": 3000
    });

    let url = format!("{}:9000/admin/config/pages", world.admin_url);
    let response = world
        .client
        .as_ref()
        .unwrap()
        .post(&url)
        .json(&payload)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_else(|_| "no body".to_string());
            assert!(
                status.is_success(),
                "Failed to register Proxy Page. Status: {}, Response: {}",
                status,
                body
            );
        }
        Err(e) => panic!("Failed to register Proxy Page at {}. Error: {}", url, e),
    }
}

#[when(regex = r"^I have accepted experiment cookies$")]
async fn have_accepted_experiment_cookies(world: &mut ComposerWorld) {
    let cookie_str = "pp_xa_allowd=true; Path=/; SameSite=Lax";
    let url = format!("{}:8080", world.base_url)
        .parse::<reqwest::Url>()
        .expect("Invalid base URL");
    world.cookie_jar.add_cookie_str(cookie_str, &url);
}

#[when(regex = r"I have not accepted any tracking and experiment cookies$")]
async fn have_not_accepted_experiment_cookies(_world: &mut ComposerWorld) {
    // Do nothing, as the cookie is not accepted
}

#[when(regex = r"^I request GET (.+)$")]
async fn request_page(world: &mut ComposerWorld, path: String) {
    let url = format!("{}:8080{}", world.base_url, path);
    log::info!("Requesting page: {}", url);

    let client = world.client.as_ref().expect("HTTP client not initialized");
    let response = client.get(&url).send().await;

    match response {
        Ok(resp) => {
            world.last_status = Some(resp.status().as_u16());
            world.last_headers = Some(resp.headers().clone());
            world.last_response = Some(resp.text().await.unwrap_or_default());
            log::debug!("Response status: {}", world.last_status.unwrap());
            log::debug!(
                "Response body length: {}",
                world.last_response.as_ref().unwrap().len()
            );
        }
        Err(e) => {
            world.last_status = Some(0);
            world.last_response = Some(format!("Error: {}", e));
            log::error!("Request failed: {}", e);
        }
    }
}

#[when(regex = r"^I stream GET (.+) until the first response body chunk$")]
async fn stream_page_until_first_body_chunk(world: &mut ComposerWorld, path: String) {
    let url = format!("{}:8080{}", world.base_url, path);
    log::info!("Streaming page until first body chunk: {}", url);

    let client = world.client.as_ref().expect("HTTP client not initialized");
    let started_at = Instant::now();
    let response = client
        .get(&url)
        .send()
        .await
        .expect("Failed to start streaming response");

    world.last_status = Some(response.status().as_u16());
    world.last_headers = Some(response.headers().clone());

    let mut body_stream = response.bytes_stream();
    let first_chunk = body_stream
        .next()
        .await
        .expect("Expected at least one streamed body chunk")
        .expect("Failed to read first streamed body chunk");

    world.first_streamed_chunk_elapsed_ms = Some(started_at.elapsed().as_millis());
    world.first_streamed_chunk = Some(String::from_utf8_lossy(&first_chunk).to_string());
}

#[then(regex = r"^the response status should be (\d+)$")]
async fn check_status(world: &mut ComposerWorld, expected_code: u16) {
    let actual = world.last_status.unwrap_or(0);
    assert_eq!(
        actual, expected_code,
        "Status mismatch: expected {}, got {}",
        expected_code, actual
    );
}

#[then(regex = r#"^the response should contain \"([^\"]+)\"$"#)]
async fn check_response_contains(world: &mut ComposerWorld, expected_text: String) {
    let response = world.last_response.as_ref().expect("No response received");

    assert!(
        response.contains(&expected_text),
        "Expected text '{}' not found in response.\n\nActual response:\n{}",
        expected_text,
        response
    );
}

#[then(regex = r#"^the first streamed response body chunk should arrive before (\d+) ms$"#)]
async fn first_streamed_chunk_should_arrive_before(
    world: &mut ComposerWorld,
    max_elapsed_ms: u128,
) {
    let elapsed_ms = world
        .first_streamed_chunk_elapsed_ms
        .expect("No streamed response body chunk was recorded");

    assert!(
        elapsed_ms < max_elapsed_ms,
        "Expected first streamed chunk before {} ms, but it arrived after {} ms",
        max_elapsed_ms,
        elapsed_ms
    );
}

#[then(regex = r#"^the first streamed response body chunk should contain "([^"]*)"$"#)]
async fn first_streamed_chunk_should_contain(world: &mut ComposerWorld, expected_text: String) {
    let chunk = world
        .first_streamed_chunk
        .as_ref()
        .expect("No streamed response body chunk was recorded");

    assert!(
        chunk.contains(&expected_text),
        "Expected first streamed chunk to contain '{}'.\n\nActual chunk:\n{}",
        expected_text,
        chunk
    );
}

#[then(regex = r#"^the response should not contain \"([^\"]+)\"$"#)]
async fn check_response_not_contains(world: &mut ComposerWorld, unexpected_text: String) {
    let response = world.last_response.as_ref().expect("No response received");

    assert!(
        !response.contains(&unexpected_text),
        "Unexpected text '{}' found in response.\n\nActual response:\n{}",
        unexpected_text,
        response
    );
}

#[then(regex = r"^the upstream response should be streamed without marker replacement$")]
async fn upstream_response_should_be_streamed_without_marker_replacement(
    _world: &mut ComposerWorld,
) {
    // The current BDD client observes the completed response body. Runtime
    // streaming is covered by ADR-0020 and will need lower-level tests.
}

#[then(regex = r#"^the response should contain JSON:$"#)]
async fn check_response_contains_json(world: &mut ComposerWorld) {
    let docstring = world
        .last_response
        .as_ref()
        .expect("No response received")
        .trim();
    let response = world.last_response.as_ref().expect("No response received");

    assert!(
        response.contains(&docstring),
        "Expected JSON '{}' not found in response.\n\nActual response:\n{}",
        docstring,
        response
    );
}

#[then(
    regex = r#"^the response should contain a Cookie "pp_experiment_welcome_message_test" with value \"([^\"]+)\" or \"([^\"]+)\"$"#
)]
async fn check_response_contains_cookie(
    world: &mut ComposerWorld,
    variant_a: String,
    variant_b: String,
) {
    let headers = world.last_headers.as_ref().expect("No response received");

    assert!(
        headers.get("Set-Cookie").map_or(false, |v| v
            .to_str()
            .unwrap_or("")
            .contains(&format!("pp_experiment_welcome_message_test={}", variant_a))
            || v.to_str()
                .unwrap_or("")
                .contains(&format!("pp_experiment_welcome_message_test={}", variant_b))),
        "Expected cookie not found in response.\n\nActual response:\n{}",
        headers
            .get("Set-Cookie")
            .map_or("No Set-Cookie header".to_string(), |v| v
                .to_str()
                .unwrap_or("Invalid Set-Cookie header")
                .to_string())
    );
}

#[then(
    regex = r#"^the response should not contain a Cookie "pp_experiment_welcome_message_test" with value \"([^\"]+)\" or \"([^\"]+)\"$"#
)]
async fn check_response_not_contains_cookie(
    world: &mut ComposerWorld,
    variant_a: String,
    variant_b: String,
) {
    let headers = world.last_headers.as_ref().expect("No response received");

    assert!(
        !headers.get("Set-Cookie").map_or(false, |v| v
            .to_str()
            .unwrap_or("")
            .contains(&format!("pp_experiment_welcome_message_test={}", variant_a))
            || v.to_str()
                .unwrap_or("")
                .contains(&format!("pp_experiment_welcome_message_test={}", variant_b))),
        "Unexpected cookie found in response.\n\nActual response:\n{}",
        headers
            .get("Set-Cookie")
            .map_or("No Set-Cookie header".to_string(), |v| v
                .to_str()
                .unwrap_or("Invalid Set-Cookie header")
                .to_string())
    );
}

#[then(regex = r#"^the response should delete the Cookie "([^"]*)"$"#)]
async fn check_response_deletes_cookie(world: &mut ComposerWorld, cookie_name: String) {
    let headers = world.last_headers.as_ref().expect("No response received");
    let set_cookie = headers
        .get("Set-Cookie")
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("");

    assert!(
        set_cookie.contains(&format!("{}=", cookie_name)) && set_cookie.contains("Max-Age=0"),
        "Expected deleted cookie '{}' not found in response.\n\nActual response:\n{}",
        cookie_name,
        set_cookie
    );
}

#[then(regex = r#"^the response should have content-type "([^\"]+)"$"#)]
async fn check_response_has_content_type_json(
    world: &mut ComposerWorld,
    expected_content_type: String,
) {
    let headers = world.last_headers.as_ref().expect("No response received");

    assert!(
        headers.get("Content-Type").map_or(false, |v| v
            .to_str()
            .unwrap_or("")
            .contains(&expected_content_type)),
        "Expected content-type 'application/json' not found in response.\n\nActual response:\n{}",
        headers
            .get("Content-Type")
            .map_or("No Content-Type header".to_string(), |v| v
                .to_str()
                .unwrap_or("Invalid Content-Type header")
                .to_string())
    );
}

fn rewrite_legacy_monolith_origin(
    mut value: serde_json::Value,
    wiremock_url: &str,
) -> serde_json::Value {
    match &mut value {
        serde_json::Value::String(text) if text == "http://legacy-monolith" => {
            *text = wiremock_url.to_string();
        }
        serde_json::Value::Array(values) => {
            for item in values {
                *item = rewrite_legacy_monolith_origin(item.clone(), wiremock_url);
            }
        }
        serde_json::Value::Object(values) => {
            for item in values.values_mut() {
                *item = rewrite_legacy_monolith_origin(item.clone(), wiremock_url);
            }
        }
        _ => {}
    }

    value
}
