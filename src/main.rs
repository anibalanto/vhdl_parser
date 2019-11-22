extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

#[derive(Parser)]
#[grammar = "vhdl.pest"]
pub struct GenParser;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum AstNode {
    Gen{
        entity: Box<AstNode>,
        architecture: Box<AstNode>
    },
    Entity{
        ident: String,
        terms: Box<AstNode>
    },
    Generics(Box<AstNode>),
    Ports(Box<AstNode>),

    Architecture(Box<AstNode>),

    StateMachine{
        ident: String,
        sensibility: Box<AstNode>,
        terms: Box<AstNode>
    },
    State{
        ident: String,
        terms: Box<AstNode>
    },

    TransitionSimple{
        activator: Option<Box<AstNode>>,
        to: Box<AstNode>
    },
    TransitionComposed{
        terms: Box<AstNode>
    },
    TransitionActived{
        activator: Box<AstNode>,
        to: Box<AstNode>
    },
    TransitionLast {
        to: Box<AstNode>
    },

    Type{
        name: String,
        vector: Option<Box<AstNode>>
    },

    DefineGeneric{
        ident: String,
        def_type: Box<AstNode>,
        value: Option<String>
    },

    DefinePort{
        ident: String,
        direction: String,
        def_type: Box<AstNode>,
    },
    Vector{
        start: String,
        end: String
    },

    Terms(Vec<AstNode>),
    UniqueTerms(Vec<AstNode>),

    Null
}

use pest::error::Error;
use crate::AstNode::{Entity, Terms, Generics, Architecture, StateMachine, State, Vector, Null, Gen, UniqueTerms, TransitionComposed, TransitionSimple, TransitionActived, TransitionLast, Ports, DefineGeneric, DefinePort, Type};
use pest::iterators::{Pair, Pairs};

pub fn parse(source: &str) -> Result<AstNode, Error<Rule>> {
    let mut pairs = GenParser::parse(Rule::vhdl, source)?;
    Ok(build_ast_from_expr(pairs.next().unwrap()))
}

fn build_ast_from_expr(pair: Pair<Rule>) -> AstNode {

    fn as_string(pair: Pair<Rule>) -> String {
        pair.as_str().to_string()
    }
    fn next_item_as_string(pair: & mut Pairs<Rule>) -> String {
        match pair.next() {
            Some(pair) => as_string(pair),
            None => "None".to_owned()
        }
    }
    fn next_item(pair: & mut Pairs<Rule>) -> AstNode {
        build_ast_from_expr(pair.next().expect("no pair here: try make ast from it!"))
    }
    fn items_as_vector(pair: Pairs<Rule>) -> Vec<AstNode> {
        pair
            .map(|rule| {
                build_ast_from_expr(rule)
            })
            .collect()
    }

    match pair.as_rule() {
        Rule::entity => {
            let mut pair = pair.into_inner();
            Entity {
                ident: next_item_as_string(& mut pair),
                terms: Box::new(next_item(& mut pair))
            }
        },
        Rule::generics =>
            Generics(Box::new(next_item(& mut pair.into_inner()))),
        Rule::ports =>
            Ports(Box::new(next_item(& mut pair.into_inner()))),
        Rule::type_def => {
            let mut pair = pair.into_inner();
            let name = next_item_as_string(& mut pair);
            match pair.next() {
                Some(vector_item) =>
                    Type {
                        name,
                        vector: Some(Box::new(build_ast_from_expr(vector_item))),
                    },
                None =>
                    Type {
                        name,
                        vector: None,
                    }
            }
        }
        Rule::def_generic => {
            let mut pair = pair.into_inner();
            let ident = next_item_as_string(& mut pair);
            let def_type = Box::new(next_item(& mut pair));
            match pair.next() {
                Some(value_item) =>
                    DefineGeneric {
                        ident,
                        def_type,
                        value: Some(as_string(value_item)),
                    },
                None =>
                    DefineGeneric {
                        ident: ident,
                        def_type,
                        value: None,
                    }
            }

        },
        Rule::def_port => {
            let mut pair = pair.into_inner();
            DefinePort {
                ident: next_item_as_string(&mut pair),
                direction: next_item_as_string(&mut pair),
                def_type: Box::new(next_item(&mut pair)),
            }
        },
        Rule::vector => {
            let mut pair = pair.into_inner();
            Vector {
                start: next_item_as_string(& mut pair),
                end: next_item_as_string(& mut pair)
            }
        }

        Rule::generics |
        Rule::ports =>
            Terms(items_as_vector(pair.into_inner())),

        Rule::entity_block =>
            UniqueTerms(items_as_vector(pair.into_inner())),

        _ => Null
    }
}

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
    if test { println!("[input file]\n{}", vhdl); }
    match parse(&vhdl) {
        Ok(astnode) => {
            if test {
                println!("[json parse result]");

                let serialized = serde_json::to_string(&astnode).unwrap();
                println!("serialized = {}", serialized);

                let deserialized: AstNode = serde_json::from_str(&serialized).unwrap();
                println!("deserialized = {:?}", deserialized);
            }
        },
        Err(e) =>
            println!("{}", e)
    }
}

fn help() {
    println!("usage:
vhdl_pasrse <FILE> [options]
options {{testmode}} ");
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





#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
