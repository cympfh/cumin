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
    Bool,
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
        let bool_typing = string("Bool").map(|_| Typing::Bool);
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
            attempt(any_typing),
            attempt(nat_typing),
            attempt(int_typing),
            attempt(float_typing),
            attempt(bool_typing),
            attempt(string_typing),
            user_typing
        )
    }
}

impl Typing {
    pub fn unify(left: &Typing, right: &Typing) -> Option<Typing> {
        match (left, right) {
            // t * t = t.
            (_, _) if left == right => Some(left.clone()),
            // Any is 1.
            (Typing::Any, _) => Some(right.clone()),
            (_, Typing::Any) => Some(left.clone()),
            // Numbers down-cast (Nat -> Int -> Float).
            (Typing::Nat, Typing::Int) => Some(Typing::Int),
            (Typing::Nat, Typing::Float) => Some(Typing::Float),
            (Typing::Int, Typing::Nat) => Some(Typing::Int),
            (Typing::Int, Typing::Float) => Some(Typing::Float),
            (Typing::Float, Typing::Nat) => Some(Typing::Float),
            (Typing::Float, Typing::Int) => Some(Typing::Float),
            // struct
            (Typing::Array(s), Typing::Array(t)) => {
                Typing::unify(s, t).map(|typ| Typing::Array(Box::new(typ)))
            }
            (Typing::Option(s), Typing::Option(t)) => {
                Typing::unify(s, t).map(|typ| Typing::Option(Box::new(typ)))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod test_typing {
    use crate::parser::typing::*;
    use combine::Parser;

    macro_rules! assert_typing {
        ($code: expr, $expected: expr) => {
            assert_eq!(typing().parse($code).ok().unwrap().0, $expected);
        };
    }

    #[test]
    fn test_parse() {
        assert_typing!("Any", Typing::Any);
        assert_typing!("Nat", Typing::Nat);
        assert_typing!("Int", Typing::Int);
        assert_typing!("Float", Typing::Float);
        assert_typing!("Bool", Typing::Bool);
        assert_typing!("String", Typing::String);
        assert_typing!("Array<String>", Typing::Array(Box::new(Typing::String)));
        assert_typing!(
            "Array<Array<String>>",
            Typing::Array(Box::new(Typing::Array(Box::new(Typing::String))))
        );
        assert_typing!("Option<String>", Typing::Option(Box::new(Typing::String)));
        assert_typing!(
            "Option<Array<String>>",
            Typing::Option(Box::new(Typing::Array(Box::new(Typing::String))))
        );
        assert_typing!(
            "Option<Option<Array<Int>>>",
            Typing::Option(Box::new(Typing::Option(Box::new(Typing::Array(Box::new(
                Typing::Int
            ))))))
        );
        assert_typing!("Hoge_type", Typing::UserTyping("Hoge_type".to_string()));
    }

    macro_rules! assert_unify {
        ($left:expr, $right:expr, $unified:expr) => {
            assert_eq!(Typing::unify(&$left, &$right), $unified);
        };
    }

    #[test]
    fn test_unify() {
        assert_unify!(Typing::Any, Typing::Any, Some(Typing::Any));
        assert_unify!(Typing::Nat, Typing::Any, Some(Typing::Nat));
        assert_unify!(Typing::Nat, Typing::Int, Some(Typing::Int));
        assert_unify!(Typing::Float, Typing::Int, Some(Typing::Float));
        assert_unify!(Typing::Option(Box::new(Typing::Any)), Typing::Int, None);
        assert_unify!(
            Typing::Option(Box::new(Typing::Any)),
            Typing::Option(Box::new(Typing::Int)),
            Some(Typing::Option(Box::new(Typing::Int)))
        );
    }
}
