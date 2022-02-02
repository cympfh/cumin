use crate::eval::{eval_cumin, Environ};
use crate::json::JSON;
use crate::parser::cumin::Cumin;
use anyhow::Result;

pub fn eval_wasm(cumin: Cumin) -> Result<JSON> {
    let mut env = Environ::wasm();
    let val = eval_cumin(&mut env, &cumin)?;
    Ok(JSON::from_cumin(val))
}
