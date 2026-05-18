use cucumber::{gherkin::Step as GherkinStep, given, then, when};
use reqwest::cookie::Jar;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;

#[derive(cucumber::World, Debug)]
#[world(init = ComposerWorld::new)]
pub struct ComposerWorld {
    pub client: Option<reqwest::Client>,
    pub base_url: String,
    pub admin_url: String,
    pub last_response: Option<String>,
    pub last_status: Option<u16>,
    pub last_headers: Option<reqwest::header::HeaderMap>,
    pub cookie_jar: Arc<Jar>,
    pub page_config: HashMap<String, String>,
}

impl ComposerWorld {
    pub fn new() -> Self {
        let jar = Arc::new(Jar::default());
        let base_url =
            env::var("COMPOSER_BASE_URL").unwrap_or_else(|_| "http://localhost".to_string());
        let admin_url = env::var("COMPOSER_ADMIN_URL").unwrap_or_else(|_| base_url.clone());

        Self {
            client: Some(
                reqwest::Client::builder()
                    .cookie_provider(Arc::clone(&jar))
                    .build()
                    .unwrap(),
            ),
            base_url,
            admin_url,
            last_response: None,
            last_status: None,
            last_headers: None,
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

        self.last_response = None;
        self.last_status = None;
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

#[given(regex = r"^a registered page config:$")]
async fn register_page_config(world: &mut ComposerWorld, step: &GherkinStep) {
    let docstring = step
        .docstring()
        .expect("Expected docstring for page config");

    let payload: serde_json::Value =
        serde_json::from_str(docstring).expect("Invalid JSON in page config docstring");

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

#[given(regex = r#"^a registered experiment:$"#)]
async fn register_experiment_config(world: &mut ComposerWorld, step: &GherkinStep) {
    let docstring = step
        .docstring()
        .expect("Expected docstring for experiment config");

    let url = format!("{}:9000/admin/config/experiments", world.admin_url);
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(
            &docstring
                .parse::<serde_json::Value>()
                .expect("Invalid JSON in experiment config"),
        )
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
    let cookie_str = format!("{}={}", cookie_name, cookie_value);
    let url = format!("{}:8080", world.base_url)
        .parse::<reqwest::Url>()
        .expect("Invalid base URL");
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

#[then(
    regex = r#"^the response should contain a Cookie "experiment_welcome_message_test" with value \"([^\"]+)\" or \"([^\"]+)\"$"#
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
            .contains(&format!("experiment_welcome_message_test={}", variant_a))
            || v.to_str()
                .unwrap_or("")
                .contains(&format!("experiment_welcome_message_test={}", variant_b))),
        "Expected cookie not found in response.\n\nActual response:\n{}",
        headers
            .get("Set-Cookie")
            .map_or("No Set-Cookie header".to_string(), |v| v
                .to_str()
                .unwrap_or("Invalid Set-Cookie header")
                .to_string())
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
