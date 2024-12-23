#![feature(future_join)]

mod apply_filter;
mod array_walker;
mod core;
mod flatten_json_array;
mod get_selection;
mod group_walker;
mod parser;
mod range_selector;
mod tests;
mod types;
mod utils;

use crate::core::walker;
extern crate clap;
extern crate pest;
extern crate serde_json;
#[macro_use]
extern crate pest_derive;

use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Read};
use std::string::String;

use colored_json::ColoredFormatter;
use serde_json::ser::{CompactFormatter, PrettyFormatter};
use std::path::Path;

#[derive(Parser, Debug)]
#[command(version, about, long_about = "A json query tool")]
pub struct CommandArgs {
    #[arg(short, long, default_value = None, help = "The JSON file to query, will read from stdin if not given")]
    file: Option<String>,
    #[arg(short, long)]
    selector: String,
    #[arg(short, long, default_value = "false")]
    pretty: bool,
    #[arg(short, long, help = "Inlines JSON output")]
    inline: bool,
}

/// Try to serialize the raw JSON content, output selection or throw an error.
fn output(json_content: &str, inline: bool, selectors: String) {
    match serde_json::from_str(json_content) {
        Ok(valid_json) => {
            // walk through the JSON content with the provided selector.
            match walker(&valid_json, Some(selectors.as_str())) {
                Ok(items) => {
                    println!(
                        "{}",
                        // Inline or pretty output
                        (if inline {
                            ColoredFormatter::new(CompactFormatter {})
                                .to_colored_json_auto(&items)
                        } else {
                            ColoredFormatter::new(PrettyFormatter::new())
                                .to_colored_json_auto(&items)
                        })
                        .unwrap()
                    )
                }
                Err(error) => println!("has no value: {:?}", error),
            }
        }
        Err(_) => println!("Invalid JSON file or content!"),
    }
}

fn main() {
    // parse the command
    let args = CommandArgs::parse();

    let selector = args.selector;
    match args.file {
        Some(file) => {
            let path = Path::new(&file);
            let file = match File::open(path) {
                Err(error) => panic!("{}", error),
                Ok(file) => file,
            };
            let mut buffer_reader = BufReader::new(file);
            let mut contents = String::new();
            match buffer_reader.read_to_string(&mut contents) {
                Ok(_) => output(contents.as_str(), args.inline, selector),
                Err(error) => {
                    panic!("Couldn't read {}: {}", path.display(), error)
                }
            }
        }
        None => {
            let mut buffer = String::new();
            match std::io::stdin().read_line(&mut buffer) {
                Ok(_) => output(&buffer, args.inline, selector),
                Err(error) => eprintln!("error: {}", error),
            }
        }
    }
}
