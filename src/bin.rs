use libvhdlparser::vhdl_to_json;
use libvhdlparser::AstNode;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn read_file(path: &String) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    Ok(contents)
}

fn vhdl_file_to_json(path: &String, test: bool) {
    let vhdl = read_file(&path)
        .expect("Something went wrong when I was reading the file");
    if test {
        println!("[input file]");
        println!("{}", vhdl);
    }
    match vhdl_to_json(&vhdl, test) {
        Ok(serialized) => {
            if test {
                println!("[json parse result]");
            }
            println!("{}", serialized);
            if test {
                let deserialized: AstNode = serde_json::from_str(&serialized).unwrap();
                println!("deserialized = {:?}", deserialized);
            }
        },
        Err(e) => eprintln!("Parsing error: {}", e)
    }
}

fn help() {
    println!("usage:
vhdl_parser <FILE> [options]
options {{test}} ");
}

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => println!("No .vhdl file input!"),
        2 => vhdl_file_to_json(&args[1], false),
        3 => {
            match &args[2][..] {
                "test" => vhdl_file_to_json(&args[1], true),
                _ => {
                    eprintln!("error: invalid command");
                    help();
                },
            }
        },
        _ => {
            help();
        }
    }
}

