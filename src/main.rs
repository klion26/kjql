#![feature(type_ascription)]
#![feature(future_join)]

extern crate clap;
extern crate serde_json;

use clap::Parser;
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::future::join;
use std::io::Read;

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

type Selection = Result<Vec<Value>, String>;

fn get_selection(json: &Value, selector: Option<&str>) -> Option<Selection> {
    let mut inner_json = json;
    let throw = |s, selector: &Vec<&str>, i: usize | -> String {
        ["Node (", s, ") not found on parent (", selector[i - 1], ")"].join(" ")
    };

    if let Some(selector) = selector {
        let selector: Vec<&str> = selector.split('.').collect();
        // Returns Result of values or Err early on, stopping the iteration.
        let items: Selection = selector
            .iter()
            .enumerate()
            .map(|(i, s)| -> Result<Value, String> {
                // let iter_index = i.to_string();
                if let Ok(index) = s.parse::<usize>() {
                    if (index as isize).is_negative() {
                        Err("Negative index".to_string())
                    } else {
                        if inner_json[index] == Value::Null {
                            Err(throw(s, &selector, i))
                        } else {
                            inner_json = &inner_json[index];
                            Ok(inner_json.clone())
                        }
                    }
                } else {
                    if inner_json[s] == Value::Null {
                        if i == 0 {
                            Err(["Node (", s, ") is not the root element"].join(" "))
                        } else {
                            Err(throw(s, &selector, i))
                        }
                    } else {
                        inner_json = &inner_json[s];
                        Ok(inner_json.clone())
                    }
                }
            }).collect();
        Some(items)
    } else {
        None
    }
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
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {
            match serde_json::from_str(&contents) {
                Ok(valid_json) => {
                    if args.pretty {
                        println!("{}", serde_json::to_string_pretty(&valid_json).unwrap())
                    }
                    match get_selection(&valid_json, Some(selector.as_str())) {
                        Some(items) => match items {
                            Ok(results) => println!("{}", serde_json::to_string_pretty(&results.last()).unwrap()),
                            Err(error) => println!("{}", error),
                        },
                        None => println!("has no value"),
                    }
                }
                Err(_) => println!("Invalid JSON file!"),
            }
        }
        Err(error) => panic!("Couldn't read {}: {}", path.display(), error.description()),
    }

    println!("Hello, world!");
}
