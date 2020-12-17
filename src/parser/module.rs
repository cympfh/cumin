use crate::parser::statement::*;
use crate::parser::util::*;

use nom::{combinator::map, multi::many0, sequence::tuple, IResult};

pub fn load_module(input: &str) -> IResult<&str, Vec<Statement>> {
    map(
        tuple((commentable_spaces, many0(stmt), commentable_spaces)),
        |(_, ss, _)| ss,
    )(input)
}

#[cfg(test)]
mod test_module {
    use crate::parser::{
        expr::Expr::*, module::load_module, statement::Statement::*, typing::Typing,
        value::Value::*,
    };

    macro_rules! assert_module {
        ($code: expr, $expected: expr) => {
            assert_eq!(load_module($code), Ok(("", $expected)));
        };
    }

    #[test]
    fn test() {
        assert_module!("", vec![]);
        assert_module!("struct A{}", vec![Struct("A".to_string(), vec![])]);
        assert_module!(
            "struct A{x: Int}",
            vec![Struct(
                "A".to_string(),
                vec![("x".to_string(), Typing::Int, None)]
            )]
        );
        assert_module!(
            "struct A{x: Int}
            struct B{}",
            vec![
                Struct("A".to_string(), vec![("x".to_string(), Typing::Int, None)]),
                Struct("B".to_string(), vec![]),
            ]
        );
        assert_module!(
            "let x=1;",
            vec![Let("x".to_string(), Typing::Any, Val(Nat(1)))]
        );
        assert_module!(
            "let x: Int=1;
            use \"hoge.cumin\";
            ",
            vec![
                Let("x".to_string(), Typing::Int, Val(Nat(1))),
                Import("hoge.cumin".to_string()),
            ]
        );
    }
}
