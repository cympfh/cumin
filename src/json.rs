use crate::parser::value::*;

#[derive(Debug, Clone, PartialEq)]
pub enum JSON {
    Nat(u128),
    Int(i128),
    Float(f64),
    Str(String),
    Array(Vec<JSON>),
    Dict(Vec<(String, JSON)>),
}

impl JSON {
    pub fn stringify(&self) -> String {
        use JSON::*;
        match self {
            Nat(x) => format!("{}", x),
            Int(x) => format!("{}", x),
            Float(x) => format!("{}", x),
            Str(x) => format!("\"{}\"", x),
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
        }
    }
    pub fn from_cumin(val: Value) -> Self {
        use JSON::*;
        match val {
            Value::Nat(x) => Nat(x),
            Value::Int(x) => Int(x),
            Value::Str(x) => Str(x),
            Value::Var(v) => panic!("Var {} is unresolved", v),
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
        assert_eq!(
            Dict(vec![
                ("arr".to_string(), Array(vec![Nat(1), Nat(2), Nat(3)])),
                ("str".to_string(), Str("Hello".to_string())),
                ("dict_empty".to_string(), Dict(vec![])),
            ])
            .stringify(),
            "{\"arr\":[1,2,3],\"str\":\"Hello\",\"dict_empty\":{}}".to_string()
        );
    }
}
