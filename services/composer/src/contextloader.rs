use crate::DataValue;
use serde_json::{Map, Value};
use std::collections::HashMap;

/// Build the renderer context from page data values.
///
/// Static values are passed through directly. Dynamic REST values are currently
/// represented by their default payload until the REST loader is implemented.
pub fn build_context(data: &HashMap<String, DataValue>) -> Value {
    let mut context = Map::new();

    for (key, value) in data {
        let item = match value {
            DataValue::Static(static_data) => static_data.value.clone(),
            DataValue::DynamicRest(dynamic) => dynamic.default.clone(),
        };
        context.insert(key.clone(), item);
    }

    Value::Object(context)
}
