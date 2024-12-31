#![feature(future_join)]

extern crate clap;
extern crate pest;
extern crate serde_json;

mod panic;

use std::string::String;

use anyhow::Result;
use async_std::{fs, io, path::Path, prelude::*, process::exit};
use clap::Parser;
use colored_json::{ColoredFormatter, Paint};
use kjql::walker;
use panic::use_custom_panic_hook;
use serde_json::{
    ser::{CompactFormatter, PrettyFormatter},
    Deserializer, Value,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = "A json query tool")]
pub struct CommandArgs {
    #[arg(
        short,
        long,
        required = false,
        help = "The JSON file to query, will read from stdin if not given"
    )]
    json: Option<String>,
    #[arg(short, long, default_value = "false")]
    pretty: bool,
    #[arg(short, long, conflicts_with = "check", help = "Inlines JSON output")]
    inline: bool,
    #[arg(
        short,
        long,
        default_value = "false",
        conflicts_with = "check",
        help = "Writes raw strings selection directly to standard output \
                without JSON dobule-quotes"
    )]
    raw_output: bool,
    #[arg(short, long, default_value = "false")]
    stream: bool,
    #[arg(
        short,
        long,
        default_value = "false",
        help = "Checks if the input is valid JSON"
    )]
    check: bool,
    #[arg(
        index = 1,
        required_unless_present_any = ["check", "from_file"],
        help = "The JSON selector to query the JSON content"
    )]
    selector: Option<String>,
    #[arg(
        short,
        long,
        conflicts_with = "check",
        help = "Reads selector from file than from a command argument"
    )]
    from_file: Option<String>,
}

/// Try to serialize the raw JSON content, output selection or throw an error.
async fn render_output(json_content: &str, args: &CommandArgs) {
    if args.check {
        match serde_json::from_str::<Value>(json_content).is_ok() {
            Ok(_) => {
                println!("{}", Paint::green("Valid JSON content!"));
                exit(0);
            }
            Err(error) => {
                println!(
                    "{}",
                    Paint::red(&format!("Invalid JSON content, error: {}", error))
                );
                exit(1);
            }
        }
    }

    // Get a deserializer out of the JSON content.
    let mut deserializer = Deserializer::from_str(json_content);
    // disable recursion limit. pelease to trace the whole story in https://github.com/yamafaktory/jql/issues/120
    deserializer.disable_recursion_limit();

    let selectors: String = match args.from_file.clone() {
        Some(selector_file) => {
            let path = Path::new(selector_file.as_str());
            let contents = fs::read_to_string(path).await;
            match contents {
                Ok(selectors) => selectors,
                Err(error) => {
                    eprintln!("{}", error);
                    exit(1);
                }
            }
        }
        None => args.selector.clone().unwrap(),
    };
    deserializer
        .into_iter::<Value>()
        .for_each(|value| match value {
            Ok(valid_json) => {
                // walk through the JSON content with the provided selector.
                match walker(&valid_json, selectors.as_str()) {
                    Ok(selection) => {
                        println!(
                            "{}",
                            // Inline or pretty output
                            (if args.inline {
                                ColoredFormatter::new(CompactFormatter {})
                                    .to_colored_json_auto(&selection)
                                    .unwrap()
                            } else {
                                // if the selection is a string and the raw-output
                                // flat is passed, directly return the raw string
                                // without JSON double-quotes.
                                if args.raw_output && selection.is_string() {
                                    String::from(selection.as_str().unwrap())
                                } else {
                                    ColoredFormatter::new(PrettyFormatter::new())
                                        .to_colored_json_auto(&selection)
                                        .unwrap()
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
    // use a custom panic hook.
    use_custom_panic_hook();
    // parse the command
    let args = CommandArgs::parse();

    if args.from_file.is_none() && args.selector.is_none() {
        eprintln!("No selectors provided");
        exit(1);
    }

    // hack here, if the check flag enabled, we use the first arguments as files
    // in normal mode the first is `selector` and the second is `files`
    match if args.check {
        args.selector.clone()
    } else {
        args.json.clone()
    } {
        Some(file) => {
            let path = Path::new(&file);

            let contents = fs::read_to_string(path).await?;
            render_output(&contents, &args).await;
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
                            render_output(&line, &args).await;
                        }

                        return Ok(());
                    }

                    // render every line for the stream option.
                    render_output(&line, &args).await;
                    stdout.flush().await?;
                    line.resetting();
                }
            }

            // by default, read the whole piped content from stdin
            let mut buffer = Vec::new();
            stdin.read_to_end(&mut buffer).await?;
            match String::from_utf8(buffer) {
                Ok(lines) => {
                    render_output(&lines, &args).await;
                    Ok(())
                }
                Err(error) => {
                    eprintln!("{}", error);
                    exit(1);
                }
            }
        }
    }
}
