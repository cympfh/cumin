use crate::parser::util::identifier;
use combine::error::ParseError;
use combine::parser::char::{char, spaces, string};
use combine::parser::combinator::attempt;
use combine::stream::Stream;
use combine::{choice, parser};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Typing {
    Any,
    Nat,
    Int,
    Float,
    String,
    Array(Box<Typing>),
    Option(Box<Typing>),
    UserTyping(String),
}

parser! {
    pub fn typing[Input]()(Input) -> Typing
    where [
        Input: Stream<Token = char>,
        Input::Error: ParseError<char, Input::Range, Input::Position>,
    ]
    {
        let any_typing = string("Any").map(|_| Typing::Any);
        let nat_typing = string("Nat").map(|_| Typing::Nat);
        let int_typing = string("Int").map(|_| Typing::Int);
        let float_typing = string("Float").map(|_| Typing::Float);
        let string_typing = string("String").map(|_| Typing::String);
        let array_typing = (
            string("Array<"),
            spaces(),
            typing(),
            spaces(),
            char('>'),
        ).map(|(_, _, elements, _, _): (&str, (), Typing, (), char)| Typing::Array(Box::new(elements)));
        let option_typing = (
            string("Option<"),
            spaces(),
            typing(),
            spaces(),
            char('>'),
        ).map(|(_, _, elements, _, _): (&str, (), Typing, (), char)| Typing::Option(Box::new(elements)));
        let user_typing = identifier().map(Typing::UserTyping);
        choice!(
            attempt(array_typing),
            attempt(option_typing),
            any_typing,
            nat_typing,
            int_typing,
            float_typing,
            string_typing,
            user_typing
        )
    }
}

#[cfg(test)]
mod test_typing {
    use crate::parser::typing::*;
    use combine::Parser;

    #[test]
    fn test() {
        assert_eq!(typing().parse("Any"), Ok((Typing::Any, "")));
        assert_eq!(typing().parse("Nat"), Ok((Typing::Nat, "")));
        assert_eq!(typing().parse("Int"), Ok((Typing::Int, "")));
        assert_eq!(typing().parse("Float"), Ok((Typing::Float, "")));
        assert_eq!(typing().parse("String"), Ok((Typing::String, "")));
        assert_eq!(
            typing().parse("Array<String>"),
            Ok((Typing::Array(Box::new(Typing::String)), ""))
        );
        assert_eq!(
            typing().parse("Array<Array<String>>"),
            Ok((
                Typing::Array(Box::new(Typing::Array(Box::new(Typing::String)))),
                ""
            ))
        );
        assert_eq!(
            typing().parse("Option<String>"),
            Ok((Typing::Option(Box::new(Typing::String)), ""))
        );
        assert_eq!(
            typing().parse("Option<Array<String>>"),
            Ok((
                Typing::Option(Box::new(Typing::Array(Box::new(Typing::String)))),
                ""
            ))
        );
        assert_eq!(
            typing().parse("Option<Option<Array<Int>>>"),
            Ok((
                Typing::Option(Box::new(Typing::Option(Box::new(Typing::Array(Box::new(
                    Typing::Int
                )))))),
                ""
            ))
        );
        assert_eq!(
            typing().parse("Hoge_type"),
            Ok((Typing::UserTyping("Hoge_type".to_string()), ""))
        );
    }
}