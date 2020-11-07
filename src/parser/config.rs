use crate::parser::statement::*;
use combine::stream::Stream;
use combine::{many, parser, Parser};

parser! {
    pub fn config[Input]()(Input) -> Vec<Statement> where [Input: Stream<Token=char>] {
        many(stmt())
    }
}

#[cfg(test)]
mod test_config {
    use crate::parser::config::*;
    use crate::parser::expr::*;
    use crate::parser::statement::*;
    use crate::parser::value::*;
    use Expr::*;
    use Statement::*;
    use Value::*;

    #[test]
    fn test() {
        assert_eq!(
            config().parse("let x=1;"),
            Ok((vec![Let("x".to_string(), Val(Nat(1)))], ""))
        );
        assert_eq!(
            config().parse("let x=1; let y = x + 2;"),
            Ok((
                vec![
                    Let("x".to_string(), Val(Nat(1))),
                    Let(
                        "y".to_string(),
                        Add(Box::new(Val(Var("x".to_string()))), Box::new(Val(Nat(2))))
                    ),
                ],
                ""
            ))
        );
        assert_eq!(
            config().parse("struct X { x: Int } let x=1;"),
            Ok((
                vec![
                    Struct("X".to_string(), vec![("x".to_string(), "Int".to_string())]),
                    Let("x".to_string(), Val(Nat(1)))
                ],
                ""
            ))
        );
    }
}
