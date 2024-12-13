use std::fs::File;
use std::io::prelude::Read;
use std::io::BufReader;
use toml::Value;

pub fn get_cargo_version() -> String {
    let cargo_file = File::open("Cargo.toml").unwrap();
    let mut buffer_reader = BufReader::new(cargo_file);
    let mut yaml = String::new();

    buffer_reader.read_to_string(&mut yaml).unwrap();
    let parsed = yaml.parse::<Value>().unwrap();
    String::from((parsed["package"]["version"]).as_str().unwrap())
}
