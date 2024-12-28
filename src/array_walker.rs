use crate::types::{Display, Selections, Selectors};
use rayon::prelude::*;
use serde_json::{json, Value};

// walks through a JSON array. Iterate over the indexes of the array, returns
// a Result of values or an Err early on.
pub fn array_walker(
    array_index: &[usize],
    inner_json: &Value,
    map_index: usize,
    selector: &Selectors,
) -> Result<Value, String> {
    let results: Selections = array_index
        .par_iter()
        .map(|index| {
            // No JSON value has been found (array).
            if inner_json.get(index).is_none() {
                let error_message = match inner_json.as_array() {
                    // Trying to access an out of bound index on a
                    // node
                    // or on the root element.
                    Some(array) => {
                        if selector.len() == 1 {
                            [
                                "Index [",
                                index.to_string().as_str(),
                                "] is out of bound, root element has a length \
                                 of ",
                                &array.len().to_string(),
                            ]
                            .join("")
                        } else {
                            [
                                "Index [",
                                index.to_string().as_str(),
                                "] is out of bound, ",
                                &selector[map_index - 1].as_str(false),
                                " has a length of ",
                                &array.len().to_string(),
                            ]
                            .join("")
                        }
                    }
                    // Trying to acces an index on a node which
                    // is not an arrya.
                    None => {
                        if selector.len() == 1 || map_index == 0 {
                            String::from("Root element is not an array")
                        } else {
                            [
                                &selector[map_index - 1].as_str(true),
                                " is not an array",
                            ]
                            .join("")
                        }
                    }
                };
                return Err(error_message);
            }

            Ok(inner_json.get(index).unwrap().clone())
        })
        .collect();

    match results {
        Ok(values) => Ok(if array_index.len() == 1 {
            values[0].clone()
        } else {
            json!(values)
        }),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Selector;

    #[test]
    fn valid_array_walker() {
        assert_eq!(
            Ok(json!("bar")),
            array_walker(
                &[1],
                &json!(["foo", "bar"]),
                1,
                &[Selector::Default("foo".to_string())]
            )
        );

        assert_eq!(
            Ok(json!(["foo", "bar"])),
            array_walker(
                &[0, 1],
                &json!(["foo", "bar"]),
                1,
                &[Selector::Default("foo".to_string())]
            )
        );
    }

    #[test]
    fn invalid_array_walker() {
        assert_eq!(
            Err(String::from(
                "Index [1] is out of bound, root element has a length of 0"
            )),
            array_walker(
                &[1],
                &json!([]),
                1,
                &[Selector::Default("foo".to_string())]
            )
        );

        assert_eq!(
            Err(String::from(
                "Index [1] is out of bound, node \"foo\" has a length of 0"
            )),
            array_walker(
                &[1],
                &json!([]),
                1,
                &[
                    Selector::Default("foo".to_string()),
                    Selector::Index([0].to_vec())
                ]
            )
        );

        assert_eq!(
            Err(String::from("Root element is not an array")),
            array_walker(
                &[1],
                &json!("foo"),
                1,
                &[Selector::Default("foo".to_string())]
            )
        );

        assert_eq!(
            Err(String::from("Node \"foo\" is not an array")),
            array_walker(
                &[1],
                &json!("foo"),
                1,
                &[
                    Selector::Default("foo".to_string()),
                    Selector::Index([0].to_vec())
                ]
            )
        );
    }
}
