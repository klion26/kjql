use serde_json::Value;

use crate::utils::{
    display_array_selector, display_default_selector, display_index_selector,
    display_object_selector, display_range_selector,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Selector {
    /// Default variant.
    Default(String),
    /// Array variant.
    Array,
    /// Index variant.
    Index(Vec<usize>),
    /// Object variant.
    Object(Vec<InnerObject>),
    /// Range variant.
    Range((Option<usize>, Option<usize>)),
}

/// Inner objects.
#[derive(Debug, PartialEq, Eq)]
pub enum InnerObject {
    /// Array variant
    Array,
    /// Index variant
    Index(Vec<usize>),
    /// Key / value variant
    KeyValue(String, Option<String>),
    /// Range variant.
    Range((Option<usize>, Option<usize>)),
}

#[doc(hidden)]
pub trait Display {
    fn as_str(&self, capitalized: bool) -> String;
}

impl Display for Selector {
    fn as_str(&self, capitalized: bool) -> String {
        // return the selector as a readable string.
        match self {
            Selector::Default(value) => display_default_selector(&value.clone(), capitalized),
            Selector::Array => display_array_selector(capitalized),
            Selector::Index(index) => display_index_selector(index, capitalized),
            Selector::Object(properties) => display_object_selector(properties, capitalized),
            Selector::Range(range) => display_range_selector(*range, capitalized),
        }
    }
}

impl Display for InnerObject {
    // Return the selector as a readable string.
    fn as_str(&self, capitalized: bool) -> String {
        match self {
            InnerObject::Array => display_array_selector(capitalized),
            InnerObject::Index(indexes) => display_index_selector(indexes, capitalized),
            InnerObject::KeyValue(key, _value) => key.to_string(),
            InnerObject::Range(range) => display_range_selector(*range, capitalized),
        }
    }
}
/// A Group is a set of grammer elements used to define a selection.
#[derive(Debug, PartialEq, Eq)]
pub struct Group {
    /// filters.
    pub filters: Vec<Selector>,
    /// filter lenses.
    pub filter_lenses: Vec<Selector>,
    /// root marker.
    pub root: Option<()>,
    /// selectors.
    pub selectors: Vec<Selector>,
    /// spread marker.
    pub spread: Option<()>,
    /// truncate marker.
    pub truncate: Option<()>,
}

/// Group implementations
impl Group {
    /// Create a new group.
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            filter_lenses: Vec::new(),
            root: None,
            selectors: Vec::new(),
            spread: None,
            truncate: None,
        }
    }
}

impl Default for Group {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub enum MayArray {
    Array(Vec<Value>),
    NonArray(Vec<Value>),
}

pub(crate) type Selection = Result<Value, String>;
pub(crate) type Selections = Result<Vec<Value>, String>;
pub(crate) type ExtendedSelections = Result<MayArray, String>;
