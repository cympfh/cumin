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
pub fn load(input: &str) -> String {
    if let Ok((_, data)) = cumin(input) {
        if let Ok(data) = eval_wasm(data) {
            return data.stringify();
        }
    }
    String::new()
}
