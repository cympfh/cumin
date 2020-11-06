use crate::parser::value::*;
use combine::parser::char::{char, spaces};
use combine::parser::combinator::attempt;
use combine::stream::Stream;
use combine::{choice, parser, Parser};

#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    Val(Value),
    Add(Box<Expr>, Box<Expr>),
}

parser! {
    pub fn expr[Input]()(Input) -> Expr where [Input: Stream<Token=char>] {
        choice!(
            attempt((value(), spaces(), char('+'), spaces(), expr()).map(|t|
                Expr::Add(Box::new(Expr::Val(t.0)), Box::new(t.4)))),
            value().map(|x: Value| Expr::Val(x))
        )
    }
}

#[cfg(test)]
mod test_expr {
    use crate::parser::expr::*;
    use Expr::*;
    use Value::*;

    #[test]
    fn test() {
        assert_eq!(expr().parse("2"), Ok((Val(Nat(2)), "")));
        assert_eq!(expr().parse("-1"), Ok((Val(Int(-1)), "")));
        assert_eq!(expr().parse("x"), Ok((Val(Var("x".to_string())), "")));
        assert_eq!(
            expr().parse("0 + 1"),
            Ok((Add(Box::new(Val(Nat(0))), Box::new(Val(Nat(1))),), ""))
        );
        assert_eq!(
            expr().parse("0 + x"),
            Ok((
                Add(Box::new(Val(Nat(0))), Box::new(Val(Var("x".to_string()))),),
                ""
            ))
        );
        assert_eq!(
            expr().parse("x + 2"),
            Ok((
                Add(Box::new(Val(Var("x".to_string()))), Box::new(Val(Nat(2))),),
                ""
            ))
        );
        assert_eq!(
            expr().parse("x + y + z"),
            Ok((
                Add(
                    Box::new(Val(Var("x".to_string()))),
                    Box::new(Add(
                        Box::new(Val(Var("y".to_string()))),
                        Box::new(Val(Var("z".to_string()))),
                    ))
                ),
                ""
            ))
        );
    }
}
