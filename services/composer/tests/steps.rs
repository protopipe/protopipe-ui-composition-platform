use cucumber::{given, when, then, gherkin::Step as GherkinStep};
use std::collections::HashMap;

#[derive(cucumber::World, Debug)]
#[world(init = ComposerWorld::new)]
pub struct ComposerWorld {
    pub client: Option<reqwest::Client>,
    pub base_url: String,
    pub admin_url: String,
    pub last_response: Option<String>,
    pub last_status: Option<u16>,
    pub page_config: HashMap<String, String>,
}

impl ComposerWorld {
    pub fn new() -> Self {
        Self {
            client: Some(reqwest::Client::new()),
            base_url: "http://localhost".to_string(),
            admin_url: "http://localhost".to_string(),
            last_response: None,
            last_status: None,
            page_config: HashMap::new(),
        }
    }
}

#[given(regex = r"^I register a page config:$")]
async fn register_page_config(world: &mut ComposerWorld, step: &GherkinStep) {
    let docstring = step
        .docstring()
        .expect("Expected docstring for page config");

    let payload: serde_json::Value = serde_json::from_str(docstring)
        .expect("Invalid JSON in page config docstring");

    let url = format!("{}:9000/admin/config/pages", world.admin_url);
    log::info!("Registering page config at: {}", url);
    log::debug!("Payload: {}", serde_json::to_string_pretty(&payload).unwrap());

    let client = reqwest::Client::new();
    let response = client
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
                "Failed to register page config. Status: {}, Response: {}",
                status,
                body
            );
        }
        Err(e) => {
            panic!(
                "Failed to register page config at {}. Error: {}",
                url, e
            );
        }
    }
}

#[when(regex = r#"^I register an RFA \"([^\"]+)\":$"#)]
async fn register_rfa(world: &mut ComposerWorld, id: String, step: &GherkinStep) {
    let docstring = step
        .docstring()
        .expect("Expected docstring for RFA source");

    let payload = serde_json::json!({
        "id": id,
        "source": docstring,
        "version": "1.0.0"
    });

    let url = format!("{}:9000/admin/rfa/register", world.admin_url);
    log::info!("Registering RFA '{}' at: {}", id, url);
    log::debug!("Payload: {}", serde_json::to_string_pretty(&payload).unwrap());

    let client = reqwest::Client::new();
    let response = client
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
                "Failed to register RFA '{}'. Status: {}, Response: {}",
                id,
                status,
                body
            );
        }
        Err(e) => {
            panic!(
                "Failed to register RFA '{}' at {}. Error: {}",
                id, url, e
            );
        }
    }
}

#[when(regex = r"^I request GET (.+)$")]
async fn request_page(world: &mut ComposerWorld, path: String) {
    let url = format!("{}:8080{}", world.base_url, path);
    log::info!("Requesting page: {}", url);

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await;

    match response {
        Ok(resp) => {
            world.last_status = Some(resp.status().as_u16());
            world.last_response = Some(resp.text().await.unwrap_or_default());
            log::debug!("Response status: {}", world.last_status.unwrap());
            log::debug!("Response body length: {}", world.last_response.as_ref().unwrap().len());
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
    let response = world
        .last_response
        .as_ref()
        .expect("No response received");

    assert!(
        response.contains(&expected_text),
        "Expected text '{}' not found in response.\n\nActual response:\n{}",
        expected_text,
        response
    );
}
