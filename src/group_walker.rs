use crate::apply_filter::apply_filter;
use crate::flatten_array::flatten_array;
use crate::get_selection::get_selections;
use crate::get_selector::get_selector;
use crate::types::{MaybeArray, Selection, Selector};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{json, Value};

// walks through a group
pub fn group_walker(capture: &str, json: &Value) -> Result<Value, String> {
    lazy_static! {
        static ref FILTER_REGEX: Regex =
            Regex::new(r"^(\.{2})*(.*)\|([^|]+)$").unwrap();
        static ref SUB_GROUP_REGEX: Regex =
            Regex::new(r#"("[^"]+")|([^.]+)"#).unwrap();
    }

    let group = capture.trim();
    let parsed_group: (Option<()>, &str, Option<&str>) = FILTER_REGEX
        .captures_iter(group)
        .map(|capture| {
            println!(
                "filter === [{:?}],[{:?}], [{:?}]",
                capture.get(1),
                capture.get(2),
                capture.get(3)
            );
            (
                // Spread capture.
                capture.get(1).map(|_| ()),
                // Group capture.
                capture.get(2).map_or("", |m| m.as_str()),
                // filter capture.
                capture.get(3).map(|m| m.as_str()),
            )
        })
        .next()
        .unwrap_or((None, group, None));

    // empty group, return early
    if parsed_group.1.is_empty() {
        return Err(String::from("Empty group"));
    }

    // capture sub-groups of doulbe quoted selectors and simple ones surrounded
    // by dots.
    let selectors: Vec<Selector> = SUB_GROUP_REGEX
        .captures_iter(parsed_group.1)
        .map(|capture| get_selector(capture.get(0).map_or("", |m| m.as_str())))
        .collect();

    // perform the same operation on the filter.
    let filter_selectors = parsed_group.2.map(|filter| {
        SUB_GROUP_REGEX
            .captures_iter(filter)
            .map(|capture| {
                get_selector(capture.get(0).map_or("", |m| m.as_str()))
            })
            .collect::<Vec<Selector>>()
    });

    eprintln!("filter_selector:{:?}", filter_selectors);
    // Returns a Result of values or an Err early on, stopping the iteration
    // as soon as the latter is encountered.
    let items: Selection = get_selections(&selectors, &json);
    eprintln!("items:{:?}", items);

    // check for empty selection, in this case we assume that the user expects
    // to get back the complete raw JSON back for this group.
    match items {
        Ok(ref items) => {
            // check for an empty selection, in this case we assume that the
            // user expects to get back the complete raw JSON for
            // this group.
            let output_json = if items.is_empty() {
                json.clone()
            } else {
                json!(items.last().unwrap())
            };

            let is_spreading = parsed_group.0.is_some();

            match apply_filter(&output_json, &filter_selectors) {
                Ok(filtered) => match filtered {
                    MaybeArray::Array(array) => Ok(if is_spreading {
                        json!(flatten_array(&array))
                    } else {
                        json!(array)
                    }),
                    MaybeArray::NonArray(single_value) => {
                        if is_spreading {
                            Err(String::from("Only array can be flattened."))
                        } else {
                            // we know that we are holding a single value
                            // wrapped
                            // inside a MaybeArray::NoArray enum.
                            // we nned to pick the first item of the vector.
                            Ok(json!(single_value[0]))
                        }
                    }
                },
                Err(error) => Err(error),
            }
        }
        Err(error) => Err(error),
    }
}
