use crate::parser::expr::*;
use crate::parser::statement::*;
use crate::parser::util::*;
use combine::stream::Stream;
use combine::{many, parser};

#[derive(Debug, Clone, PartialEq)]
pub struct Config(pub Vec<Statement>, pub Expr);

parser! {
    pub fn config[Input]()(Input) -> Config where [Input: Stream<Token=char>] {
        (commentable_spaces(), many(stmt()), expr(), commentable_spaces()).map(|t| Config(t.1, t.2))
    }
}

#[cfg(test)]
mod test_config {
    use crate::parser::config::*;
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
                    vec![Let("x".to_string(), "Any".to_string(), Val(Nat(1)))],
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
                        Let("x".to_string(), "Int".to_string(), Val(Nat(1))),
                        Let(
                            "y".to_string(),
                            "Any".to_string(),
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
                        vec![("x".to_string(), "Int".to_string())]
                    )],
                    Add(
                        Box::new(Val(Var("x".to_string()))),
                        Box::new(Val(Var("y".to_string())))
                    )
                ),
                ""
            ))
        );
        // assert_eq!(
        //     config().parse("struct X { x: Int } let x=1; X(x)"),
        //     Ok((
        //         vec![
        //             Struct("X".to_string(), vec![("x".to_string(), "Int".to_string())]),
        //             Let("x".to_string(), Val(Nat(1)))
        //         ],
        //         ""
        //     ))
        // );
    }
}
