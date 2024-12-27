#![feature(future_join)]

extern crate clap;
extern crate pest;
extern crate serde_json;

use std::string::String;

use anyhow::Result;
use async_std::{fs, io, path::Path, prelude::*, process::exit, task};
use clap::Parser;
use colored_json::{ColoredFormatter, Paint};
use kjql::walker;
use serde_json::{
    ser::{CompactFormatter, PrettyFormatter},
    Deserializer, Value,
};

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
    #[arg(short, long, default_value = "false")]
    stream: bool,
    #[arg(help = "The JSON selector to query the JSON content")]
    selector: String,
}

/// Try to serialize the raw JSON content, output selection or throw an error.
fn render_output(
    json_content: &str,
    inline: bool,
    selectors: &str,
    raw_output: bool,
) {
    Deserializer::from_str(json_content)
        .into_iter::<Value>()
        .for_each(|value| match value {
            Ok(valid_json) => {
                // walk through the JSON content with the provided selector.
                match walker(&valid_json, Some(selectors)) {
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
                    Err(error) => {
                        println!("has no value: {:?}", error);
                        exit(1);
                    }
                }
            }
            Err(_) => {
                println!("Invalid JSON file or content!");
                exit(1);
            }
        });
}

#[async_std::main]
async fn main() -> Result<()> {
    // parse the command
    let args = CommandArgs::parse();

    let selector = args.selector;
    match args.file {
        Some(file) => {
            let path = Path::new(&file);

            let contents = fs::read_to_string(path).await?;
            render_output(
                &contents,
                args.inline,
                selector.as_str(),
                args.raw_output,
            );
            Ok(())
        }
        // JSON content coming from stdin.
        None => {
            let stream = args.stream;
            let mut stdin = io::stdin();
            let mut stdout = io::stdout();

            // special case for the stream option.
            // in this case, read line by line.
            if stream {
                let mut line = String::new();

                loop {
                    // read one line from stdin
                    let n = stdin.read_line(&mut line).await?;
                    // check for the EOF.
                    if n == 0 {
                        if !stream {
                            render_output(
                                &line,
                                args.inline,
                                selector.as_str(),
                                args.raw_output,
                            );
                        }

                        return Ok(());
                    }

                    // render every line for the stream option.
                    render_output(
                        &line,
                        args.inline,
                        selector.as_str(),
                        args.raw_output,
                    );
                    stdout.flush().await?;
                    line.resetting();
                }
            }

            // by default, read the whole piped content from stdin
            let mut buffer = Vec::new();
            stdin.read_to_end(&mut buffer).await?;
            match String::from_utf8(buffer) {
                Ok(lines) => {
                    render_output(
                        &lines,
                        args.inline,
                        selector.as_str(),
                        args.raw_output,
                    );
                    return Ok(());
                }
                Err(error) => {
                    eprintln!("{}", error);
                    exit(1);
                }
            }
        }
    }
}
