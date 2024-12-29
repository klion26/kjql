use crate::{
    array_walker::array_walker,
    range_selector::range_selector,
    types::{Display, InnerObject, Selection, Selections, Selector, Selectors},
};
use rayon::prelude::*;
use serde_json::{json, Map, Value};
use std::sync::{Arc, Mutex};

fn apply_selector(
    inner_json: &Value,
    map_index: usize,
    raw_selector: &str,
    selectors: &Selectors,
) -> Selection {
    // No JSON value has been found.
    if inner_json.get(raw_selector).is_none() {
        return if map_index == 0 {
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
        };
    }

    // Default case
    Ok(inner_json[raw_selector].clone())
}

fn object_to_vec(inner_json: &Value) -> Vec<(String, Value)> {
    // Make a mutable copy of the inner JSON.
    let mut inner_json_mut = inner_json.clone();

    inner_json_mut
        .as_object_mut()
        .unwrap()
        .to_owned()
        .into_iter()
        .collect::<Vec<(String, Value)>>()
}

// returns a selection based on selectors and some JSON content.
pub fn get_selections(selectors: &Selectors, json: &Value) -> Selections {
    // local copy of the origin json that will be reused in the loop.
    let data = Arc::new(Mutex::new(json.clone()));
    selectors
        .iter()
        .enumerate()
        .map(|(map_index, current_selector)| -> Selection {
            match current_selector {
                // Object selector.
                Selector::Object(properties) => properties
                    .par_iter()
                    .fold(
                        || Ok(json!({})),
                        |acc: Selection, property| match property {
                            InnerObject::Index(indexes) => {
                                let mut data = data.lock().unwrap();
                                let key_and_values = object_to_vec(&data);
                                let properties = key_and_values.len();
                                let last_index = properties - 1;
                                match indexes
                                    .par_iter()
                                    .find_last(|&&x| x > last_index)
                                {
                                    Some(index) => {
                                        let reference = if map_index > 0 {
                                            selectors[map_index - 1]
                                                .as_str(false)
                                        } else {
                                            "object".to_string()
                                        };

                                        Err([
                                            "Index [",
                                            index.to_string().as_str(),
                                            "] is out of bound, ",
                                            reference.as_str(),
                                            " contains ",
                                            &(properties).to_string(),
                                            if properties == 1 {
                                                " property"
                                            } else {
                                                " properties"
                                            },
                                        ]
                                        .join(""))
                                    }
                                    None => {
                                        let map = indexes.iter().fold(
                                            Map::with_capacity(indexes.len()),
                                            |mut acc, index| {
                                                acc.insert(
                                                    index.to_string(),
                                                    key_and_values[*index]
                                                        .1
                                                        .clone(),
                                                );
                                                acc
                                            },
                                        );

                                        *data = json!(map);
                                        Ok(json!(map))
                                    }
                                }
                            }

                            InnerObject::Key(key) => {
                                let data = data.lock().unwrap();
                                match apply_selector(
                                    &data, map_index, key, selectors,
                                ) {
                                    Ok(value) => match acc {
                                        Ok(mut current) => {
                                            current
                                                .as_object_mut()
                                                .unwrap()
                                                .insert(key.clone(), value);
                                            Ok(current)
                                        }
                                        Err(error) => Err(error),
                                    },
                                    Err(error) => Err(error),
                                }
                            }

                            InnerObject::Array => {
                                let data = data.lock().unwrap();
                                Ok(data.clone())
                            }

                            InnerObject::Range((start, end)) => {
                                let data = data.lock().unwrap();
                                let key_and_values = object_to_vec(&data);
                                let properties = key_and_values.len();
                                let last_index = properties - 1;
                                let start_with_default = start.unwrap_or(0);
                                let end_with_default =
                                    end.unwrap_or(last_index);
                                let is_default =
                                    start_with_default < end_with_default;
                                if start_with_default > last_index
                                    || end_with_default > last_index
                                {
                                    let reference = if map_index > 0 {
                                        selectors[map_index - 1].as_str(false)
                                    } else {
                                        "object".to_string()
                                    };

                                    return Err([
                                        "Range [",
                                        start_with_default.to_string().as_str(),
                                        ":",
                                        end_with_default.to_string().as_str(),
                                        "] is out of bound, ",
                                        reference.as_str(),
                                        " contains ",
                                        &(properties).to_string(),
                                        if properties == 1 {
                                            " property"
                                        } else {
                                            " properties"
                                        },
                                    ]
                                    .join(""));
                                }

                                let indexes = if is_default {
                                    (start_with_default..=end_with_default)
                                        .step_by(1)
                                        .collect::<Vec<usize>>()
                                } else {
                                    (end_with_default..=start_with_default)
                                        .step_by(1)
                                        .collect::<Vec<usize>>()
                                        .into_par_iter()
                                        .rev()
                                        .collect::<Vec<usize>>()
                                };

                                let map = indexes.iter().fold(
                                    Map::with_capacity(indexes.len()),
                                    |mut acc, index| {
                                        acc.insert(
                                            index.to_string(),
                                            key_and_values[*index].1.clone(),
                                        );
                                        acc
                                    },
                                );

                                Ok(json!(map))
                            }
                        },
                    )
                    .reduce(
                        || Ok(json!({})),
                        |first, second| {
                            first.and_then(|mut first| {
                                second.map(|mut second| {
                                    first.as_object_mut().unwrap().extend(
                                        second.as_object_mut().unwrap().clone(),
                                    );
                                    first
                                })
                            })
                        },
                    ),
                // Default selector
                Selector::Default(raw_selector) => {
                    let mut data = data.lock().unwrap();
                    match apply_selector(
                        &data,
                        map_index,
                        raw_selector,
                        selectors,
                    ) {
                        Ok(ref json) => {
                            *data = json.clone();
                            Ok(json.clone())
                        }
                        Err(error) => Err(error),
                    }
                }

                // range selector
                Selector::Range((start, end)) => {
                    let mut data = data.lock().unwrap();
                    match range_selector(
                        &data.clone(),
                        *start,
                        *end,
                        map_index,
                        selectors,
                        if map_index == 0 {
                            None
                        } else {
                            Some(&selectors[map_index - 1])
                        },
                    ) {
                        Ok(json) => {
                            *data = json.clone();
                            Ok(json)
                        }
                        Err(error) => Err(error),
                    }
                }

                // Array selector.
                Selector::Array => {
                    let mut data = data.lock().unwrap();
                    match range_selector(
                        &data.clone(),
                        Some(0),
                        None,
                        map_index,
                        selectors,
                        if map_index == 0 {
                            None
                        } else {
                            Some(&selectors[map_index - 1])
                        },
                    ) {
                        Ok(json) => {
                            *data = json.clone();
                            Ok(json)
                        }
                        Err(error) => Err(error),
                    }
                }

                // Index selector
                Selector::Index(array_index) => {
                    let mut data = data.lock().unwrap();
                    match array_walker(array_index, &data, map_index, selectors)
                    {
                        Ok(ref json) => {
                            *data = json.clone();
                            Ok(json.clone())
                        }
                        Err(error) => Err(error),
                    }
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_get_selection() {
        assert_eq!(
            Ok(vec![json!({"0": 10, "2": 30})]),
            get_selections(
                &[Selector::Object(vec![InnerObject::Index(vec![0, 2])])],
                &json!({ "A": 10, "B": 20, "C": 30})
            )
        );

        assert_eq!(
            Err(String::from(
                "Index [10] is out of bound, object contains 3 properties"
            )),
            get_selections(
                &[Selector::Object(vec![InnerObject::Index(vec![10])])],
                &json!({" A": 10, "B": 20, "C": 30})
            )
        );

        assert_eq!(
            Ok(vec![json!({"A": 10})]),
            get_selections(
                &[Selector::Object(vec![InnerObject::Key("A".to_string())])],
                &json!({"A": 10, "B": 20, "C": 30})
            )
        );

        assert_eq!(
            Err(String::from("Node \"D\" not found on the parent element")),
            get_selections(
                &[Selector::Object(vec![InnerObject::Key("D".to_string())])],
                &json!({"A": 10, "B": 20, "C": 30})
            )
        );

        assert_eq!(
            Ok(vec![json!({"A": 10, "B": 20, "C": 30})]),
            get_selections(
                &[Selector::Object(vec![InnerObject::Array])],
                &json!({"A": 10, "B": 20, "C": 30})
            )
        );

        assert_eq!(
            Ok(vec![json!({"0": 10, "1": 20})]),
            get_selections(
                &[Selector::Object(vec![InnerObject::Range((
                    Some(0),
                    Some(1)
                ))])],
                &json!({"A": 10, "B": 20, "C": 30})
            )
        );

        assert_eq!(
            Err(String::from(
                "Range [10:20] is out of bound, object contains 3 properties"
            )),
            get_selections(
                &[Selector::Object(vec![InnerObject::Range((
                    Some(10),
                    Some(20)
                ))])],
                &json!({"A": 10, "B": 20, "C": 30})
            )
        );

        assert_eq!(
            Ok(vec![json!({"A": 10, "2":30, "3":40})]),
            get_selections(
                &[Selector::Object(vec![
                    InnerObject::Key("A".to_string()),
                    InnerObject::Range((Some(2), Some(3)))
                ])],
                &json!({"A": 10, "B": 20, "C":30, "D": 40, "E":50})
            )
        );
    }

    #[test]
    fn test_default_get_selection() {
        assert_eq!(
            Ok(vec![json!(10)]),
            get_selections(
                &[Selector::Default("A".to_string())],
                &json!({"A": 10, "B": 20, "C": 30})
            )
        );

        assert_eq!(
            Err(String::from("Node \"D\" not found on the parent element")),
            get_selections(
                &[Selector::Default("D".to_string())],
                &json!({"A": 10, "B": 20, "C": 30})
            )
        );
    }

    #[test]
    fn test_range_get_selection() {
        assert_eq!(
            Ok(vec![json!(["B", "C", "D"])]),
            get_selections(
                &[Selector::Range((Some(1), Some(3)))],
                &json!(["A", "B", "C", "D", "E"])
            )
        );

        assert_eq!(
            Err(String::from(
                "Range [10:20] is out of bound, root element has a length of 5"
            )),
            get_selections(
                &[Selector::Range((Some(10), Some(20)))],
                &json!(["A", "B", "C", "D", "E"])
            )
        );
    }

    #[test]
    fn test_array_get_selection() {
        assert_eq!(
            Ok(vec![json!(["A", "B", "C", "D", "E"])]),
            get_selections(
                &[Selector::Array],
                &json!(["A", "B", "C", "D", "E"])
            )
        );
    }

    #[test]
    fn test_index_get_selection() {
        assert_eq!(
            Ok(vec![json!(["B", "D"])]),
            get_selections(
                &[Selector::Index([1, 3].to_vec())],
                &json!(["A", "B", "C", "D", "E"])
            )
        );

        assert_eq!(
            Err(String::from(
                "Index [10] is out of bound, root element has a length of 5"
            )),
            get_selections(
                &[Selector::Index(vec![10])],
                &json!(["A", "B", "C", "D", "E"])
            )
        );
    }

    #[test]
    fn test_get_selection() {
        assert_eq!(
            Ok(vec![json!(["C", "D"]), json!("C")]),
            get_selections(
                &[
                    Selector::Range((Some(2), Some(3))),
                    Selector::Index(vec![0])
                ],
                &json!(["A", "B", "C", "D", "E"])
            )
        );
    }
}
