use crate::types::{Group, Groups, Selector};
use pest::{iterators as pest_iterators, Parser};
use pest_derive::*;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct GroupsParser;

type PestPair<'a> = pest_iterators::Pair<'a, Rule>;

// convert a span to a default selector.
fn span_to_default(inner_span: &str) -> Selector {
    Selector::Default(inner_span.replace(r#"\""#, r#"""#))
}

fn span_to_range(pair: PestPair) -> Selector {
    // Pairs<'_, crate::parser::Rule > iter = pair.clone().into_inner();
    let (start, end) = pair.into_inner().fold(
        (None, None),
        |acc: (Option<PestPair<'_>>, Option<PestPair<'_>>), inner_pair| {
            match inner_pair.as_rule() {
                Rule::start => (Some(inner_pair.clone()), acc.1),
                Rule::end => (acc.0, Some(inner_pair.clone())),
                _ => (acc.0, acc.1),
            }
        },
    );
    let position_to_usize = |value: Option<PestPair<'_>>| {
        value.map(|pair| pair.as_span().as_str().parse::<usize>().unwrap())
    };
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

fn span_to_object(inner_span: Vec<String>) -> Selector {
    Selector::Object(inner_span)
}

// return a vector of chars found inside a default pair.
fn get_chars_from_default_pair(pair: PestPair<'_>) -> Vec<String> {
    pair.into_inner()
        .fold(Vec::new(), |mut acc: Vec<String>, inner_pair| {
            if inner_pair.as_rule() == Rule::chars {
                acc.push(String::from(inner_pair.clone().as_span().as_str()));
            }
            acc
        })
}

// return a vector of nested chars found inside a given pair.
fn get_nested_chars_from_default_pair(pair: PestPair<'_>) -> Vec<String> {
    pair.into_inner()
        .fold(Vec::new(), |mut acc: Vec<Vec<String>>, inner_pair| {
            if inner_pair.as_rule() == Rule::default {
                acc.push(get_chars_from_default_pair(inner_pair));
            }
            acc
        })
        .into_iter()
        .flatten()
        .collect::<Vec<String>>()
}

pub fn selectors_parser(selectors: &str) -> Result<Groups, String> {
    println!("input for selectgor_parser : [{:?}]", selectors);
    match GroupsParser::parse(Rule::groups, selectors) {
        Ok(pairs) => {
            let mut groups: Groups = Vec::new();
            for pair in pairs {
                let mut group: Group = (None, None, Vec::new(), Vec::new());
                // Loop over the pairs converted as an iterator of the tokens
                // which composed it.
                for inner_pair in pair.into_inner() {
                    let inner_span = inner_pair.clone().as_span().as_str();
                    // populate the group based on the rules fond by the parser.
                    match inner_pair.as_rule() {
                        Rule::default => group.2.push(span_to_default(
                            &get_chars_from_default_pair(inner_pair)[0],
                        )),
                        Rule::filter_default => group.3.push(span_to_default(
                            // filter_default will reuse default
                            // we need to unfold to the inner_pair here
                            &get_chars_from_default_pair(
                                inner_pair.into_inner().next().unwrap(),
                            )[0],
                        )),
                        Rule::index => group.2.push(span_to_index(inner_span)),
                        Rule::filter_index => {
                            group.3.push(span_to_index(inner_span))
                        }
                        Rule::range => group.2.push(span_to_range(inner_pair)),
                        Rule::root => group.1 = Some(()),
                        Rule::filter_range => {
                            // filter_range will reuse range
                            // we need to unfold to the inner_pair here
                            group.3.push(span_to_range(
                                inner_pair.into_inner().next().unwrap(),
                            ))
                        }
                        Rule::property => group.2.push(span_to_object(
                            get_nested_chars_from_default_pair(inner_pair),
                        )),
                        Rule::spread => group.0 = Some(()),
                        _ => {
                            println!(
                                "Error, unable to parse invalid selectors"
                            );
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
