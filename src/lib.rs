#[macro_use]
extern crate anyhow;
extern crate nom;

pub mod builtins;
pub mod errors;
pub mod eval;
pub mod eval_wasm;
pub mod json;
pub mod parser;

use crate::{eval_wasm::eval_wasm, parser::cumin::cumin};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compile(input: &str) -> String {
    match cumin(input) {
        Ok((rest, data)) => {
            if rest == "" {
                match eval_wasm(data) {
                    Ok(data) => data.stringify(),
                    Err(err) => format!("Error: eval failed ({:?})", err),
                }
            } else {
                format!("Error: Parsing stopped at {}", &rest)
            }
        }
        Err(err) => format!("Error: Parsing failed ({:?})", &err),
    }
}
