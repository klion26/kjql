use crate::{
    array_walker::array_walker,
    range_selector::range_selector,
    types::{Display, Selection, Selector, Selectors},
};
use serde_json::{json, Value};

fn apply_selector(
    inner_json: &Value,
    map_index: usize,
    raw_selector: &str,
    selectors: &Selectors,
) -> Result<Value, String> {
    // No JSON value has been found.
    if inner_json.get(raw_selector).is_none() {
        if map_index == 0 {
            Err([
                r#"Node ""#,
                raw_selector,
                r#"" not found on the parent element"#,
            ]
            .join(""))
        } else {
            Err([
                r#"Node ""#,
                raw_selector,
                r#"" not found on parent "#,
                &selectors[map_index - 1].as_str(false),
            ]
            .join(""))
        }
    } else {
        // Default case
        Ok(inner_json[raw_selector].clone())
    }
}

// returns a selection based on selectors and some JSON content.
pub fn get_selections(selectors: &Selectors, json: &Value) -> Selection {
    // local copy of the origin json that will be reused in the loop.
    let mut inner_json = json.clone();
    selectors
        .iter()
        .enumerate()
        .map(|(map_index, current_selector)| -> Result<Value, String> {
            match current_selector {
                // Object selector.
                Selector::Object(properties) => properties.iter().fold(
                    Ok(json!({})),
                    |acc: Result<Value, String>, property| {
                        println!("{}", property);
                        let value = apply_selector(
                            &inner_json,
                            map_index,
                            property,
                            selectors,
                        );
                        match value {
                            Ok(value) => match acc {
                                Ok(mut current) => {
                                    // get the associated mutable Map and insert
                                    // the property.
                                    current
                                        .as_object_mut()
                                        .unwrap()
                                        .insert(property.clone(), value);
                                    Ok(current)
                                }
                                Err(error) => Err(error),
                            },
                            Err(error) => Err(error),
                        }
                    },
                ),
                // Default selector
                Selector::Default(raw_selector) => {
                    match apply_selector(
                        &inner_json,
                        map_index,
                        raw_selector,
                        selectors,
                    ) {
                        Ok(json) => {
                            inner_json = json.clone();
                            Ok(json.clone())
                        }
                        Err(error) => Err(error),
                    }
                }

                // range selector
                Selector::Range((start, end)) => match range_selector(
                    map_index,
                    &inner_json.clone(),
                    *start,
                    *end,
                    &selectors,
                    if map_index == 0 {
                        None
                    } else {
                        Some(&selectors[map_index - 1])
                    },
                ) {
                    Ok(json) => {
                        inner_json = json.clone();
                        Ok(json.clone())
                    }
                    Err(error) => Err(error),
                },

                // Array selector.
                Selector::Array => match range_selector(
                    map_index,
                    &inner_json.clone(),
                    Some(0),
                    None,
                    &selectors,
                    if map_index == 0 {
                        None
                    } else {
                        Some(&selectors[map_index - 1])
                    },
                ) {
                    Ok(json) => {
                        inner_json = json.clone();
                        Ok(json.clone())
                    }
                    Err(error) => Err(error),
                },

                // Index selector
                Selector::Index(array_index) => match array_walker(
                    map_index,
                    &array_index,
                    &inner_json,
                    &selectors,
                ) {
                    Ok(json) => {
                        inner_json = json.clone();
                        Ok(json.clone())
                    }
                    Err(error) => Err(error),
                },
            }
        })
        .collect()
}
