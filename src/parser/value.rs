use crate::parser::typing::*;
use crate::parser::util::*;
use combine::error::ParseError;
use combine::parser::char::{char, digit, string};
use combine::parser::combinator::attempt;
use combine::stream::Stream;
use combine::{between, choice, many, many1, none_of, optional, parser, token};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nat(u128),
    Int(i128),
    Float(f64),
    Str(String),
    Var(String),
    Env(String, Option<String>),
    Dict(Vec<(String, Value)>),
    EnumVariant(String, String),
    Array(Vec<Value>),
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
            (Str(x), Typing::String) => Str(x.to_string()),
            _ => panic!("Cannot cast {:?} into {:?}", self, typ),
        }
    }
}

parser! {
    pub fn value[Input]()(Input) -> Value
    where [
        Input: Stream<Token = char>,
        Input::Error: ParseError<char, Input::Range, Input::Position>,
    ]
    {
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
        let str_value = between(token('"'), token('"'), many(none_of("\"".chars())).map(|x:String| Value::Str(x)));
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
            var_value
        )
    }
}

#[cfg(test)]
mod test_value {
    use crate::parser::value::*;
    use combine::Parser;
    use Value::*;

    #[test]
    fn test_parse() {
        assert_eq!(value().parse("23"), Ok((Value::Nat(23), "")));
        assert_eq!(value().parse("23_"), Ok((Value::Nat(23), "_")));
        assert_eq!(value().parse("-23_"), Ok((Value::Int(-23), "_")));
        assert_eq!(value().parse("0.5"), Ok((Value::Float(0.5), "")));
        assert_eq!(value().parse("21.5"), Ok((Value::Float(21.5), "")));
        assert_eq!(value().parse("-0.5"), Ok((Value::Float(-0.5), "")));
        assert_eq!(value().parse("-21.5"), Ok((Value::Float(-21.5), "")));
        assert_eq!(
            value().parse("\"hoge\""),
            Ok((Value::Str("hoge".to_string()), ""))
        );
        assert_eq!(
            value().parse("\"hoge !?\""),
            Ok((Value::Str("hoge !?".to_string()), ""))
        );
        assert_eq!(
            value().parse("hoge"),
            Ok((Value::Var("hoge".to_string()), ""))
        );
        assert_eq!(
            value().parse("_hoge0"),
            Ok((Value::Var("_hoge0".to_string()), ""))
        );
        assert_eq!(
            value().parse("$USER"),
            Ok((Value::Env("USER".to_string(), None), ""))
        );
        assert_eq!(
            value().parse("${USER_iD2}"),
            Ok((Value::Env("USER_iD2".to_string(), None), ""))
        );
        assert_eq!(
            value().parse("${X:-hoge}"),
            Ok((Value::Env("X".to_string(), Some("hoge".to_string())), ""))
        );
        assert_eq!(
            value().parse("X::Zoo"),
            Ok((Value::EnumVariant("X".to_string(), "Zoo".to_string()), ""))
        );
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
    }
}
