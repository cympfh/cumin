use crate::parser::typing::*;
use crate::parser::util::*;
use anyhow::Result;
use combine::error::ParseError;
use combine::parser::char::{char, digit, string};
use combine::parser::combinator::attempt;
use combine::stream::Stream;
use combine::{any, between, choice, many, many1, none_of, one_of, optional, parser};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nat(u128),
    Int(i128),
    Float(f64),
    Bool(bool),
    Str(String),
    Var(String),
    Env(String, Option<String>),
    Dict(Option<String>, Vec<(String, Value)>),
    EnumVariant(String, String),
    Array(Vec<Value>),
    Just(Box<Value>),
    Nothing,
}

impl Value {
    pub fn type_of(&self) -> Option<Typing> {
        match self {
            Value::Nat(_) => Some(Typing::Nat),
            Value::Int(_) => Some(Typing::Int),
            Value::Float(_) => Some(Typing::Float),
            Value::Bool(_) => Some(Typing::Bool),
            Value::Str(_) | Value::Env(_, _) => Some(Typing::String),
            Value::Dict(Some(name), _) | Value::EnumVariant(name, _) => {
                Some(Typing::UserTyping(name.to_string()))
            }
            Value::Array(elems) => {
                for elem in elems {
                    if let Some(typ) = elem.type_of() {
                        return Some(Typing::Array(Box::new(typ)));
                    }
                }
                None
            }
            Value::Just(val) => val.type_of().map(|typ| Typing::Option(Box::new(typ))),
            _ => None,
        }
    }

    pub fn cast(&self, typ: &Typing) -> Result<Value> {
        use Value::*;
        let ret = match (self, typ) {
            (_, Typing::Any) => self.clone(),
            (Nat(_), Typing::Nat) => self.clone(),
            (Nat(x), Typing::Int) => Int((*x) as i128),
            (Nat(x), Typing::Float) => Float((*x) as f64),
            (Int(_), Typing::Int) => self.clone(),
            (Int(x), Typing::Float) => Float((*x) as f64),
            (Float(_), Typing::Float) => self.clone(),
            (Bool(_), Typing::Bool) => self.clone(),
            (Str(_), Typing::String) => self.clone(),
            (Array(elems), Typing::Array(t)) => {
                let elems = elems.iter().map(|val| val.cast(t)).collect::<Result<_>>()?;
                Array(elems)
            }
            (Just(x), Typing::Option(t)) => {
                let val = x.cast(t)?;
                Just(Box::new(val))
            }
            (Nothing, Typing::Option(_)) => self.clone(),
            (Dict(dict_name, _), Typing::UserTyping(type_name))
                if &Some(type_name.to_string()) == dict_name =>
            {
                self.clone()
            }
            (EnumVariant(enum_name, _), Typing::UserTyping(type_name))
                if enum_name == type_name =>
            {
                self.clone()
            }
            _ => bail!("Cannot cast {:?} => {:?}", self, typ),
        };
        Ok(ret)
    }
    pub fn coerce(&self, typ: &Typing) -> Result<Value> {
        use Value::*;
        let ret = match (self, typ) {
            (Nat(x), Typing::String) => Str(format!("{}", x)),
            (Int(x), Typing::Nat) => Nat((*x) as u128),
            (Int(x), Typing::String) => Str(format!("{}", x)),
            (Float(x), Typing::Nat) => Nat((*x) as u128),
            (Float(x), Typing::Int) => Int((*x) as i128),
            (Float(x), Typing::String) => Str(format!("{}", x)),
            (Str(x), Typing::Nat) => Nat(x.parse::<u128>().unwrap()),
            (Str(x), Typing::Int) => Int(x.parse::<i128>().unwrap()),
            (Str(x), Typing::Float) => Float(x.parse::<f64>().unwrap()),
            (Str(x), Typing::Bool) if x.as_str() == "true" => Bool(true),
            (Str(x), Typing::Bool) if x.as_str() == "false" => Bool(false),
            _ => self.cast(typ)?,
        };
        Ok(ret)
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
        let const_value =
            choice!(
                string("None").map(|_| Value::Nothing),
                string("true").map(|_| Value::Bool(true)),
                string("false").map(|_| Value::Bool(false)));

        let float_value =
            (
                optional(char('-')),
                many1(digit()),
                many(one_of("_0123456789".chars())),
                char('.'),
                many(one_of("_0123456789".chars())),
            ).map(|(sign, head, tail, _dot, under): (Option<char>, String, String, char, String)| {
                let mut s: String = sign.iter().collect();
                s.push_str(head.as_str());
                s.push_str(tail.as_str());
                s.push('.');
                s.push_str(under.as_str());
                let s: String = s.chars().filter(|&c| c != '_').collect();
                Value::Float(s.parse::<f64>().unwrap())
            });

        let num_value =
            (
                optional(char('-')),
                many1(digit()),
                many(one_of("_0123456789".chars())),
            ).map(|(sign, head, tail): (Option<char>, String, String)| {
                let mut s: String = sign.iter().collect();
                s.push_str(head.as_str());
                s.push_str(tail.as_str());
                let s: String = s.chars().filter(|&c| c != '_').collect();
                if sign.is_none() {
                    Value::Nat(s.parse::<u128>().unwrap())
                } else {
                    Value::Int(s.parse::<i128>().unwrap())
                }
            });

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
            attempt(float_value),
            attempt(num_value),
            attempt(str_value),
            attempt(env_value),
            attempt(variant_value),
            attempt(const_value),
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
            assert_eq!(value().parse($code).ok().unwrap().0, $expected);
        };
    }

    #[test]
    fn test_num() {
        assert_value!("23", Value::Nat(23));
        assert_value!("23_", Value::Nat(23));
        assert_value!("-23_", Value::Int(-23));
        assert_value!("0.5", Value::Float(0.5));
        assert_value!("21.5", Value::Float(21.5));
        assert_value!("-0.5", Value::Float(-0.5));
        assert_value!("-21.5", Value::Float(-21.5));
    }
    #[test]
    fn test_const() {
        assert_value!("true", Value::Bool(true));
        assert_value!("false", Value::Bool(false));
        assert_value!("None", Value::Nothing);
    }
    #[test]
    fn test_str() {
        assert_value!("\"hoge\"", Value::Str("hoge".to_string()));
        assert_value!("\"hoge !?\"", Value::Str("hoge !?".to_string()));
        assert_value!("\"ho\\nge\"", Value::Str("ho\nge".to_string()));
        assert_value!("\"ho\\\"ge\"", Value::Str("ho\"ge".to_string()));
        assert_value!("\"ho\\\\ge\\'\"", Value::Str("ho\\ge'".to_string()));
    }
    #[test]
    fn test_var() {
        assert_value!("hoge", Value::Var("hoge".to_string()));
        assert_value!("_hoge0", Value::Var("_hoge0".to_string()));
        assert_value!("$USER", Value::Env("USER".to_string(), None));
        assert_value!("${USER_iD2}", Value::Env("USER_iD2".to_string(), None));
    }
    #[test]
    fn test_enum() {
        assert_value!(
            "${X:-hoge}",
            Value::Env("X".to_string(), Some("hoge".to_string()))
        );
        assert_value!(
            "X::Zoo",
            Value::EnumVariant("X".to_string(), "Zoo".to_string())
        );
    }

    macro_rules! assert_cast {
        ($val:expr, $type:expr, $expected:expr) => {
            assert_eq!($val.cast(&$type).unwrap(), $expected)
        };
    }

    #[test]
    fn test_cast() {
        assert_cast!(Nat(0), Typing::Nat, Nat(0));
        assert_cast!(Nat(0), Typing::Int, Int(0));
        assert_cast!(Nat(0), Typing::Float, Float(0.0));
        assert_cast!(Int(0), Typing::Int, Int(0));
        assert_cast!(Int(0), Typing::Float, Float(0.0));
        assert_cast!(Str("0".to_string()), &Typing::String, Str("0".to_string()));
        assert_cast!(Bool(true), Typing::Bool, Bool(true));
        assert_cast!(Bool(false), Typing::Bool, Bool(false));
        assert_cast!(Nothing, Typing::Option(Box::new(Typing::Int)), Nothing);
        assert_cast!(
            Just(Box::new(Nat(0))),
            Typing::Option(Box::new(Typing::Int)),
            Just(Box::new(Int(0)))
        );
        assert_cast!(
            Array(vec![Nat(0), Int(-1), Float(0.5)]),
            Typing::Array(Box::new(Typing::Float)),
            Array(vec![Float(0.0), Float(-1.0), Float(0.5)])
        );
    }

    macro_rules! assert_coerce {
        ($val:expr, $type:expr, $expected:expr) => {
            assert_eq!($val.coerce(&$type).unwrap(), $expected)
        };
    }

    #[test]
    fn test_coerce() {
        assert_coerce!(Nat(0), Typing::String, Str("0".to_string()));
        assert_coerce!(Int(0), Typing::String, Str("0".to_string()));
        assert_coerce!(Int(0), Typing::Nat, Nat(0));
        assert_coerce!(Str("0".to_string()), Typing::Nat, Nat(0));
        assert_coerce!(Str("0".to_string()), Typing::Int, Int(0));
        assert_coerce!(Str("true".to_string()), Typing::Bool, Bool(true));
        assert_coerce!(Str("false".to_string()), Typing::Bool, Bool(false));
    }

    macro_rules! assert_type_of {
        ($val:expr, $type:expr) => {
            assert_eq!($val.type_of(), $type);
        };
    }

    #[test]
    fn test_type_of() {
        assert_type_of!(Value::Int(1), Some(Typing::Int));
        assert_type_of!(Value::Nothing, None);
        assert_type_of!(
            Value::Just(Box::new(Value::Nat(2))),
            Some(Typing::Option(Box::new(Typing::Nat)))
        );
        assert_type_of!(Value::Array(vec![]), None);
        assert_type_of!(
            Value::Array(vec![Value::Int(1)]),
            Some(Typing::Array(Box::new(Typing::Int)))
        );
    }
}
