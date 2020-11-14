use crate::parser::expr::*;
use crate::parser::util::commentable_spaces;
use crate::parser::value::value;
use combine::error::ParseError;
use combine::parser::char::{char, string};
use combine::parser::combinator::attempt;
use combine::stream::Stream;
use combine::{choice, many, parser};

parser! {
    pub fn arith_expr[Input]()(Input) -> Expr
    where [
        Input: Stream<Token = char>,
        Input::Error: ParseError<char, Input::Range, Input::Position>,
    ]
    {
        (
            (
                term(),
                commentable_spaces(),
            ).map(|t| t.0),
            many::<Vec<(char, Expr)>, _, _>(
                (
                    choice!(char('+'), char('-')),
                    commentable_spaces(),
                    term(),
                    commentable_spaces(),
                ).map(|(op, _, x, _): (char, (), Expr, ())| (op, x))),
        ).map(|(x, xs): (Expr, Vec<(char, Expr)>)| {
            let mut x = x;
            for (op, z) in xs {
                x = match op {
                    '+' => Expr::Add(Box::new(x), Box::new(z)),
                    _ => Expr::Sub(Box::new(x), Box::new(z)),
                };
            }
            x
        })
    }
}

parser! {
    fn term[Input]()(Input) -> Expr
    where [
        Input: Stream<Token = char>,
        Input::Error: ParseError<char, Input::Range, Input::Position>,
    ]
    {
        (
            (
                factor(),
                commentable_spaces(),
            ).map(|t| t.0),
            many::<Vec<(char, Expr)>, _, _>(
                (
                    choice!(
                        attempt(string("**").map(|_| '^')),
                        char('*'),
                        char('/'),
                        char('^')
                    ),
                    commentable_spaces(),
                    factor(),
                    commentable_spaces(),
                ).map(|(op, _, x, _): (char, (), Expr, ())| (op, x))),
        ).map(|(x, xs): (Expr, Vec<(char, Expr)>)| {
            let mut x = x;
            for (op, z) in xs {
                x = match op {
                    '*' => Expr::Mul(Box::new(x), Box::new(z)),
                    '/' => Expr::Div(Box::new(x), Box::new(z)),
                    _ => Expr::Pow(Box::new(x), Box::new(z)),
                };
            }
            x
        })
    }
}

parser! {
    fn factor[Input]()(Input) -> Expr
    where [
        Input: Stream<Token = char>,
        Input::Error: ParseError<char, Input::Range, Input::Position>,
    ]
    {
        let parened =
            (
                char('('),
                commentable_spaces(),
                arith_expr(),
                commentable_spaces(),
                char(')'),
                commentable_spaces(),
            ).map(|(_, _, x, _, _, _): (char, (), Expr, (), char, ())| x);
        choice!(
            attempt(parened),
            value().map(Expr::Val)
        )
    }
}

#[cfg(test)]
mod test_arith {
    use crate::parser::arith::arith_expr;
    use crate::parser::expr::*;
    use crate::parser::value::*;
    use combine::Parser;
    use Expr::*;
    use Value::*;

    #[test]
    fn test() {
        assert_eq!(
            arith_expr().parse("1  /2"),
            Ok((Div(Box::new(Val(Nat(1))), Box::new(Val(Nat(2))),), ""))
        );
        assert_eq!(
            arith_expr().parse("1 + 2 - 3"),
            Ok((
                Sub(
                    Box::new(Add(Box::new(Val(Nat(1))), Box::new(Val(Nat(2))),)),
                    Box::new(Val(Nat(3)))
                ),
                ""
            ))
        );
        assert_eq!(
            arith_expr().parse("1 * 2 * 3 / 4"),
            Ok((
                Div(
                    Box::new(Mul(
                        Box::new(Mul(Box::new(Val(Nat(1))), Box::new(Val(Nat(2))),)),
                        Box::new(Val(Nat(3)))
                    )),
                    Box::new(Val(Nat(4)))
                ),
                ""
            ))
        );
        assert_eq!(
            arith_expr().parse("1 + 2 * 3"),
            Ok((
                Add(
                    Box::new(Val(Nat(1))),
                    Box::new(Mul(Box::new(Val(Nat(2))), Box::new(Val(Nat(3))),))
                ),
                ""
            ))
        );
        assert_eq!(
            arith_expr().parse("(1 + 2) * ((3) / 4 - 5)"),
            Ok((
                Mul(
                    Box::new(Add(Box::new(Val(Nat(1))), Box::new(Val(Nat(2))),)),
                    Box::new(Sub(
                        Box::new(Div(Box::new(Val(Nat(3))), Box::new(Val(Nat(4))),)),
                        Box::new(Val(Nat(5)))
                    ))
                ),
                ""
            ))
        );
    }
}
