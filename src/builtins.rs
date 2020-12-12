use crate::parser::value::Value;
use anyhow::Result;

pub fn concat(args: &Vec<Value>) -> Result<Value> {
    let mut r = vec![];
    for arr in args {
        match arr {
            Value::Array(xs) => r.extend(xs.iter().cloned()),
            _ => bail!("Cannot concat {:?}, because this is not array", arr),
        }
    }
    Ok(Value::Array(r))
}

pub fn reverse(x: &Value) -> Result<Value> {
    match x {
        Value::Array(xs) => {
            let r = xs.iter().rev().cloned().collect();
            Ok(Value::Array(r))
        }
        _ => bail!("Cannot reverse {:?}, because this is not array", x),
    }
}
