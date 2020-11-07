use combine::parser::char::{alpha_num, char, digit, spaces};
use combine::stream::Stream;
use combine::{between, choice, many, many1, none_of, parser, token, Parser};

#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    Nat(u128),
    Int(i128),
    Str(String),
    Var(String),
}

parser! {
    pub fn value[Input]()(Input) -> Value where [Input: Stream<Token=char>] {
        let int_value = char('-')
            .with(many1(digit()))
            .map(|x: String| Value::Int(-x.parse::<i128>().unwrap()));
        let nat_value = many1(digit()).map(|x: String| Value::Nat(x.parse::<u128>().unwrap()));
        let str_value = between(token('"'), token('"'), many(none_of("\"".chars())).map(|x:String| Value::Str(x)));
        let var_value = many(alpha_num()).map(|x: String| Value::Var(x));

        choice!(int_value, nat_value, str_value, var_value).skip(spaces())
    }
}

#[cfg(test)]
mod test_value {
    use crate::parser::value::*;

    #[test]
    fn test() {
        assert_eq!(value().parse("23 "), Ok((Value::Nat(23), "")));
        assert_eq!(value().parse("23x"), Ok((Value::Nat(23), "x")));
        assert_eq!(value().parse("-23 _"), Ok((Value::Int(-23), "_")));
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
    }
}
