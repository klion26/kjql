use rayon::prelude::*;
use serde_json::json;
use serde_json::Value;

pub fn flatten_json_array(value: &Value) -> Value {
    json!(value
        .as_array()
        .unwrap()
        .par_iter()
        .fold_with(Vec::new(), |mut acc: Vec<Value>, inner_value: &Value| {
            if inner_value.is_array() {
                let recursive_value = flatten_json_array(inner_value);
                if recursive_value.is_array() {
                    acc.append(
                        &mut recursive_value.as_array().unwrap().clone(),
                    );
                } else {
                    acc.push(inner_value.clone());
                }
                acc
            } else {
                acc.push(inner_value.clone());
                acc
            }
        })
        .flatten()
        .collect::<Vec<Value>>())
}
