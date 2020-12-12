use crate::parser::typing::*;
use crate::parser::value::Value;
use anyhow::Result;

pub fn concat(args: &Vec<Value>) -> Result<Value> {
    let mut r = vec![];
    let mut t = Typing::Any;
    for arr in args {
        match arr {
            Value::Array(typ, xs) => {
                if let Some(unified) = Typing::unify(&t, &typ) {
                    t = unified;
                    r.extend(xs.iter().cloned())
                } else {
                    bail!("Cannot concat Array of {:?} and Array of {:?}", &t, &typ)
                }
            }
            _ => bail!("Cannot concat {:?}, because this is not array", arr),
        }
    }
    Ok(Value::Array(t, r))
}

pub fn reverse(x: &Value) -> Result<Value> {
    match x {
        Value::Array(t, xs) => {
            let r = xs.iter().rev().cloned().collect();
            Ok(Value::Array(t.clone(), r))
        }
        _ => bail!("Cannot reverse {:?}, because this is not array", x),
    }
}
