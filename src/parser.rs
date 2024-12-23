use crate::types::Selector;
use crate::types::{Group, Groups};
use lazy_static::lazy_static;
use pest::Parser;
use regex::Regex;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct GroupsParser;

// drop the enclosing double quotes of a span
// and convert it to a default selector
fn span_to_default(inner_span: &str) -> Selector {
    Selector::Default(
        String::from(&inner_span[1..inner_span.len() - 1])
            .replace(r#"\""#, r#"""#),
    )
}

fn span_to_range(inner_span: &str) -> Selector {
    lazy_static! {
        static ref RANGE_REGEX: Regex = Regex::new(r"(\d+):(\d+)").unwrap();
    }

    let ranges: Vec<(&str, &str)> = RANGE_REGEX
        .captures_iter(inner_span)
        .map(|capture| {
            (
                capture.get(1).map_or("", |m| m.as_str()),
                capture.get(2).map_or("", |m| m.as_str()),
            )
        })
        .collect();
    if ranges.is_empty() {
        Selector::Default(String::from(inner_span))
    } else {
        let (start, end) = &ranges[0];
        Selector::Range((
            start.parse::<usize>().unwrap(),
            end.parse::<usize>().unwrap(),
        ))
    }
}

fn span_to_index(inner_span: &str) -> Selector {
    Selector::Index(
        inner_span
            .replace(r#"["#, "")
            .replace(r#"]"#, "")
            .parse::<usize>()
            .unwrap(),
    )
}
pub fn selectors_parser(selectors: &str) -> Result<Groups, String> {
    println!("selectors:[{:?}]", selectors);
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
                    println!(
                        "Inner_span:[{:?}]<--[{:?}]<<--[{:?}]",
                        inner_span,
                        inner_pair.as_rule(),
                        inner_pair
                    );
                    match inner_pair.as_rule() {
                        Rule::default => {
                            group.2.push(span_to_default(inner_span))
                        }
                        Rule::filterDefault => {
                            group.3.push(span_to_default(inner_span))
                        }
                        Rule::index => group.2.push(span_to_index(inner_span)),
                        Rule::filterIndex => {
                            group.3.push(span_to_index(inner_span))
                        }
                        Rule::range => group.2.push(span_to_range(inner_span)),
                        Rule::root => group.1 = Some(()),
                        Rule::filterRange => {
                            group.3.push(span_to_range(inner_span))
                        }
                        Rule::spread => group.0 = Some(()),
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
