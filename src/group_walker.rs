use crate::apply_filter::apply_filter;
use crate::get_selection::get_selections;
use crate::get_selector::get_selector;
use crate::types::{Selection, Selector};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;

// walks through a group
pub fn group_walker(
    capture: &regex::Captures<'_>,
    filter: Option<&str>,
    json: &Value,
) -> Selection {
    lazy_static! {
        static ref SUB_GROUP_REGEX: Regex =
            Regex::new(r#"("[^"]+")|([^.]+)"#).unwrap();
    }
    let group = capture.get(0).map_or("", |m| m.as_str().trim());

    println!("** {:?} **", filter);

    // empty group, return early
    if group.is_empty() {
        return Err(String::from("Empty group"));
    }

    // capture sub-groups of doulbe quoted selectors and simple ones surrounded
    // by dots.
    let selectors: Vec<Selector> = SUB_GROUP_REGEX
        .captures_iter(group)
        .map(|capture| get_selector(capture.get(0).map_or("", |m| m.as_str())))
        .collect();

    // perform the same operation on the filter.
    let filter_selectors = filter.map(|filter| {
        SUB_GROUP_REGEX
            .captures_iter(filter)
            .map(|capture| {
                get_selector(capture.get(0).map_or("", |m| m.as_str()))
            })
            .collect::<Vec<Selector>>()
    });
    println!("filter_selector:{:?}", filter_selectors);
    // Returns a Result of values or an Err early on, stopping the iteration
    // as soon as the latter is encountered.
    let items: Selection = get_selections(&selectors, &json);
    println!("items:{:?}", items);

    // check for empty selection, in this case we assume that the user expects
    // to get back the complete raw JSON back for this group.
    match items {
        Ok(items) => {
            if items.is_empty() {
                apply_filter(&json, &filter_selectors)
            } else {
                items
                    .iter()
                    .map(|item| apply_filter(&item, &filter_selectors))
                    .last()
                    .unwrap()
            }
        }
        Err(error) => Err(error),
    }
}
