use crate::types::{Display, Selection, Selector};
use rayon::prelude::*;
use serde_json::{json, Value};

pub fn range_selector(
    inner_json: &Value,
    start: Option<usize>,
    end: Option<usize>,
    map_index: usize,
    selectors: &[Selector],
    previous_selector: Option<&Selector>,
) -> Selection {
    match inner_json.as_array() {
        Some(json_array) => {
            if json_array.is_empty() {
                return Ok(json!([]));
            }

            let start = start.unwrap_or(0);
            let end = end.unwrap_or_else(|| json_array.len() - 1);

            let is_default = start < end;
            if json_array.len() < start || json_array.len() < (end + 1) {
                return Err(if selectors.len() == 1 {
                    [
                        "Range [",
                        start.to_string().as_str(),
                        ":",
                        end.to_string().as_str(),
                        "] is out of bound, root element has a length of ",
                        json_array.len().to_string().as_str(),
                    ]
                    .join("")
                } else {
                    [
                        "Range [",
                        start.to_string().as_str(),
                        ":",
                        end.to_string().as_str(),
                        "] is out of bound, ",
                        &selectors[map_index - 1].as_str(false),
                        " has a length of ",
                        &(json_array.len().to_string()),
                    ]
                    .join("")
                });
            }

            // what if start < 0 and end > len?
            Ok(if is_default {
                json!(json_array[start..=end])
            } else {
                // Get the normalized slice selection, i.e from end to start.
                let normalized_range_selection = json!(json_array[end..=start]);
                let reversed_range_selection: Vec<&Value> =
                    normalized_range_selection
                        .as_array()
                        .unwrap()
                        .par_iter()
                        .rev()
                        .collect();
                json!(reversed_range_selection)
            })
        }
        None => Err([
            (match previous_selector {
                Some(selector) => selector.as_str(true),
                None => String::from("Root element"),
            })
            .as_str(),
            " is not an array",
        ]
        .join("")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_array_range_selector() {
        assert_eq!(
            Ok(json!([])),
            range_selector(
                &json!([]),
                None,
                None,
                1,
                &[Selector::Default("foo".to_string()),],
                None
            )
        )
    }

    #[test]
    fn is_default_range_selector() {
        assert_eq!(
            Ok(json!(["foo", "bar"])),
            range_selector(
                &json!(["foo", "bar"]),
                None,
                None,
                1,
                &[Selector::Default("foo".to_string())],
                None
            )
        )
    }

    #[test]
    fn reversed_range_selector() {
        assert_eq!(
            Ok(json!(["bar", "foo"])),
            range_selector(
                &json!(["foo", "bar"]),
                Some(1),
                Some(0),
                1,
                &[Selector::Default("foo".to_string())],
                None
            )
        )
    }

    #[test]
    fn invalid_range_selector() {
        assert_eq!(
            Err(String::from(
                "Range [100:1] is out of bound, root element has a length of 2"
            )),
            range_selector(
                &json!(["foo", "bar"]),
                Some(100),
                None,
                1,
                &[Selector::Default("foo".to_string()),],
                None,
            )
        );
        assert_eq!(
            Err(String::from(
                "Range [0:100] is out of bound, root element has a length of 2"
            )),
            range_selector(
                &json!(["foo", "bar"]),
                None,
                Some(100),
                1,
                &[Selector::Default("foo".to_string()),],
                None,
            )
        );
        assert_eq!(
            Err(String::from(
                "Range [100:1] is out of bound, node \"foo\" has a length of 2"
            )),
            range_selector(
                &json!(["foo", "bar"]),
                Some(100),
                None,
                1,
                &[
                    Selector::Default("foo".to_string()),
                    Selector::Index([0].to_vec()),
                ],
                None,
            )
        );
    }

    #[test]
    fn none_array_range_selector() {
        assert_eq!(
            Err(String::from("Root element is not an array")),
            range_selector(
                &json!("foo"),
                None,
                None,
                1,
                &[Selector::Default("foo".to_string()),],
                None,
            )
        );
        assert_eq!(
            Err(String::from("Node \"foo\" is not an array")),
            range_selector(
                &json!("foo"),
                Some(100),
                None,
                1,
                &[Selector::Default("foo".to_string()),],
                Some(&Selector::Default("foo".to_string())),
            )
        )
    }
}
