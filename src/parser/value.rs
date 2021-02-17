use crate::parser::typing::*;
use crate::parser::util::*;
use anyhow::Result;
use nom::combinator;
use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, is_not, tag},
    character::complete::{char, one_of},
    combinator::{map, opt, recognize},
    multi::{many0, many1},
    sequence::{delimited, pair, terminated, tuple},
    IResult,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nat(u128),
    Int(i128),
    Float(f64),
    Bool(bool),
    Str(String),
    Env(String, Option<String>),
    Dict(Option<String>, Vec<(String, Value)>),
    EnumVariant(String, String),
    Array(Typing, Vec<Value>),
    Optional(Typing, Box<Option<Value>>),
    Wrapped(Typing, Box<Value>),
}

impl Value {
    pub fn type_of(&self) -> Typing {
        match self {
            Value::Nat(_) => Typing::Nat,
            Value::Int(_) => Typing::Int,
            Value::Float(_) => Typing::Float,
            Value::Bool(_) => Typing::Bool,
            Value::Str(_) | Value::Env(_, _) => Typing::String,
            Value::Dict(Some(name), _) | Value::EnumVariant(name, _) => {
                Typing::UserTyping(name.to_string())
            }
            Value::Array(typ, _) => Typing::Array(Box::new(typ.clone())),
            Value::Optional(typ, _) => Typing::Option(Box::new(typ.clone())),
            Value::Wrapped(typ, _) => typ.clone(),
            _ => Typing::Any,
        }
    }

    pub fn cast(&self, typ: &Typing) -> Result<Value> {
        use Value::*;
        let ret = match (self, typ) {
            (_, Typing::Any) => self.clone(),
            (val, typ) if &val.type_of() == typ => self.clone(),
            (Nat(x), Typing::Int) => Int((*x) as i128),
            (Nat(x), Typing::Float) => Float((*x) as f64),
            (Int(x), Typing::Float) => Float((*x) as f64),
            (Array(s, elems), Typing::Array(t)) => {
                if let Some(typ) = Typing::unify(s, t) {
                    let elems: Vec<Value> = elems
                        .iter()
                        .map(|val| val.cast(&typ))
                        .collect::<Result<_>>()?;
                    let elems = elems
                        .iter()
                        .map(|val| val.cast(&t))
                        .collect::<Result<_>>()?;
                    Array(typ, elems)
                } else {
                    bail!("Cannot unify Array<{:?}> and Array<{:?}>", &s, &t);
                }
            }
            (Optional(s, val), Typing::Option(t)) => {
                if let Some(typ) = Typing::unify(s, t) {
                    match &**val {
                        Some(x) => {
                            let val = x.cast(&typ)?;
                            let val = val.cast(&t)?;
                            Optional(typ, Box::new(Some(val)))
                        }
                        None => Optional(typ, Box::new(None)),
                    }
                } else {
                    bail!("Cannot unify Option<{:?}> and Option<{:?}>", &s, &t);
                }
            }
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
            _ => bail!("No ways to cast {:?} => {:?}", self, typ),
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

pub fn value(input: &str) -> IResult<&str, Value> {
    let const_values = alt((
        combinator::value(Value::Optional(Typing::Any, Box::new(None)), tag("None")),
        combinator::value(Value::Bool(true), tag("true")),
        combinator::value(Value::Bool(false), tag("false")),
    ));

    fn decimal(input: &str) -> IResult<&str, &str> {
        recognize(many1(terminated(one_of("0123456789"), many0(char('_')))))(input)
    }

    let float_value = map(
        alt((
            recognize(tuple((opt(char('-')), char('.'), decimal))),
            recognize(tuple((opt(char('-')), decimal, char('.'), decimal))),
        )),
        |num_str: &str| {
            let num: String = num_str.chars().filter(|&c| c != '_').collect();
            let x: f64 = num.parse().unwrap();
            Value::Float(x)
        },
    );

    let num_value = map(pair(opt(tag("-")), decimal), |(sign, num): (_, &str)| {
        let num: String = num.chars().filter(|&c| c != '_').collect();
        match sign {
            None => Value::Nat(num.parse::<u128>().unwrap()),
            _ => Value::Int(-num.parse::<i128>().unwrap()),
        }
    });

    let str_value = alt((
        combinator::value(Value::Str(String::new()), tag("\"\"")),
        map(
            delimited(
                tag("\""),
                escaped_transform(
                    is_not("\"\\"),
                    '\\',
                    alt((
                        combinator::value("\\", tag("\\")),
                        combinator::value("\"", tag("\"")),
                        combinator::value("\'", tag("\'")),
                        combinator::value("\n", tag("n")),
                        combinator::value("\r", tag("r")),
                        combinator::value("\t", tag("t")),
                    )),
                ),
                tag("\""),
            ),
            Value::Str,
        ),
    ));

    let variant_value = map(tuple((identifier, tag("::"), identifier)), |(x, _, y)| {
        Value::EnumVariant(x, y)
    });

    let env_value = {
        let default_value = map(tuple((tag(":-"), is_not("}"))), |(_, val): (_, &str)| {
            val.to_string()
        });
        alt((
            map(
                tuple((tag("${"), identifier, opt(default_value), tag("}"))),
                |(_, name, default, _)| Value::Env(name, default),
            ),
            map(tuple((tag("$"), identifier)), |(_, name)| {
                Value::Env(name, None)
            }),
        ))
    };

    alt((
        const_values,
        float_value,
        num_value,
        str_value,
        variant_value,
        env_value,
    ))(input)
}

#[cfg(test)]
mod test_value {
    use crate::parser::value::*;
    use Value::*;

    macro_rules! assert_value {
        ($code: expr, $expected: expr) => {
            assert_eq!(value($code), Ok(("", $expected)));
        };
    }

    #[test]
    fn test_num() {
        assert_value!("0", Value::Nat(0));
        assert_value!("123", Value::Nat(123));
        assert_value!("-123", Value::Int(-123));
        assert_value!("123_456_789", Value::Nat(123_456_789));
        assert_value!("0.0", Value::Float(0.0));
        assert_value!("0.5", Value::Float(0.5));
        assert_value!("-0.5", Value::Float(-0.5));
        assert_value!("100_000.0", Value::Float(100000.0));
        assert_value!("0.000_000_001", Value::Float(0.000000001));
        assert_value!("123_456.000_000_001", Value::Float(123456.000000001));
    }
    #[test]
    fn test_const() {
        assert_value!("true", Value::Bool(true));
        assert_value!("false", Value::Bool(false));
        assert_value!("None", Value::Optional(Typing::Any, Box::new(None)));
    }
    #[test]
    fn test_str() {
        assert_value!("\"\"", Value::Str("".to_string()));
        assert_value!("\"hoge\"", Value::Str("hoge".to_string()));
        assert_value!("\"hoge !?\"", Value::Str("hoge !?".to_string()));
        assert_value!("\"ho\\nge\"", Value::Str("ho\nge".to_string()));
        assert_value!("\"ho\\\"ge\"", Value::Str("ho\"ge".to_string()));
        assert_value!("\"ho\\\\ge\\'\"", Value::Str("ho\\ge'".to_string()));
        assert_value!(
            "\"[\\n\\r\\t][\\\\][\\\"\\\']\"",
            Value::Str("[\n\r\t][\\][\"\']".to_string())
        );
    }
    #[test]
    fn test_enum() {
        assert_value!(
            "X::Zoo",
            Value::EnumVariant("X".to_string(), "Zoo".to_string())
        );
    }
    #[test]
    fn test_env() {
        assert_value!("$USER", Value::Env("USER".to_string(), None));
        assert_value!("${USER}", Value::Env("USER".to_string(), None));
        assert_value!(
            "${USER:-hoge}",
            Value::Env("USER".to_string(), Some("hoge".to_string()))
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
        assert_cast!(
            Optional(Typing::Any, Box::new(None)),
            Typing::Option(Box::new(Typing::Int)),
            Optional(Typing::Int, Box::new(None))
        );
        assert_cast!(
            Optional(Typing::Nat, Box::new(Some(Nat(0)))),
            Typing::Option(Box::new(Typing::Int)),
            Optional(Typing::Int, Box::new(Some(Int(0))))
        );
        assert_cast!(
            Array(Typing::Any, vec![Nat(0), Int(-1), Float(0.5)]),
            Typing::Array(Box::new(Typing::Float)),
            Array(Typing::Float, vec![Float(0.0), Float(-1.0), Float(0.5)])
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
        assert_type_of!(Value::Int(1), Typing::Int);
        assert_type_of!(
            Value::Optional(Typing::Any, Box::new(None)),
            Typing::Option(Box::new(Typing::Any))
        );
        assert_type_of!(
            Value::Optional(Typing::Nat, Box::new(Some(Value::Nat(2)))),
            Typing::Option(Box::new(Typing::Nat))
        );
        assert_type_of!(
            Value::Array(Typing::Any, vec![]),
            Typing::Array(Box::new(Typing::Any))
        );
        assert_type_of!(
            Value::Array(Typing::Nat, vec![]),
            Typing::Array(Box::new(Typing::Nat))
        );
        assert_type_of!(
            Value::Array(Typing::Int, vec![Value::Int(1)]),
            Typing::Array(Box::new(Typing::Int))
        );
    }
}
