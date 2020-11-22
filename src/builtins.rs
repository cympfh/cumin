use crate::parser::value::Value;

pub fn concat(args: &Vec<Value>) -> Value {
    let mut r = vec![];
    for arr in args {
        match arr {
            Value::Array(xs) => r.extend(xs.iter().cloned()),
            _ => panic!("Cannot concat {:?}", arr),
        }
    }
    Value::Array(r)
}

pub fn reverse(x: &Value) -> Value {
    match x {
        Value::Array(xs) => {
            let r = xs.iter().rev().cloned().collect();
            Value::Array(r)
        }
        _ => panic!("Cannot reverse {:?}", x),
    }
}
