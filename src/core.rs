use rayon::prelude::*;
use serde_json::{json, Value};

use crate::{
    group_walker::group_walker,
    parser::selectors_parser,
    types::{Group, Selection, Selections},
};

/// walks over the Serde JSON value based on the provided selectors
pub fn walker(json: &Value, selectors: &str) -> Selection {
    match selectors_parser(selectors) {
        Ok(groups) => groups_walker(json, &groups),
        Err(error) => Err(error),
    }
}

/// Walks over the Serde JSON value based on the provided groups.
pub fn groups_walker(json: &Value, groups: &[Group]) -> Result<Value, String> {
    // Capture groups separated by commas apply the selector for the
    // current group and return a Result of values or an Err early
    // on.
    let inner_groups: Selections = groups
        .par_iter()
        .map(|group| group_walker(group, json))
        .collect();
    match inner_groups {
        Ok(groups) => match groups.len() {
            0 => Err(String::from("Empty selection")),
            1 => Ok(json!(groups[0])),
            _ => Ok(json!(groups)),
        },
        Err(error) => Err(error),
    }
}
