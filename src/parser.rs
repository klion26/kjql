use pest::{iterators as pest_iterators, Parser};
use pest_derive::*;

use crate::types::{Group, InnerObject, Selector};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct GroupsParser;

type PestPair<'a> = pest_iterators::Pair<'a, Rule>;

// convert a span to a default selector.
fn span_to_default(inner_span: &str) -> Selector {
    Selector::Default(inner_span.replace(r#"\""#, r#"""#))
}

fn get_start_and_end_from_pair(pair: PestPair) -> (Option<PestPair<'_>>, Option<PestPair<'_>>) {
    pair.into_inner().fold(
        (None, None),
        |acc: (Option<PestPair<'_>>, Option<PestPair<'_>>), inner_pair| match inner_pair.as_rule() {
            Rule::start => (Some(inner_pair.clone()), acc.1),
            Rule::end => (acc.0, Some(inner_pair.clone())),
            _ => (acc.0, acc.1),
        },
    )
}

fn position_to_usize(value: Option<PestPair<'_>>) -> Option<usize> {
    value.map(|pair| pair.as_span().as_str().parse::<usize>().unwrap())
}

fn span_to_range(pair: PestPair) -> Selector {
    let (start, end) = get_start_and_end_from_pair(pair);

    Selector::Range((position_to_usize(start), position_to_usize(end)))
}

fn span_to_index(inner_span: &str) -> Selector {
    if inner_span.is_empty() {
        return Selector::Array;
    }

    Selector::Index(
        inner_span
            .split(",")
            .map(|index| index.parse::<usize>().unwrap())
            .collect::<Vec<usize>>(),
    )
}

// convert a span to an index selector.
fn span_to_object_index(inner_span: &str) -> InnerObject {
    if inner_span.is_empty() {
        return InnerObject::Array;
    }

    InnerObject::Index(
        inner_span
            .split(",")
            .map(|index| index.parse::<usize>().unwrap())
            .collect::<Vec<usize>>(),
    )
}

// convert a span to a range selector with an inner object.
fn span_to_object_range(pair: PestPair<'_>) -> InnerObject {
    let (start, end) = get_start_and_end_from_pair(pair);

    InnerObject::Range((position_to_usize(start), position_to_usize(end)))
}

// return a vector of chars found inside a default pair.
fn get_chars_from_pair(pair: PestPair<'_>) -> Vec<String> {
    pair.into_inner()
        .fold(Vec::new(), |mut acc: Vec<String>, inner_pair| {
            if inner_pair.as_rule() == Rule::chars {
                acc.push(String::from(inner_pair.clone().as_span().as_str()));
            }
            acc
        })
}

// return a vector of nested chars found inside a given pair.
fn get_inner_object_from_pair(pair: PestPair<'_>) -> Vec<InnerObject> {
    pair.into_inner()
        .fold(Vec::new(), |mut acc: Vec<InnerObject>, inner_pair| {
            match inner_pair.as_rule() {
                Rule::default => {
                    acc.push(InnerObject::KeyValue(
                        get_chars_from_pair(inner_pair)[0].clone(),
                        None,
                    ));
                }
                Rule::filter_lens_key_value_pair => {
                    let key = &get_chars_from_pair(
                        inner_pair
                            .clone()
                            .into_inner()
                            .next()
                            .unwrap()
                            .into_inner()
                            .next()
                            .unwrap(),
                    )[0];

                    let maybe_value = inner_pair.into_inner().nth(1).map(|pair| {
                        get_chars_from_pair(pair.into_inner().next().unwrap())[0].clone()
                    });

                    acc.push(InnerObject::KeyValue(key.to_string(), maybe_value));
                }
                Rule::object_range => {
                    acc.push(span_to_object_range(
                        inner_pair.into_inner().next().unwrap(),
                    ));
                }
                Rule::object_index => {
                    acc.push(span_to_object_index(inner_pair.as_span().as_str()));
                }
                _ => {}
            }
            acc
        })
}

pub fn selectors_parser(selectors: &str) -> Result<Vec<Group>, String> {
    println!("input for selectgor_parser : [{:?}]", selectors);
    match GroupsParser::parse(Rule::groups, selectors) {
        Ok(pairs) => {
            let mut groups: Vec<Group> = Vec::new();
            for pair in pairs {
                let mut group: Group = Group::new();
                // Loop over the pairs converted as an iterator of the tokens
                // which composed it.
                for inner_pair in pair.into_inner() {
                    let inner_span = inner_pair.clone().as_span().as_str();
                    // populate the group based on the rules fond by the parser.
                    match inner_pair.as_rule() {
                        // Default
                        Rule::default => group
                            .selectors
                            .push(span_to_default(&get_chars_from_pair(inner_pair)[0])),
                        Rule::filter_default => {
                            group.filters.push(span_to_default(
                                // filter_default will reuse default
                                // we need to unfold to the inner_pair here
                                &get_chars_from_pair(inner_pair.into_inner().next().unwrap())[0],
                            ))
                        }
                        // index
                        Rule::index => group.selectors.push(span_to_index(inner_span)),
                        Rule::filter_index => group.filters.push(span_to_index(inner_span)),
                        // range
                        Rule::range => group.selectors.push(span_to_range(inner_pair)),
                        Rule::filter_range => {
                            // filter_range will reuse range
                            // we need to unfold to the inner_pair here
                            group
                                .filters
                                .push(span_to_range(inner_pair.into_inner().next().unwrap()))
                        }
                        // property
                        Rule::property => group
                            .selectors
                            .push(Selector::Object(get_inner_object_from_pair(inner_pair))),
                        Rule::filter_property => group
                            .filters
                            .push(Selector::Object(get_inner_object_from_pair(inner_pair))),
                        // filter lenses property.
                        Rule::filter_lens_key_value => group
                            .filter_lenses
                            .push(Selector::Object(get_inner_object_from_pair(inner_pair))),
                        // root
                        Rule::root => group.root = Some(()),
                        // spread
                        Rule::spread => group.spread = Some(()),
                        // truncate
                        Rule::truncate => group.truncate = Some(()),
                        _ => {
                            println!("Error, unable to parse invalid selectors");
                            todo!()
                        }
                    };
                }
                // add the group.
                groups.push(group);
            }
            println!("::: single groups :::::: {:?}", groups);
            Ok(groups)
        }
        Err(_) => Err(String::from("Error, unable to parse invalid selectors")),
    }
}
