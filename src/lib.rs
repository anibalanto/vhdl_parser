extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

#[derive(Parser)]
#[grammar = "vhdl.pest"]
pub struct GenParser;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum AstNode {
    Entity{
        ident: String,
        generics: Option<Vec<AstNode>>,
        ports: Vec<AstNode>,
        signals: Vec<AstNode>,
    },

    Generic(Vec<AstNode>),
    Port(Vec<AstNode>),
    DefineSignal{
        ident: String,
        def_type: Box<AstNode>,
    },

    DefineGeneric{
        ident: String,
        def_type: Box<AstNode>,
        value: Option<Box<AstNode>>
    },

    DefinePort{
        ident: String,
        direction: String,
        def_type: Box<AstNode>,
    },

    Type{
        name: String,
        vector: Option<Box<AstNode>>
    },

    Operation {
        left: Box<AstNode>,
        operator: String,
        right: Box<AstNode>
    },

    Vector{
        start: Box<AstNode>,
        end: Box<AstNode>
    },

    Str(String),
    Int(u32),

    Null
}

use pest::error::Error;
use crate::AstNode::{Entity, Operation, Vector, Null, DefineGeneric, DefinePort, Type, Int, Str, Generic, DefineSignal, Port};
use pest::iterators::{Pair, Pairs};

pub fn parse(source: &str) -> Result<AstNode, Error<Rule>> {
    let mut pairs = GenParser::parse(Rule::vhdl, source)?;
    Ok(build_ast(pairs.next().unwrap()))
}

fn build_ast(pair: Pair<Rule>) -> AstNode {

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
        build_ast(pair.next().expect("no pair here: try make ast from it!"))
    }
    fn items_as_vector(pair: Pairs<Rule>) -> Vec<AstNode> {
        pair
            .map(|rule| {
                build_ast(rule)
            })
            .collect()
    }

    match pair.as_rule() {
        Rule::entity => {
            let mut pair = pair.into_inner();
            let ident = as_string(pair.next().expect("entity ident not found"));

            let mut generics: Option<Vec<AstNode>> = None;
            let mut ports: Vec<AstNode> = Vec::new();
            let mut signals: Vec<AstNode> = Vec::new();
            let pair = pair.next().expect("entity_block not found").into_inner();
            for reg in pair {
                match build_ast(reg) {
                    AstNode::Generic(vec) =>
                        generics = Some(vec),
                    AstNode::Port(vec) =>
                        ports = vec,
                    AstNode::DefineSignal {ident, def_type} => {
                        signals.push(
                            AstNode::DefineSignal{
                                ident,
                                def_type
                            });
                    }
                    _ => ()
                }
            }
            Entity {
                ident,
                generics,
                ports,
                signals
            }
        },
        Rule::generics =>
            Generic(items_as_vector(pair.into_inner())),
        Rule::ports =>
            Port(items_as_vector(pair.into_inner())),
        Rule::def_signal =>{
            let mut pair = pair.into_inner();
            DefineSignal {
                ident: next_item_as_string(&mut pair),
                def_type: Box::new(next_item(&mut pair)),
            }
        },
        Rule::type_def_generic =>
                    Type {
                        name: as_string(pair),
                        vector: None,
                    },
        Rule::type_def => {
            let mut pair = pair.into_inner();
            let name = next_item_as_string(& mut pair);
            match pair.next() {
                Some(vector_item) =>
                    Type {
                        name,
                        vector: Some(Box::new(build_ast(vector_item))),
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
            DefineGeneric {
                ident,
                def_type,
                value: match pair.next() {
                    Some(value_item) =>
                        Some(Box::new(Int(value_item.as_str().parse::<u32>().unwrap()))),
                    None => None
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
                start: Box::new(build_ast(pair.next().unwrap())),
                end: Box::new(build_ast(pair.next().unwrap()))
            }
        }
        Rule::max_term | Rule::min_term | Rule::minmin_term | Rule::term => {
            let mut pair = pair.into_inner();
            let first = pair.next().expect("first not found");
            match pair.next() {
                Some(oper) => {
                    Operation{
                        left: Box::new(build_ast(first)),
                        operator: oper.as_str().to_string(),
                        right: Box::new(build_ast(pair.next().expect("right not found")))
                    }
                },
                None =>
                    build_ast(first)
            }
        },
        Rule::identifier => {
            Str(pair.as_str().to_owned())
        },
        Rule::integer_value => {
            Int(pair.as_str().parse::<u32>().unwrap())
        },
        _ => Null
    }
}

pub fn vhdl_to_json(vhdl: &String, pretty: bool) -> Result<String, Error<Rule>>{
    match parse(&vhdl) {
        Ok(astnode) => {
            let result: String;
            if pretty {
                result = serde_json::to_string_pretty(&astnode).unwrap();
            }
            else {
                result = serde_json::to_string(&astnode).unwrap();
            }
            Ok(result)
        },
        Err(e) => Err(e)
    }
}

use std::os::raw::{c_char};
use std::ffi::{CStr, CString};

#[no_mangle]
pub extern fn rust_vhdl_as_json(c_buf_input: *const c_char, c_buf_result: *mut *const c_char) -> bool {
    //make &CStr from C characters
    let r_cstr: &CStr = unsafe { CStr::from_ptr(c_buf_input) };
    //make &str from &CStr
    let r_str = r_cstr.to_str().unwrap();
    //make String from accross format macros with the &str as parameter
    match vhdl_to_json(&r_str.to_owned(), true) {
        Ok(json_value) => {
            //make a &CString from String
            let r_cstring = CString::new(json_value).unwrap();
            //make a *const c_char from CString using into_raw to transfer the owning to C
            unsafe { *c_buf_result = r_cstring.into_raw() as *const c_char; }
            //parse ok [c_buf_result as json]
            true
        },
        Err(e) => {
            //error string
            let error_string = format!("[parse error]\n{}", e);
            //make a e*const c_char from CString using into_raw to transfer the owning to C
            unsafe { *c_buf_result = CString::new(error_string).unwrap().into_raw() as *const c_char };
            //parse ok [NO c_buf_result as error message]
            false
        }
    }
}