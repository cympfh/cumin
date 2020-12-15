use crate::parser::config::*;
use crate::parser::logic::logic_expr;
use crate::parser::typing::*;
use crate::parser::util::*;
use crate::parser::value::*;
use combine::error::ParseError;
use combine::parser::char::{char, spaces, string};
use combine::parser::combinator::attempt;
use combine::stream::Stream;
use combine::{choice, many, parser, sep_by, sep_by1};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Val(Value),
    Apply(String, Vec<Expr>),
    FieledApply(String, Vec<(String, Expr)>),
    AnonymousStruct(Vec<(String, Expr)>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Minus(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Xor(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Equal(Box<Expr>, Box<Expr>),
    Less(Box<Expr>, Box<Expr>),
    Arrayed(Vec<Expr>),
    Blocked(Box<Config>),
    AsCast(Box<Expr>, Typing),
}

parser! {
    pub fn expr[Input]()(Input) -> Expr
    where [
        Input: Stream<Token = char>,
        Input::Error: ParseError<char, Input::Range, Input::Position>,
    ]
    {
        // <identifier>.<identifier>(<expr>, <expr>)
        let apply_expr = {
            let composed_func = sep_by1::<Vec<String>, _, _, _>(
                identifier(),
                char('.'));
            let inner_sep = sep_by::<Vec<Expr>,_, _, _>(
                (
                    expr(),
                    commentable_spaces()
                ).map(|t: (Expr, ())| t.0),
                char(',').with(commentable_spaces()));
            let inner_trailing = many::<Vec<Expr>, _, _>(
                (
                    expr(),
                    commentable_spaces(),
                    char(','),
                    commentable_spaces()
                ).map(|t: (Expr, (), char, ())| t.0));
            (
                composed_func,
                commentable_spaces(),
                char('('),
                commentable_spaces(),
                choice!(attempt(inner_trailing), inner_sep),
                char(')'),
                commentable_spaces(),
            ).map(|(fs, _, _, _, args, _, _): (Vec<String>, (), char, (), Vec<Expr>, char, _)| {
                let n = fs.len();
                assert!(n > 0);
                let mut e = Expr::Apply(fs[n-1].to_string(), args);
                for i in (0..n-1).rev() {
                    e = Expr::Apply(fs[i].to_string(), vec![e]);
                }
                e
            })
        };

        // F { id = expr [,] }
        let fielded_apply_expr = {
            let composed_func = sep_by1::<Vec<String>, _, _, _>(
                identifier(),
                char('.'));
            let inner_sep = sep_by::<Vec<(String, Expr)>, _, _, _>(
                (
                    identifier(),
                    commentable_spaces(),
                    char('='),
                    commentable_spaces(),
                    expr(),
                    commentable_spaces(),
                ).map(|t| (t.0, t.4)),
                char(',').with(commentable_spaces()));
            let inner_trailing = many::<Vec<(String, Expr)>, _, _>(
                (
                    identifier(),
                    commentable_spaces(),
                    char('='),
                    commentable_spaces(),
                    expr(),
                    commentable_spaces(),
                    char(','),
                    commentable_spaces(),
                ).map(|t| (t.0, t.4)));
            (
                composed_func,
                commentable_spaces(),
                char('{'),
                commentable_spaces(),
                choice!(attempt(inner_trailing), inner_sep),
                char('}'),
                commentable_spaces(),
            ).map(|(fs, _, _, _, args, _, _): (Vec<String>, (), char, (), Vec<(String, Expr)>, char, ())| {
                let n = fs.len();
                assert!(n > 0);
                let mut e = Expr::FieledApply(fs[n-1].to_string(), args);
                for i in (0..n-1).rev() {
                    e = Expr::Apply(fs[i].to_string(), vec![e]);
                }
                e
            })
        };

        // {{ id = expr [,] }}
        let dict_expr = {
            let inner_sep = sep_by::<Vec<(String, Expr)>, _, _, _>(
                (
                    identifier(),
                    commentable_spaces(),
                    char('='),
                    commentable_spaces(),
                    expr(),
                    commentable_spaces(),
                ).map(|t: (String, (), char, (), Expr, ())| (t.0, t.4)),
                char(',').with(commentable_spaces())
            );
            let inner_trailing = many::<Vec<(String, Expr)>, _, _>(
                (
                    identifier(),
                    commentable_spaces(),
                    char('='),
                    commentable_spaces(),
                    expr(),
                    commentable_spaces(),
                    char(','),
                    commentable_spaces(),
                ).map(|t| (t.0, t.4))
            );
            (
                string("{{"),
                commentable_spaces(),
                choice!(attempt(inner_trailing), inner_sep),
                string("}}"),
                commentable_spaces(),
            ).map(|t: (&str, (), Vec<(String, Expr)>, &str, ())| Expr::AnonymousStruct(t.2))
        };

        // { statement...; expr }
        let blocked_expr = (
            char('{'),
            commentable_spaces(),
            config(),
            char('}'),
            commentable_spaces(),
        ).map(|(_, _, inner, _, _): (char, (), Config, char, ())| Expr::Blocked(Box::new(inner)));

        // as cast
        let as_expr = {
            let plain_value = value().map(Expr::Val);
            let parenthesis_expr = (
                char('('),
                commentable_spaces(),
                expr(),
                char(')'),
            ).map(|t: (char, (), Expr, char)| t.2);
            (
                choice!(parenthesis_expr, plain_value),
                spaces(),
                string("as"),
                spaces(),
                typing(),
                commentable_spaces(),
            ).map(|t: (Expr, (), &str, (), Typing, ())| Expr::AsCast(Box::new(t.0), t.4))
        };

        let value_expr = value().map(|x: Value| Expr::Val(x));

        // [ Expr [,] ]
        let arrayed_expr = {
            let inner_sep =
                sep_by::<Vec<Expr>, _, _, _>(
                    (
                        expr(),
                        commentable_spaces()
                    ).map(|t: (Expr, ())| t.0),
                    char(',').with(commentable_spaces())
                );
            let inner_trailing = many::<Vec<Expr>, _, _>(
                    (
                        expr(),
                        commentable_spaces(),
                        char(','),
                        commentable_spaces()
                    ).map(|t: (Expr, (), char, ())| t.0)
                );
            (
                char('['),
                commentable_spaces(),
                choice!(attempt(inner_trailing), inner_sep),
                char(']'),
                commentable_spaces(),
            ).map(|t: (char, (), Vec<Expr>, char, ())| Expr::Arrayed(t.2))
        };

        choice!(
            attempt(apply_expr),
            attempt(dict_expr),
            attempt(blocked_expr),
            attempt(arrayed_expr),
            attempt(fielded_apply_expr),
            attempt(as_expr),
            attempt(logic_expr()),
            value_expr
        )
    }
}

#[cfg(test)]
mod test_expr {
    use crate::parser::expr::*;
    use crate::parser::statement::*;
    use combine::Parser;
    use Expr::*;
    use Value::*;

    macro_rules! assert_expr {
        ($code: expr, $expected: expr) => {
            assert_eq!(expr().parse($code).ok().unwrap().0, $expected);
        };
    }

    #[test]
    fn test() {
        assert_expr!("2", Val(Nat(2)));
        assert_expr!("-1", Val(Int(-1)));
        assert_expr!("x", Val(Var("x".to_string())));
        assert_expr!("0 + 1", Add(Box::new(Val(Nat(0))), Box::new(Val(Nat(1)))));
        assert_expr!(
            "0 + x",
            Add(Box::new(Val(Nat(0))), Box::new(Val(Var("x".to_string()))))
        );
        assert_expr!(
            "x + 2",
            Add(Box::new(Val(Var("x".to_string()))), Box::new(Val(Nat(2))))
        );
        assert_expr!(
            "x + y + z",
            Add(
                Box::new(Add(
                    Box::new(Val(Var("x".to_string()))),
                    Box::new(Val(Var("y".to_string()))),
                )),
                Box::new(Val(Var("z".to_string()))),
            )
        );
        assert_expr!(
            "x - y",
            Sub(
                Box::new(Val(Var("x".to_string()))),
                Box::new(Val(Var("y".to_string()))),
            )
        );
        assert_expr!(
            "( 1 - 2 ) ",
            Sub(Box::new(Val(Nat(1))), Box::new(Val(Nat(2))))
        );
        assert_expr!(
            "(x * y) / z",
            Div(
                Box::new(Mul(
                    Box::new(Val(Var("x".to_string()))),
                    Box::new(Val(Var("y".to_string()))),
                )),
                Box::new(Val(Var("z".to_string())))
            )
        );
    }

    #[test]
    fn test_bool_expression() {
        assert_expr!("true", Val(Bool(true)));
        assert_expr!("false", Val(Bool(false)));
        assert_expr!("not false", Not(Box::new(Val(Bool(false)))));
        assert_expr!(
            "true or false",
            Or(Box::new(Val(Bool(true))), Box::new(Val(Bool(false))))
        );
        assert_expr!(
            "(a or not b) xor (not c and d)",
            Xor(
                Box::new(Or(
                    Box::new(Val(Var("a".to_string()))),
                    Box::new(Not(Box::new(Val(Var("b".to_string())))))
                )),
                Box::new(Not(Box::new(And(
                    Box::new(Val(Var("c".to_string()))),
                    Box::new(Val(Var("d".to_string())))
                ))))
            )
        );
        assert_expr!(
            "not not(true)",
            Not(Box::new(Not(Box::new(Val(Bool(true))))))
        );
    }

    #[test]
    fn test_dict() {
        assert_expr!(
            "{{ x=1, z = 2 }}",
            AnonymousStruct(vec![
                ("x".to_string(), Val(Nat(1))),
                ("z".to_string(), Val(Nat(2)))
            ])
        );
        assert_expr!(
            "{{
                x= 1,
                z = \"hoge\",
                }}",
            AnonymousStruct(vec![
                ("x".to_string(), Val(Nat(1))),
                ("z".to_string(), Val(Str("hoge".to_string())))
            ])
        );
    }

    #[test]
    fn test_arrayed() {
        assert_expr!("[]", Arrayed(vec![]));
        assert_expr!(
            "[1, 2, 3,]",
            Arrayed(vec![Val(Nat(1)), Val(Nat(2)), Val(Nat(3))])
        );
        assert_expr!(
            "[1, 2, 3]",
            Arrayed(vec![Val(Nat(1)), Val(Nat(2)), Val(Nat(3))])
        );
        assert_expr!(
            "[1, 2, 3]//comment",
            Arrayed(vec![Val(Nat(1)), Val(Nat(2)), Val(Nat(3))])
        );
        assert_expr!(
            "[1, //one
                2, //two
                3]//comment",
            Arrayed(vec![Val(Nat(1)), Val(Nat(2)), Val(Nat(3))])
        );
    }

    #[test]
    fn test_apply() {
        assert_expr!(
            "X(1, -2, \"x\")",
            Apply(
                "X".to_string(),
                vec![Val(Nat(1)), Val(Int(-2)), Val(Str("x".to_string()))]
            )
        );
        assert_expr!(
            "X(1, // comment
                -2, \"x\")//comment",
            Apply(
                "X".to_string(),
                vec![Val(Nat(1)), Val(Int(-2)), Val(Str("x".to_string()))]
            )
        );
        assert_expr!(
            "X.Y(1, -2, \"x\")",
            Apply(
                "X".to_string(),
                vec![Apply(
                    "Y".to_string(),
                    vec![Val(Nat(1)), Val(Int(-2)), Val(Str("x".to_string()))]
                )]
            )
        );
    }

    #[test]
    fn test_field_apply() {
        assert_expr!(
            "X { x=1, y=-2, z=\"x\"}",
            FieledApply(
                "X".to_string(),
                vec![
                    ("x".to_string(), Val(Nat(1))),
                    ("y".to_string(), Val(Int(-2))),
                    ("z".to_string(), Val(Str("x".to_string())))
                ]
            )
        );
        assert_expr!(
            "X {//comment
                x=1, //comment
                // comment
                y=-2,//comment
                z=\"x\"
                } // comment",
            FieledApply(
                "X".to_string(),
                vec![
                    ("x".to_string(), Val(Nat(1))),
                    ("y".to_string(), Val(Int(-2))),
                    ("z".to_string(), Val(Str("x".to_string())))
                ]
            )
        );
        assert_expr!(
            "X.Y.Z{}",
            Apply(
                "X".to_string(),
                vec![Apply(
                    "Y".to_string(),
                    vec![FieledApply("Z".to_string(), vec![])]
                )]
            )
        );
    }

    #[test]
    fn test_blocked() {
        assert_expr!(
            "{
                let x: Int = 1;
                let y = -2;
                x + y
                }
                ",
            Blocked(Box::new(Config(
                vec![
                    Statement::Let("x".to_string(), Typing::Int, Val(Nat(1))),
                    Statement::Let("y".to_string(), Typing::Any, Val(Int(-2))),
                ],
                Add(
                    Box::new(Val(Var("x".to_string()))),
                    Box::new(Val(Var("y".to_string())))
                )
            )))
        );
    }

    #[test]
    fn test_as_cast() {
        assert_expr!("1 as Int", AsCast(Box::new(Val(Nat(1))), Typing::Int));
        assert_expr!(
            "1 as Int
                // Nat -> Int",
            AsCast(Box::new(Val(Nat(1))), Typing::Int)
        );
        assert_expr!(
            "(1+1) as Int",
            AsCast(
                Box::new(Add(Box::new(Val(Nat(1))), Box::new(Val(Nat(1))))),
                Typing::Int
            )
        );
    }
}
