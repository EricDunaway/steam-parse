use serde_json::to_string_pretty;
use std::{collections::HashMap, fs::File, io::Read};
use vdf_parse::parse_hash_entity;

use clap::Parser;

/// Simple program to parse a VDF file and print the result
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name and path of the file to read
    #[arg(short, long)]
    input: String,
}

fn main() {
    println!("Most of the work is in the vdf-parse library");
    let args = Args::parse();
    println!("Reading file: {}", args.input);

    // Read file into &mut &[u8] and call parse_hash_entity then print the result
    let mut vdf_hashmap = HashMap::new();
    {
        let mut vdf_file = File::open(args.input).unwrap();
        let size = vdf_file.metadata().unwrap().len() as usize;
        let mut buffer: Vec<u8> = Vec::with_capacity(size);
        vdf_file
            .read_to_end(&mut buffer)
            .expect("Eror Reading file");
        let mut buffer = buffer.as_slice();
        let result = match parse_hash_entity(&mut buffer){
            Ok(actual) => actual,
            Err(e) => panic!("Error parsing VDF file: {}", e),
        };
        vdf_hashmap.insert(result.0, result.1);
    }
    let json = to_string_pretty(&vdf_hashmap).unwrap();
    println!("{}", json);
}
