use serde_json::{json, Map, Value};

// Truncate a JSON value.
pub fn truncate_json(mut value: Value) -> Value {
    // Closure that returns the primitive of a given value.
    let to_primitive = |value: &Value| match value {
        _ if value.is_array() => json!([]),
        _ if value.is_object() => json!({}),
        _ => value.to_owned(),
    };

    match value {
        _ if value.is_array() => value
            .as_array_mut()
            .unwrap()
            .iter()
            .map(|element| to_primitive(element))
            .collect::<Value>(),
        _ if value.is_object() => {
            Value::Object(value.as_object().unwrap().iter().fold(
                Map::new(),
                |mut acc, property| {
                    acc.insert(
                        property.0.to_string(),
                        to_primitive(property.1),
                    );
                    acc
                },
            ))
        }
        _ => value,
    }
}
