#![feature(future_join)]

mod core;
mod tests;
mod types;
mod utils;

use crate::core::walker;
extern crate clap;
#[macro_use]
extern crate serde_json;

use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Read};
use std::string::String;

use std::path::Path;

#[derive(Parser, Debug)]
#[command(version, about, long_about = "A json query tool")]
pub struct CommandArgs {
    #[arg(short, long)]
    file: String,
    #[arg(short, long)]
    selector: String,
    #[arg(short, long, default_value = "false")]
    pretty: bool,
}

fn main() {
    // parse the command
    let args = CommandArgs::parse();

    let selector = args.selector;
    let file = args.file;
    let path = Path::new(&file);
    let mut file = match File::open(&path) {
        Err(error) => panic!("{}", error),
        Ok(file) => file,
    };
    let mut buffer_reader = BufReader::new(file);
    let mut contents = String::new();
    match buffer_reader.read_to_string(&mut contents) {
        Ok(_) => match serde_json::from_str(&contents) {
            Ok(valid_json) => {
                if args.pretty {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&valid_json).unwrap()
                    )
                }
                // walk through the JSON content with the provided selector.
                match walker(&valid_json, Some(selector.as_str())) {
                    Some(items) => match items {
                        Ok(results) => println!(
                            "{}",
                            serde_json::to_string_pretty(&results.last())
                                .unwrap()
                        ),
                        Err(error) => println!("{}", error),
                    },
                    None => println!("has no value"),
                }
            }
            Err(_) => println!("Invalid JSON file!"),
        },
        Err(error) => {
            panic!("Couldn't read {}: {}", path.display(), error)
        }
    }

    println!("Hello, world!");
}
