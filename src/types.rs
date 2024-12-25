use crate::utils::{
    display_array_selector, display_default_selector, display_index_selector,
    display_object_selector, display_range_selector,
};
use serde_json::Value;

pub type Selection = Result<Vec<Value>, String>;

#[derive(Debug)]
pub enum Selector {
    Default(String),
    Array,
    Index(Vec<usize>),
    Object(Vec<String>),
    Range((Option<usize>, Option<usize>)),
}

pub trait Display {
    fn as_str(&self, capitalized: bool) -> String;
}

impl Display for Selector {
    fn as_str(&self, capitalized: bool) -> String {
        // return the selector as a readable string.
        match self {
            Selector::Default(value) => {
                display_default_selector(&value.clone(), capitalized)
            }
            Selector::Array => display_array_selector(capitalized),
            Selector::Index(index) => {
                display_index_selector(index, capitalized)
            }
            Selector::Range(range) => {
                display_range_selector(*range, capitalized)
            }
            Selector::Object(properties) => {
                display_object_selector(properties, capitalized)
            }
        }
    }
}

#[derive(Debug)]
pub enum MaybeArray {
    Array(Vec<Value>),
    NonArray(Vec<Value>),
}

pub type Selectors = [Selector];

pub type ExtendedSelection = Result<MaybeArray, String>;

pub type Group = (
    // spread part.
    Option<()>,
    // Root part.
    Option<()>,
    // selectors part.
    Vec<Selector>,
    // filters part.
    Vec<Selector>,
);

pub type Groups = Vec<Group>;
