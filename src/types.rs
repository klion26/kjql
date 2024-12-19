use serde_json::Value;

pub type Selection = Result<Vec<Value>, String>;

#[derive(Debug)]
pub enum Selector {
    Default(String),
    Range((usize, usize)),
}

#[derive(Debug)]
pub enum MaybeArray {
    Array(Vec<Value>),
    NonArray(Vec<Value>),
}

pub type Selectors = [Selector];

pub type ExtendedSelection = Result<MaybeArray, String>;
