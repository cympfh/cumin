use crate::parser::expr::*;
use crate::parser::statement::*;
use crate::parser::util::*;
use combine::error::ParseError;
use combine::stream::Stream;
use combine::{many, parser};

#[derive(Debug, Clone, PartialEq)]
pub struct Config(pub Vec<Statement>, pub Expr);

parser! {
    pub fn config[Input]()(Input) -> Config
    where [
        Input: Stream<Token = char>,
        Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    ]
    {
        (
            commentable_spaces(),
            many(stmt()),
            expr(),
            commentable_spaces(),
        )
            .map(|t: ((), Vec<Statement>, Expr, ())| Config(t.1, t.2))
    }
}

#[cfg(test)]
mod test_config {
    use crate::parser::config::*;
    use crate::parser::typing::*;
    use crate::parser::value::*;
    use combine::Parser;
    use Expr::*;
    use Statement::*;
    use Value::*;

    #[test]
    fn test() {
        assert_eq!(config().parse("-1"), Ok((Config(vec![], Val(Int(-1))), "")));
        assert_eq!(
            config().parse("let x=1; x"),
            Ok((
                Config(
                    vec![Let("x".to_string(), Typing::Any, Val(Nat(1)))],
                    Val(Var("x".to_string()))
                ),
                ""
            ))
        );
        assert_eq!(
            config().parse("let x:Int=1; let y = x + 2; x + y"),
            Ok((
                Config(
                    vec![
                        Let("x".to_string(), Typing::Int, Val(Nat(1))),
                        Let(
                            "y".to_string(),
                            Typing::Any,
                            Add(Box::new(Val(Var("x".to_string()))), Box::new(Val(Nat(2))))
                        ),
                    ],
                    Add(
                        Box::new(Val(Var("x".to_string()))),
                        Box::new(Val(Var("y".to_string())))
                    )
                ),
                ""
            ))
        );
        assert_eq!(
            config().parse("struct X { x: Int } x + y"),
            Ok((
                Config(
                    vec![Struct(
                        "X".to_string(),
                        vec![("x".to_string(), Typing::Int, None)]
                    )],
                    Add(
                        Box::new(Val(Var("x".to_string()))),
                        Box::new(Val(Var("y".to_string())))
                    )
                ),
                ""
            ))
        );
        assert_eq!(
            config().parse("struct X { x: Int } let x=1; X(x)"),
            Ok((
                Config(
                    vec![
                        Struct("X".to_string(), vec![("x".to_string(), Typing::Int, None)]),
                        Let("x".to_string(), Typing::Any, Val(Nat(1)))
                    ],
                    Apply("X".to_string(), vec![Val(Var("x".to_string()))])
                ),
                ""
            ))
        );
    }
}
