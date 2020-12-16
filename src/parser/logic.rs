use crate::parser::expr::*;
use crate::parser::util::{commentable_spaces, spaces};
use crate::parser::value::value;

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    multi::fold_many0,
    sequence::{preceded, terminated, tuple},
    IResult,
};

pub fn logic_expr(input: &str) -> IResult<&str, Expr> {
    let compare = map(
        tuple((
            ab_expr,
            commentable_spaces,
            alt((
                tag("=="),
                tag("!="),
                tag("<="),
                tag(">="),
                tag("<"),
                tag(">"),
            )),
            commentable_spaces,
            ab_expr,
            commentable_spaces,
        )),
        |(x, _, op, _, y, _)| match op {
            "==" => Expr::Equal(Box::new(x), Box::new(y)),
            "!=" => Expr::Not(Box::new(Expr::Equal(Box::new(x), Box::new(y)))),
            "<=" => Expr::Not(Box::new(Expr::Less(Box::new(y), Box::new(x)))),
            ">=" => Expr::Not(Box::new(Expr::Less(Box::new(x), Box::new(y)))),
            "<" => Expr::Less(Box::new(x), Box::new(y)),
            ">" => Expr::Less(Box::new(y), Box::new(x)),
            _ => panic!(),
        },
    );
    alt((compare, ab_expr))(input)
}

fn ab_expr(input: &str) -> IResult<&str, Expr> {
    let (input, x) = term(input)?;
    let (input, _) = commentable_spaces(input)?;
    fold_many0(
        tuple((
            alt((tag("and"), tag("or"), tag("xor"), tag("+"), tag("-"))),
            commentable_spaces,
            term,
        )),
        x,
        |acc, (op, _, val)| match op {
            "and" => Expr::And(Box::new(acc), Box::new(val)),
            "or" => Expr::Or(Box::new(acc), Box::new(val)),
            "xor" => Expr::Xor(Box::new(acc), Box::new(val)),
            "+" => Expr::Add(Box::new(acc), Box::new(val)),
            "-" => Expr::Sub(Box::new(acc), Box::new(val)),
            _ => panic!(),
        },
    )(input)
}

fn term(input: &str) -> IResult<&str, Expr> {
    let (input, x) = factor(input)?;
    let (input, _) = commentable_spaces(input)?;
    fold_many0(
        tuple((
            alt((tag("**"), tag("*"), tag("/"))),
            commentable_spaces,
            factor,
        )),
        x,
        |acc, (op, _, val)| match op {
            "**" => Expr::Pow(Box::new(acc), Box::new(val)),
            "*" => Expr::Mul(Box::new(acc), Box::new(val)),
            "/" => Expr::Div(Box::new(acc), Box::new(val)),
            _ => panic!(),
        },
    )(input)
}

fn factor(input: &str) -> IResult<&str, Expr> {
    let parened = map(
        tuple((
            tag("("),
            commentable_spaces,
            expr,
            commentable_spaces,
            tag(")"),
        )),
        |(_, _, e, _, _)| e,
    );
    let minused = map(preceded(tag("-"), ab_expr), |e| Expr::Minus(Box::new(e)));
    let notted = map(preceded(tag("not"), preceded(spaces, term)), |e| {
        Expr::Not(Box::new(e))
    });
    let avalue = map(value, Expr::Val);
    terminated(alt((parened, notted, avalue, minused)), commentable_spaces)(input)
}

#[cfg(test)]
mod test_logic {
    use crate::parser::logic::*;
    use crate::parser::value::*;
    use Expr::*;
    use Value::*;

    macro_rules! assert_logic {
        ($code: expr, $expected: expr) => {
            assert_eq!(logic_expr($code), Ok(("", $expected)))
        };
    }

    #[test]
    fn test_arith() {
        assert_logic!("1 // one", Val(Nat(1)));
        assert_logic!("1+-1", Add(Box::new(Val(Nat(1))), Box::new(Val(Int(-1)))));
        assert_logic!("1  /2", Div(Box::new(Val(Nat(1))), Box::new(Val(Nat(2)))));
        assert_logic!(
            "1 + 2 - 3",
            Sub(
                Box::new(Add(Box::new(Val(Nat(1))), Box::new(Val(Nat(2))),)),
                Box::new(Val(Nat(3)))
            )
        );
        assert_logic!(
            "1 * 2 * 3 / 4",
            Div(
                Box::new(Mul(
                    Box::new(Mul(Box::new(Val(Nat(1))), Box::new(Val(Nat(2))),)),
                    Box::new(Val(Nat(3)))
                )),
                Box::new(Val(Nat(4)))
            )
        );
        assert_logic!(
            "1 + 2 * 3",
            Add(
                Box::new(Val(Nat(1))),
                Box::new(Mul(Box::new(Val(Nat(2))), Box::new(Val(Nat(3))),))
            )
        );
        assert_logic!(
            "(1 + 2) * ((3) / 4 - 5)",
            Mul(
                Box::new(Add(Box::new(Val(Nat(1))), Box::new(Val(Nat(2))),)),
                Box::new(Sub(
                    Box::new(Div(Box::new(Val(Nat(3))), Box::new(Val(Nat(4))),)),
                    Box::new(Val(Nat(5)))
                ))
            )
        );
        assert_logic!("-(-2)", Minus(Box::new(Val(Int(-2)))));
        assert_logic!("-x", Minus(Box::new(Val(Var("x".to_string())))));
    }

    #[test]
    fn test_bool() {
        assert_logic!("true", Val(Bool(true)));
        assert_logic!("not x", Not(Box::new(Val(Var("x".to_string())))));
        assert_logic!(
            "not true or true",
            Or(
                Box::new(Not(Box::new(Val(Bool(true))))),
                Box::new(Val(Bool(true)))
            )
        );
        assert_logic!(
            "true or not true",
            Or(
                Box::new(Val(Bool(true))),
                Box::new(Not(Box::new(Val(Bool(true)))))
            )
        );
        assert_logic!(
            "x and y",
            And(
                Box::new(Val(Var("x".to_string()))),
                Box::new(Val(Var("y".to_string())))
            )
        );
        assert_logic!(
            "true and false or true xor false",
            Xor(
                Box::new(Or(
                    Box::new(And(Box::new(Val(Bool(true))), Box::new(Val(Bool(false))))),
                    Box::new(Val(Bool(true)))
                )),
                Box::new(Val(Bool(false)))
            )
        );
        assert_logic!(
            "true and (false or not true)",
            And(
                Box::new(Val(Bool(true))),
                Box::new(Or(
                    Box::new(Val(Bool(false))),
                    Box::new(Not(Box::new(Val(Bool(true)))))
                ))
            )
        );
    }
    #[test]
    fn test_compare() {
        assert_logic!(
            "1 == 2",
            Equal(Box::new(Val(Nat(1))), Box::new(Val(Nat(2))))
        );
        assert_logic!(
            "1 <= 2",
            Not(Box::new(Less(Box::new(Val(Nat(2))), Box::new(Val(Nat(1))))))
        );
        assert_logic!(
            "1 + 1 == 2 - 0",
            Equal(
                Box::new(Add(Box::new(Val(Nat(1))), Box::new(Val(Nat(1))))),
                Box::new(Sub(Box::new(Val(Nat(2))), Box::new(Val(Nat(0)))))
            )
        );
        assert_logic!(
            "(1 <= 2) == false",
            Equal(
                Box::new(Not(Box::new(Less(
                    Box::new(Val(Nat(2))),
                    Box::new(Val(Nat(1)))
                )))),
                Box::new(Val(Bool(false)))
            )
        );
    }
}
