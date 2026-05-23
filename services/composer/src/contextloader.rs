use crate::DataValue;
use actix_web::HttpRequest;
use serde_json::{Map, Value};
use std::collections::HashMap;

/// Build the renderer context from page data values.
///
/// Static values are passed through directly. Dynamic REST values are currently
/// represented by their default payload until the REST loader is implemented.
pub fn build_context(data: &HashMap<String, DataValue>, req: &HttpRequest) -> Value {
    let mut context = Map::new();
    let query_params = query_params(req.query_string());

    for (key, value) in data {
        let item = match value {
            DataValue::Static(static_data) => static_data.value.clone(),
            DataValue::DynamicRest(dynamic) => dynamic.default.clone(),
            DataValue::Url => Value::String(req.path().to_string()),
            DataValue::GetParameter(get_parameter) => query_params
                .get(&get_parameter.key)
                .cloned()
                .unwrap_or(Value::Null),
        };
        context.insert(key.clone(), item);
    }

    Value::Object(context)
}

fn query_params(query_string: &str) -> Value {
    let mut params = Map::new();

    for pair in query_string.split('&').filter(|pair| !pair.is_empty()) {
        let mut parts = pair.splitn(2, '=');
        let key = percent_decode(parts.next().unwrap_or_default());
        let value = percent_decode(parts.next().unwrap_or_default());

        if key.is_empty() {
            continue;
        }

        match params.get_mut(&key) {
            Some(Value::Array(values)) => values.push(Value::String(value)),
            Some(existing) => {
                let previous = existing.clone();
                *existing = Value::Array(vec![previous, Value::String(value)]);
            }
            None => {
                params.insert(key, Value::String(value));
            }
        }
    }

    Value::Object(params)
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                if let (Some(high), Some(low)) =
                    (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
                {
                    output.push(high * 16 + low);
                    index += 3;
                } else {
                    output.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }

    String::from_utf8_lossy(&output).into_owned()
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_request(uri: &str) -> HttpRequest {
        actix_web::test::TestRequest::with_uri(uri).to_http_request()
    }

    #[test]
    fn query_params_returns_single_values_as_strings() {
        let params = query_params("message=Hello&currency=EUR");

        assert_eq!(
            params,
            serde_json::json!({
                "message": "Hello",
                "currency": "EUR"
            })
        );
    }

    #[test]
    fn query_params_returns_repeated_values_as_arrays() {
        let params = query_params("tag=red&tag=blue");

        assert_eq!(
            params,
            serde_json::json!({
                "tag": ["red", "blue"]
            })
        );
    }

    #[test]
    fn query_params_decodes_percent_encoding_and_plus_spaces() {
        let params = query_params("message=Hello+World%21");

        assert_eq!(
            params,
            serde_json::json!({
                "message": "Hello World!"
            })
        );
    }

    #[test]
    fn build_context_injects_declared_url_without_query_string() {
        let mut data = HashMap::new();
        data.insert("url".to_string(), DataValue::Url);
        let request = test_request("/index.html?message=Hello");

        let context = build_context(&data, &request);

        assert_eq!(
            context,
            serde_json::json!({
                "url": "/index.html"
            })
        );
    }

    #[test]
    fn build_context_injects_declared_get_parameter() {
        let mut data = HashMap::new();
        data.insert(
            "getMessage".to_string(),
            DataValue::GetParameter(crate::page::GetParameterData {
                key: "message".to_string(),
            }),
        );
        let request = test_request("/index.html?message=Hello");

        let context = build_context(&data, &request);

        assert_eq!(
            context,
            serde_json::json!({
                "getMessage": "Hello"
            })
        );
    }

    #[test]
    fn build_context_uses_null_for_missing_get_parameter() {
        let mut data = HashMap::new();
        data.insert(
            "getMessage".to_string(),
            DataValue::GetParameter(crate::page::GetParameterData {
                key: "message".to_string(),
            }),
        );
        let request = test_request("/index.html");

        let context = build_context(&data, &request);

        assert_eq!(
            context,
            serde_json::json!({
                "getMessage": null
            })
        );
    }
}
