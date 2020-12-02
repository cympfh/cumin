use crate::parser::typing::*;
use crate::parser::util::*;
use combine::error::ParseError;
use combine::parser::char::{char, digit, string};
use combine::parser::combinator::attempt;
use combine::stream::Stream;
use combine::{any, between, choice, many, many1, none_of, optional, parser};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nat(u128),
    Int(i128),
    Float(f64),
    Bool(bool),
    Str(String),
    Var(String),
    Env(String, Option<String>),
    Dict(Vec<(String, Value)>),
    EnumVariant(String, String),
    Array(Vec<Value>),
    Just(Box<Value>),
    Nothing,
}

impl Value {
    pub fn cast(&self, typ: &Typing) -> Value {
        use Value::*;
        match (self, typ) {
            (Nat(x), Typing::Nat) => Nat(*x),
            (Nat(x), Typing::Int) => Int((*x) as i128),
            (Nat(x), Typing::Float) => Float((*x) as f64),
            (Nat(x), Typing::String) => Str(format!("{}", x)),
            (Int(x), Typing::Nat) => Nat((*x) as u128),
            (Int(x), Typing::Int) => Int(*x),
            (Int(x), Typing::Float) => Float((*x) as f64),
            (Int(x), Typing::String) => Str(format!("{}", x)),
            (Float(x), Typing::Nat) => Nat((*x) as u128),
            (Float(x), Typing::Int) => Int((*x) as i128),
            (Float(x), Typing::Float) => Float(*x),
            (Float(x), Typing::String) => Str(format!("{}", x)),
            (Str(x), Typing::Nat) => Nat(x.parse::<u128>().unwrap()),
            (Str(x), Typing::Int) => Int(x.parse::<i128>().unwrap()),
            (Str(x), Typing::Float) => Float(x.parse::<f64>().unwrap()),
            (Str(x), Typing::Bool) if x.as_str() == "true" => Bool(true),
            (Str(x), Typing::Bool) if x.as_str() == "false" => Bool(false),
            (Str(x), Typing::String) => Str(x.to_string()),
            _ => panic!("Cannot cast {:?} as {:?}", self, typ),
        }
    }
}

fn escaped_character(c: char) -> char {
    match c {
        'n' => '\n',
        't' => '\t',
        'r' => '\r',
        _ => c,
    }
}

parser! {
    pub fn value[Input]()(Input) -> Value
    where [
        Input: Stream<Token = char>,
        Input::Error: ParseError<char, Input::Range, Input::Position>,
    ]
    {
        let none_value = string("None").map(|_| Value::Nothing);
        let bool_value =
            choice!(
                string("true").map(|_| Value::Bool(true)),
                string("false").map(|_| Value::Bool(false)));
        let float_value =
            (
                many1(digit()),
                char('.'),
                many1(digit()),
            ).map(|(head, _, tail): (String, char, String)| {
                let mut num: String = head;
                num.push('.');
                num.push_str(tail.as_str());
                Value::Float(num.parse::<f64>().unwrap())
            });
        let minus_float_value =
            (
                char('-'),
                many1(digit()),
                char('.'),
                many1(digit()),
            ).map(|(_, head, _, tail): (char, String, char, String)| {
                let mut num: String = head;
                num.push('.');
                num.push_str(tail.as_str());
                Value::Float(-num.parse::<f64>().unwrap())
            });
        let int_value =
            (
                char('-'),
                many1(digit()),
            ).map(|(_, x): (char, String)| {
                Value::Int(-x.parse::<i128>().unwrap())
            });
        let nat_value = many1(digit()).map(|x: String| Value::Nat(x.parse::<u128>().unwrap()));
        let str_value =
            (
                char('"'),
                many::<String, _, _>(
                    choice!(
                        attempt(char('\\').with(any()).map(escaped_character)),
                        attempt(none_of("\"".chars())))),
                char('"'),
            ).map(|(_, s, _): (char, String, char)| Value::Str(s));
        let variant_value =
            (
                identifier(),
                string("::"),
                identifier(),
            ).map(|t: (String, &str, String)| Value::EnumVariant(t.0, t.2));
        let env_value = {
            let default_string_value = (
                string(":-"),
                many::<String, _, _>(none_of("}".chars())),
            ).map(|t| t.1);
            (
                char('$'),
                choice!(
                    identifier().map(|s: String| (s, None)),
                    between(char('{'), char('}'), (identifier(), optional(default_string_value)))
                ),
            ).map(|(_, (name, default_value)): (char, (String, Option<String>))| {
                Value::Env(name, default_value)
            })
        };
        let var_value = identifier().map(Value::Var);

        choice!(
            attempt(minus_float_value),
            attempt(float_value),
            int_value,
            nat_value,
            str_value,
            env_value,
            attempt(variant_value),
            attempt(none_value),
            attempt(bool_value),
            var_value
        )
    }
}

#[cfg(test)]
mod test_value {
    use crate::parser::value::*;
    use combine::Parser;
    use Value::*;

    macro_rules! assert_value {
        ($code: expr, $expected: expr) => {
            assert_eq!(value().parse($code).ok().unwrap(), $expected);
        };
    }
    #[test]
    fn test_parse() {
        assert_value!("23", (Value::Nat(23), ""));
        assert_value!("23_", (Value::Nat(23), "_"));
        assert_value!("-23_", (Value::Int(-23), "_"));
        assert_value!("0.5", (Value::Float(0.5), ""));
        assert_value!("21.5", (Value::Float(21.5), ""));
        assert_value!("-0.5", (Value::Float(-0.5), ""));
        assert_value!("-21.5", (Value::Float(-21.5), ""));
        assert_value!("true", (Value::Bool(true), ""));
        assert_value!("false", (Value::Bool(false), ""));
        assert_value!("\"hoge\"", (Value::Str("hoge".to_string()), ""));
        assert_value!("\"hoge !?\"", (Value::Str("hoge !?".to_string()), ""));
        assert_value!("\"ho\\nge\"", (Value::Str("ho\nge".to_string()), ""));
        assert_value!("\"ho\\\"ge\"", (Value::Str("ho\"ge".to_string()), ""));
        assert_value!("\"ho\\\\ge\\'\"", (Value::Str("ho\\ge'".to_string()), ""));
        assert_value!("hoge", (Value::Var("hoge".to_string()), ""));
        assert_value!("_hoge0", (Value::Var("_hoge0".to_string()), ""));
        assert_value!("$USER", (Value::Env("USER".to_string(), None), ""));
        assert_value!(
            "${USER_iD2}",
            (Value::Env("USER_iD2".to_string(), None), "")
        );
        assert_value!(
            "${X:-hoge}",
            (Value::Env("X".to_string(), Some("hoge".to_string())), "")
        );
        assert_value!(
            "X::Zoo",
            (Value::EnumVariant("X".to_string(), "Zoo".to_string()), "")
        );
        assert_value!("None", (Value::Nothing, ""));
    }

    #[test]
    fn test_cast() {
        assert_eq!(Nat(0).cast(&Typing::Nat), Nat(0));
        assert_eq!(Nat(0).cast(&Typing::Int), Int(0));
        assert_eq!(Nat(0).cast(&Typing::String), Str("0".to_string()));
        assert_eq!(Int(0).cast(&Typing::Nat), Nat(0));
        assert_eq!(Int(0).cast(&Typing::Int), Int(0));
        assert_eq!(Int(0).cast(&Typing::String), Str("0".to_string()));
        assert_eq!(Str("0".to_string()).cast(&Typing::Nat), Nat(0));
        assert_eq!(Str("0".to_string()).cast(&Typing::Int), Int(0));
        assert_eq!(
            Str("0".to_string()).cast(&Typing::String),
            Str("0".to_string())
        );
        assert_eq!(Str("true".to_string()).cast(&Typing::Bool), Bool(true));
        assert_eq!(Str("false".to_string()).cast(&Typing::Bool), Bool(false));
    }
}
