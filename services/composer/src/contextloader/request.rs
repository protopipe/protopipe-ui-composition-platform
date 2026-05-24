use actix_web::HttpRequest;
use serde_json::{Map, Value};

pub struct RequestContext {
    path: String,
    query_params: Value,
}

impl RequestContext {
    pub fn from_request(req: &HttpRequest) -> Self {
        Self {
            path: req.path().to_string(),
            query_params: query_params(req.query_string()),
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn query_value(&self, name: &str) -> Option<String> {
        match self.query_params.get(name) {
            Some(Value::String(value)) => Some(value.clone()),
            Some(Value::Number(value)) => Some(value.to_string()),
            Some(Value::Bool(value)) => Some(value.to_string()),
            _ => None,
        }
    }
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
}
