#![feature(future_join)]

extern crate clap;
extern crate pest;
extern crate serde_json;

use clap::Parser;
use std::{
    fs::File,
    io,
    io::{BufRead, BufReader, Read},
    string::String,
};

use colored_json::ColoredFormatter;
use kjql::walker;
use serde_json::{
    ser::{CompactFormatter, PrettyFormatter},
    Deserializer, Value,
};
use std::path::Path;

#[derive(Parser, Debug)]
#[command(version, about, long_about = "A json query tool")]
pub struct CommandArgs {
    #[arg(short, long, default_value = None, help = "The JSON file to query, will read from stdin if not given"
    )]
    file: Option<String>,
    #[arg(short, long, default_value = "false")]
    pretty: bool,
    #[arg(short, long, help = "Inlines JSON output")]
    inline: bool,
    #[arg(
        short,
        long,
        default_value = "false",
        help = "Writes raw strings selection directly to standard output \
                without JSON dobule-quotes"
    )]
    raw_output: bool,
    #[arg(help = "The JSON selector to query the JSON content")]
    selector: String,
}

/// Try to serialize the raw JSON content, output selection or throw an error.
fn output(
    json_content: &str,
    inline: bool,
    selectors: String,
    raw_output: bool,
) {
    Deserializer::from_str(json_content)
        .into_iter::<Value>()
        .for_each(|value| match value {
            Ok(valid_json) => {
                // walk through the JSON content with the provided selector.
                match walker(&valid_json, Some(selectors.as_str())) {
                    Ok(selection) => {
                        println!(
                            "{}",
                            // Inline or pretty output
                            (if inline {
                                ColoredFormatter::new(CompactFormatter {})
                                    .to_colored_json_auto(&selection)
                                    .unwrap()
                            } else {
                                // if the selection is a string and the raw-output
                                // flat is passed, directly return the raw string
                                // without JSON double-quotes.
                                if raw_output && selection.is_string() {
                                    String::from(selection.as_str().unwrap())
                                } else {
                                    ColoredFormatter::new(PrettyFormatter::new())
                                        .to_colored_json_auto(&selection).unwrap()
                                }
                            })
                        );
                    }
                    Err(error) => println!("has no value: {:?}", error),
                }
            }
            Err(_) => println!("Invalid JSON file or content!"),
        });
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
                Ok(_) => output(
                    contents.as_str(),
                    args.inline,
                    selector,
                    args.raw_output,
                ),
                Err(error) => {
                    panic!("Couldn't read {}: {}", path.display(), error)
                }
            }
        }
        None => {
            let stdin: Result<String, std::io::Error> =
                io::stdin().lock().lines().collect();
            match stdin {
                Ok(json) => {
                    output(&json, args.inline, selector, args.raw_output)
                }
                Err(error) => eprintln!("error: {}", error),
            }
        }
    }
}
