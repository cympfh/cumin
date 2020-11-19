use crate::parser::expr::*;
use crate::parser::util::commentable_spaces;
use crate::parser::value::value;
use combine::error::ParseError;
use combine::parser::char::{char, space, spaces, string};
use combine::parser::combinator::attempt;
use combine::stream::Stream;
use combine::{chainl1, choice, parser};

parser! {
    pub fn logic_expr[Input]()(Input) -> Expr
    where [
        Input: Stream<Token = char>,
        Input::Error: ParseError<char, Input::Range, Input::Position>,
    ]
    {
        let compare =
            (
                bool_expr(),
                commentable_spaces(),
                choice!(
                    attempt(string("==")),
                    attempt(string("!=")),
                    attempt(string("<=")),
                    attempt(string(">=")),
                    attempt(string("<")),
                    attempt(string(">"))),
                commentable_spaces(),
                bool_expr(),
                commentable_spaces(),
            ).map(|(x, _, op, _, y, _): (Expr, (), &str, (), Expr, ())| {
                match op {
                    "==" => Expr::Equal(Box::new(x), Box::new(y)),
                    "!=" => Expr::Not(Box::new(Expr::Equal(Box::new(x), Box::new(y)))),
                    "<=" => Expr::Not(Box::new(Expr::Less(Box::new(y), Box::new(x)))),
                    ">=" => Expr::Not(Box::new(Expr::Less(Box::new(x), Box::new(y)))),
                    "<" => Expr::Less(Box::new(x), Box::new(y)),
                    ">" => Expr::Less(Box::new(y), Box::new(x)),
                    _ => panic!(),
                }
            });
        choice!(attempt(compare), bool_expr())
    }
}

parser! {
    pub fn bool_expr[Input]()(Input) -> Expr
    where [
        Input: Stream<Token = char>,
        Input::Error: ParseError<char, Input::Range, Input::Position>,
    ]
    {
        let p = arith_expr().skip(commentable_spaces());
        let op_token = choice!(
            attempt(string("and")),
            attempt(string("or")),
            attempt(string("xor")));
        let op = (
            op_token,
            space(),
            commentable_spaces()
        ).map(|(token, _, _): (&str, char, ())| move |left: Expr, right: Expr|
            match &token {
                &"and" => Expr::And(Box::new(left), Box::new(right)),
                &"or" => Expr::Or(Box::new(left), Box::new(right)),
                _ => Expr::Xor(Box::new(left), Box::new(right)),
            });
        chainl1(p, op)
    }
}

parser! {
    fn arith_expr[Input]()(Input) -> Expr
    where [
        Input: Stream<Token = char>,
        Input::Error: ParseError<char, Input::Range, Input::Position>,
    ]
    {
        let p = (term(), commentable_spaces()).map(|t| t.0);
        let op_token = choice!(char('+'), char('-'));
        let op = op_token.skip(commentable_spaces()).map(|token: char| move |left: Expr, right: Expr|
            match token {
                '+' => Expr::Add(Box::new(left), Box::new(right)),
                _ => Expr::Sub(Box::new(left), Box::new(right)),
            });
        chainl1(p, op)
    }
}

parser! {
    fn term[Input]()(Input) -> Expr
    where [
        Input: Stream<Token = char>,
        Input::Error: ParseError<char, Input::Range, Input::Position>,
    ]
    {
        let p = (factor(), commentable_spaces()).map(|t| t.0);
        let op_token = choice!(
            attempt(string("**").map(|_| '^')),
            char('*'),
            char('/'),
            char('^'));
        let op = op_token.skip(commentable_spaces()).map(|token: char| move |left: Expr, right: Expr|
            match token {
                '*' => Expr::Mul(Box::new(left), Box::new(right)),
                '/' => Expr::Div(Box::new(left), Box::new(right)),
                _ => Expr::Pow(Box::new(left), Box::new(right)),
            });
        chainl1(p, op)
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
                expr(),
                commentable_spaces(),
                char(')'),
                commentable_spaces(),
            ).map(|(_, _, x, _, _, _): (char, (), Expr, (), char, ())| x);
        let minused =
            (
                char('-'),
                arith_expr(),
            ).map(|(_, e): (char, Expr)| Expr::Minus(Box::new(e)));
        let notted =
            (
                string("not"),
                spaces(),
                logic_expr(),
            ).map(|(_, _, e): (&str, (), Expr)| Expr::Not(Box::new(e)));
        choice!(
            attempt(parened),
            attempt(notted),
            attempt(value().map(Expr::Val)),
            attempt(minused)
        )
    }
}

#[cfg(test)]
mod test_logic {
    use crate::parser::logic::*;
    use crate::parser::value::*;
    use combine::Parser;
    use Expr::*;
    use Value::*;

    #[test]
    fn test_arith() {
        assert_eq!(
            logic_expr().parse("1  /2"),
            Ok((Div(Box::new(Val(Nat(1))), Box::new(Val(Nat(2))),), ""))
        );
        assert_eq!(
            logic_expr().parse("1 + 2 - 3"),
            Ok((
                Sub(
                    Box::new(Add(Box::new(Val(Nat(1))), Box::new(Val(Nat(2))),)),
                    Box::new(Val(Nat(3)))
                ),
                ""
            ))
        );
        assert_eq!(
            logic_expr().parse("1 * 2 * 3 / 4"),
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
            logic_expr().parse("1 + 2 * 3"),
            Ok((
                Add(
                    Box::new(Val(Nat(1))),
                    Box::new(Mul(Box::new(Val(Nat(2))), Box::new(Val(Nat(3))),))
                ),
                ""
            ))
        );
        assert_eq!(
            logic_expr().parse("(1 + 2) * ((3) / 4 - 5)"),
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
        assert_eq!(
            logic_expr().parse("-(-2)"),
            Ok((Minus(Box::new(Val(Int(-2)))), ""))
        );
        assert_eq!(
            logic_expr().parse("-x"),
            Ok((Minus(Box::new(Val(Var("x".to_string())))), ""))
        );
    }

    #[test]
    fn test_bool() {
        assert_eq!(logic_expr().parse("true"), Ok((Val(Bool(true)), "")));
        assert_eq!(
            logic_expr().parse("not x"),
            Ok((Not(Box::new(Val(Var("x".to_string())))), ""))
        );
        assert_eq!(
            logic_expr().parse("x and y"),
            Ok((
                And(
                    Box::new(Val(Var("x".to_string()))),
                    Box::new(Val(Var("y".to_string())))
                ),
                ""
            ))
        );
        assert_eq!(
            logic_expr().parse("true and false or true xor false"),
            Ok((
                Xor(
                    Box::new(Or(
                        Box::new(And(Box::new(Val(Bool(true))), Box::new(Val(Bool(false))))),
                        Box::new(Val(Bool(true)))
                    )),
                    Box::new(Val(Bool(false)))
                ),
                ""
            ))
        );
        assert_eq!(
            logic_expr().parse("true and (false or not true)"),
            Ok((
                And(
                    Box::new(Val(Bool(true))),
                    Box::new(Or(
                        Box::new(Val(Bool(false))),
                        Box::new(Not(Box::new(Val(Bool(true)))))
                    ))
                ),
                ""
            ))
        );
    }
    #[test]
    fn test_compare() {
        assert_eq!(
            logic_expr().parse("1 == 2"),
            Ok((Equal(Box::new(Val(Nat(1))), Box::new(Val(Nat(2)))), ""))
        );
        assert_eq!(
            logic_expr().parse("1 <= 2"),
            Ok((
                Not(Box::new(Less(Box::new(Val(Nat(2))), Box::new(Val(Nat(1)))))),
                ""
            ))
        );
        assert_eq!(
            logic_expr().parse("1 + 1 == 2 - 0"),
            Ok((
                Equal(
                    Box::new(Add(Box::new(Val(Nat(1))), Box::new(Val(Nat(1))))),
                    Box::new(Sub(Box::new(Val(Nat(2))), Box::new(Val(Nat(0)))))
                ),
                ""
            ))
        );
        assert_eq!(
            logic_expr().parse("(1 <= 2) == false"),
            Ok((
                Equal(
                    Box::new(Not(Box::new(Less(
                        Box::new(Val(Nat(2))),
                        Box::new(Val(Nat(1)))
                    )))),
                    Box::new(Val(Bool(false)))
                ),
                ""
            ))
        );
    }
}
