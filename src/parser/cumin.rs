use crate::parser::expr::*;
use crate::parser::statement::*;
use crate::parser::util::*;

use nom::{combinator::map, multi::many0, sequence::tuple, IResult};

#[derive(Debug, Clone, PartialEq)]
pub struct Cumin(pub Vec<Statement>, pub Expr);

pub fn cumin(input: &str) -> IResult<&str, Cumin> {
    map(
        tuple((commentable_spaces, many0(stmt), expr, commentable_spaces)),
        |(_, ss, e, _)| Cumin(ss, e),
    )(input)
}

#[cfg(test)]
mod test_cumin {
    use crate::parser::cumin::*;
    use crate::parser::typing::*;
    use crate::parser::value::*;
    use Expr::*;
    use Statement::*;
    use Value::*;

    macro_rules! assert_cumin {
        ($code: expr, $expected: expr) => {
            assert_eq!(cumin($code), Ok(("", $expected)));
        };
    }

    #[test]
    fn test() {
        assert_cumin!("-1", Cumin(vec![], Val(Int(-1))));
        assert_cumin!(
            "let x=1; x",
            Cumin(
                vec![Let("x".to_string(), Typing::Any, Val(Nat(1)))],
                Expr::Var("x".to_string())
            )
        );
        assert_cumin!(
            "let x:Int=1; //comment
                // comment
                let y = x + 2;
                // comment
                // comment
                x + y",
            Cumin(
                vec![
                    Let("x".to_string(), Typing::Int, Val(Nat(1))),
                    Let(
                        "y".to_string(),
                        Typing::Any,
                        Add(Box::new(Expr::Var("x".to_string())), Box::new(Val(Nat(2))))
                    ),
                ],
                Add(
                    Box::new(Expr::Var("x".to_string())),
                    Box::new(Expr::Var("y".to_string()))
                )
            )
        );
        assert_cumin!(
            "struct X { x: Int } x + y",
            Cumin(
                vec![Struct(
                    "X".to_string(),
                    vec![("x".to_string(), Typing::Int, None)]
                )],
                Add(
                    Box::new(Expr::Var("x".to_string())),
                    Box::new(Expr::Var("y".to_string()))
                )
            )
        );
        assert_cumin!(
            "struct X { x: Int } let x=1; X(x)",
            Cumin(
                vec![
                    Struct("X".to_string(), vec![("x".to_string(), Typing::Int, None)]),
                    Let("x".to_string(), Typing::Any, Val(Nat(1)))
                ],
                Apply("X".to_string(), vec![Expr::Var("x".to_string())])
            )
        );
    }
}
