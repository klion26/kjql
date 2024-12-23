use serde_json::Value;

pub type Selection = Result<Vec<Value>, String>;

#[derive(Debug)]
pub enum Selector {
    Default(String),
    Index(usize),
    Range((usize, usize)),
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
