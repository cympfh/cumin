use crate::parser::value::*;

#[derive(Debug, Clone, PartialEq)]
pub enum JSON {
    Nat(u128),
    Int(i128),
    Float(f64),
    Bool(bool),
    Str(String),
    Array(Vec<JSON>),
    Dict(Vec<(String, JSON)>),
    Null,
}

impl JSON {
    pub fn stringify(&self) -> String {
        use JSON::*;
        match self {
            Nat(x) => format!("{}", x),
            Int(x) => format!("{}", x),
            Float(x) => format!("{}", x),
            Bool(x) => format!("{:?}", x),
            Str(x) => format!("{:?}", x),
            Array(xs) => format!(
                "[{}]",
                xs.iter()
                    .map(|j| j.stringify())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Dict(d) => format!(
                "{{{}}}",
                d.iter()
                    .map(|(key, val)| format!("\"{}\":{}", key, val.stringify()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Null => "null".to_string(),
        }
    }
    pub fn from_cumin(val: Value) -> Self {
        use JSON::*;
        match val {
            Value::Nat(x) => Nat(x),
            Value::Int(x) => Int(x),
            Value::Float(x) => Float(x),
            Value::Bool(x) => Bool(x),
            Value::Str(x) => Str(x),
            Value::Env(v, _) => panic!("Env {} is unresolved", v),
            Value::Dict(_name, items) => {
                let items: Vec<(String, JSON)> = items
                    .iter()
                    .map(|(key, val)| (key.to_string(), JSON::from_cumin((*val).clone())))
                    .collect();
                Dict(items)
            }
            Value::EnumVariant(_, t) => Str(t),
            Value::Array(_typ, elements) => {
                let elements = elements
                    .iter()
                    .map(|e| JSON::from_cumin((*e).clone()))
                    .collect();
                Array(elements)
            }
            Value::Tuple(elements) => {
                let elements = elements
                    .iter()
                    .map(|e| JSON::from_cumin((*e).clone()))
                    .collect();
                Array(elements)
            }
            Value::Optional(_typ, val) => match *val {
                Some(x) => JSON::from_cumin(x),
                None => JSON::Null,
            },
            Value::Wrapped(_typ, val) => JSON::from_cumin(*val),
        }
    }
}

#[cfg(test)]
mod test_json {
    use crate::json::*;
    use JSON::*;

    #[test]
    fn test_stringify() {
        assert_eq!(Nat(3).stringify(), "3".to_string());
        assert_eq!(Int(-3).stringify(), "-3".to_string());
        assert_eq!(Bool(true).stringify(), "true".to_string());
        assert_eq!(Bool(false).stringify(), "false".to_string());
        assert_eq!(
            Dict(vec![
                ("arr".to_string(), Array(vec![Nat(1), Nat(2), Nat(3)])),
                ("str".to_string(), Str("Hello".to_string())),
                ("str_complicated".to_string(), Str("He\nl\tlo\"".to_string())),
                ("dict_empty".to_string(), Dict(vec![])),
            ])
            .stringify(),
            "{\"arr\":[1,2,3],\"str\":\"Hello\",\"str_complicated\":\"He\\nl\\tlo\\\"\",\"dict_empty\":{}}".to_string()
        );
        assert_eq!(
            Array(vec![Array(vec![]), Nat(1), Nat(2), Str("3".to_string())]).stringify(),
            "[[],1,2,\"3\"]"
        );
        assert_eq!(Array(vec![Null, Nat(1)]).stringify(), "[null,1]");
    }
}
