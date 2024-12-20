use rayon::prelude::*;
use serde_json::json;
use serde_json::Value;

pub fn flatten_json_array(array: &[Value]) -> Vec<Value> {
    fn deepen(value: Value) -> Value {
        let t: Vec<Value> = value
            .as_array()
            .unwrap()
            .par_iter()
            .fold(Vec::new, |mut acc: Vec<Value>, inner_value: &Value| {
                if inner_value.is_array() {
                    let deep = deepen(inner_value.clone());
                    if deep.is_array() {
                        acc.append(&mut deep.as_array().unwrap().clone());
                    } else {
                        acc.push(inner_value.clone())
                    }
                    acc
                } else {
                    acc.push(inner_value.clone());
                    acc
                }
            })
            .flatten()
            .collect();
        json!(t)
    }
    array
        .iter()
        .map(|value| {
            if value.is_array() {
                println!("{:?}", deepen(value.clone()));
                deepen(value.clone())
            } else {
                println!("nope");
                value.clone()
            }
        })
        .collect::<Vec<Value>>()
}
