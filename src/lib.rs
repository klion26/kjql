//! # A JSON query language library
//!
//! This crate is used by `kjql`, the `JSON query language CLI tool`.

mod apply_filter;
mod array_walker;
mod core;
mod flatten_json_array;
mod get_selection;
mod group_walker;
mod panic;
mod parser;
mod range_selector;
mod tests;
mod truncate;
mod types;
mod utils;

use serde_json::Value;

pub use crate::types::*;

/// Process a Serde JSON Value based on the provided selectors.
///
/// # Example
///
/// ```
/// use serde_json::json;
/// let json_array = json!([2, 3, 5, 7, 11]);
/// assert_eq!(kjql::walker(&json_array, "[4]").unwrap(), json!(11));
/// ```
pub fn walker(json: &Value, selectors: &str) -> Selection {
    core::walker(json, selectors)
}

/// Walks over the Serde JSON value based on the provided groups.
///
/// # Example
/// ```
/// use kjql::{Group, groups_walker, Selector::{Index}};
/// use serde_json::json;
/// let json_array = json!([2, 3, 5, 7, 11]);
///
/// assert_eq!(
///     groups_walker(
///         &json_array,
///         &[Group {
///             filters: vec![],
///             root: None,
///             selectors: vec![Index(vec![4])],
///             spread: None,
///             truncate: None,}]
///    ),
///    Ok(json!(11)));
pub fn groups_walker(json: &Value, groups: &[Group]) -> Selection {
    core::groups_walker(json, groups)
}

/// Parses the provided selectors and returns a vector of group.
pub fn selectors_parser(selectors: &str) -> Result<Vec<Group>, String> {
    parser::selectors_parser(selectors)
}
