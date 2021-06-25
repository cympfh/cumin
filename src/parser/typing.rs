use crate::parser::util::{commentable_spaces, identifier, spaces};
use nom::combinator;
use nom::{
    branch::alt, bytes::complete::tag, combinator::map, multi::separated_list1, sequence::tuple,
    IResult,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Typing {
    Any,
    Nat,
    Int,
    Float,
    Bool,
    String,
    Array(Box<Typing>),
    Tuple(Vec<Typing>),
    Option(Box<Typing>),
    UserTyping(String),
}

pub fn typing(input: &str) -> IResult<&str, Typing> {
    alt((
        combinator::value(Typing::Any, tag("Any")),
        combinator::value(Typing::Any, tag("_")),
        combinator::value(Typing::Nat, tag("Nat")),
        combinator::value(Typing::Int, tag("Int")),
        combinator::value(Typing::Float, tag("Float")),
        combinator::value(Typing::Bool, tag("Bool")),
        combinator::value(Typing::String, tag("String")),
        map(
            tuple((
                tag("Array"),
                spaces,
                tag("<"),
                spaces,
                typing,
                spaces,
                tag(">"),
                spaces,
            )),
            |item| Typing::Array(Box::new(item.4)),
        ),
        map(
            tuple((
                tag("("),
                separated_list1(tuple((tag(","), commentable_spaces)), typing),
                tag(")"),
            )),
            |item| Typing::Tuple(item.1),
        ),
        map(
            tuple((
                tag("Option"),
                spaces,
                tag("<"),
                spaces,
                typing,
                spaces,
                tag(">"),
                spaces,
            )),
            |item| Typing::Option(Box::new(item.4)),
        ),
        map(identifier, |s| Typing::UserTyping(s)),
    ))(input)
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
            (Typing::Tuple(xs), Typing::Tuple(ys)) => {
                if xs.len() == ys.len() {
                    let types = xs
                        .iter()
                        .zip(ys.iter())
                        .map(|(x, y)| Typing::unify(x, y))
                        .collect::<Option<Vec<Typing>>>()?;
                    Some(Typing::Tuple(types))
                } else {
                    None
                }
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

    macro_rules! assert_typing {
        ($code: expr, $expected: expr) => {
            assert_eq!(typing($code), Ok(("", $expected)))
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
        assert_typing!("Array<_>", Typing::Array(Box::new(Typing::Any)));
        assert_typing!("Array<String>", Typing::Array(Box::new(Typing::String)));
        assert_typing!(
            "Array<Array<String>>",
            Typing::Array(Box::new(Typing::Array(Box::new(Typing::String))))
        );
        assert_typing!("(Int, Nat)", Typing::Tuple(vec![Typing::Int, Typing::Nat]));
        assert_typing!(
            "(Int, (Option<Nat>, S))",
            Typing::Tuple(vec![
                Typing::Int,
                Typing::Tuple(vec![
                    Typing::Option(Box::new(Typing::Nat)),
                    Typing::UserTyping("S".to_string()),
                ])
            ])
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
        assert_unify!(
            Typing::Tuple(vec![Typing::Any, Typing::Nat]),
            Typing::Tuple(vec![Typing::Nat, Typing::Int]),
            Some(Typing::Tuple(vec![Typing::Nat, Typing::Int]))
        );
        assert_unify!(Typing::Option(Box::new(Typing::Any)), Typing::Int, None);
        assert_unify!(
            Typing::Option(Box::new(Typing::Any)),
            Typing::Option(Box::new(Typing::Int)),
            Some(Typing::Option(Box::new(Typing::Int)))
        );
    }
}
