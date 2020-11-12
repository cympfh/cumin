use crate::parser::config::*;
use crate::parser::util::*;
use crate::parser::value::*;
use combine::error::ParseError;
use combine::parser::char::{char, string};
use combine::parser::combinator::attempt;
use combine::stream::Stream;
use combine::{choice, many, parser, sep_by};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Val(Value),
    Apply(String, Vec<Expr>),
    FieledApply(String, Vec<(String, Expr)>),
    AnonymousStruct(Vec<(String, Expr)>),
    Add(Box<Expr>, Box<Expr>),
    Arrayed(Vec<Expr>),
    Blocked(Box<Config>),
}

parser! {
    pub fn expr[Input]()(Input) -> Expr
    where [
        Input: Stream<Token = char>,
        Input::Error: ParseError<char, Input::Range, Input::Position>,
    ]
    {
        // F(x,...)
        let apply_expr = {
            let inner_sep = sep_by::<Vec<_>,_, _, _>(
                (
                    commentable_spaces(),
                    expr(),
                    commentable_spaces()
                ).map(|t| t.1),
                char(','));
            let inner_trailing = many::<Vec<_>, _, _>(
                (
                    commentable_spaces(),
                    expr(),
                    commentable_spaces(),
                    char(','),
                    commentable_spaces()
                ).map(|t| t.1));
            (
                commentable_spaces(),
                identifier(),
                commentable_spaces(),
                char('('),
                choice!(attempt(inner_trailing), inner_sep),
                char(')'),
                commentable_spaces(),
            ).map(|t| Expr::Apply(t.1, t.4))
        };

        // F { x=val, }
        let fielded_apply_expr = {
            let inner_sep = sep_by::<Vec<_>, _, _, _>(
                (
                    commentable_spaces(),
                    identifier(),
                    commentable_spaces(),
                    char('='),
                    commentable_spaces(),
                    expr(),
                    commentable_spaces(),
                ).map(|t| (t.1, t.5)),
                char(','));
            let inner_trailing = many::<Vec<_>, _, _>(
                (
                    commentable_spaces(),
                    identifier(),
                    commentable_spaces(),
                    char('='),
                    commentable_spaces(),
                    expr(),
                    commentable_spaces(),
                    char(','),
                    commentable_spaces(),
                ).map(|t| (t.1, t.5)));
            (
                commentable_spaces(),
                identifier(),
                commentable_spaces(),
                char('{'),
                choice!(attempt(inner_trailing), inner_sep),
                char('}'),
                commentable_spaces(),
            ).map(|t| Expr::FieledApply(t.1, t.4))
        };

        // {{ x=val, }}
        let dict_expr = {
            let inner_sep = sep_by::<Vec<(String, Expr)>, _, _, _>(
                (
                    commentable_spaces(),
                    identifier(),
                    commentable_spaces(),
                    char('='),
                    commentable_spaces(),
                    expr(),
                    commentable_spaces(),
                ).map(|t| (t.1, t.5)),
                char(',')
            );
            let inner_trailing = many::<Vec<_>, _, _>(
                (
                    commentable_spaces(),
                    identifier(),
                    commentable_spaces(),
                    char('='),
                    commentable_spaces(),
                    expr(),
                    commentable_spaces(),
                    char(','),
                    commentable_spaces(),
                ).map(|t| (t.1, t.5))
            );
            (
                commentable_spaces(),
                string("{{"),
                choice!(attempt(inner_trailing), inner_sep),
                string("}}"),
                commentable_spaces(),
            ).map(|t| Expr::AnonymousStruct(t.2))
        };

        // { statement...; expr }
        let blocked_expr = (
            commentable_spaces(),
            char('{'),
            commentable_spaces(),
            config(),
            commentable_spaces(),
            char('}'),
            commentable_spaces(),
        ).map(|t| Expr::Blocked(Box::new(t.3)));

        // _ + _
        let add_expr = (value(), commentable_spaces(), char('+'), commentable_spaces(), expr())
            .map(|t| Expr::Add(Box::new(Expr::Val(t.0)), Box::new(t.4)));

        let value_expr = value().map(|x: Value| Expr::Val(x));

        let arrayed_expr = {
            let inner_sep =
                sep_by::<Vec<Expr>, _, _, _>(
                    (commentable_spaces(),
                    expr(),
                    commentable_spaces()).map(|t| t.1),
                    char(',')
                );
            let inner_trailing = many::<Vec<Expr>, _, _>(
                    (
                        commentable_spaces(),
                        expr(),
                        commentable_spaces(),
                        char(','),
                        commentable_spaces()
                    ).map(|t| t.1)
                );
            (
                commentable_spaces(),
                char('['),
                choice!(attempt(inner_trailing), inner_sep),
                char(']'),
                commentable_spaces(),
            ).map(|t| Expr::Arrayed(t.2))
        };

        choice!(
            attempt(apply_expr),
            attempt(add_expr),
            attempt(dict_expr),
            blocked_expr,
            attempt(fielded_apply_expr),
            arrayed_expr,
            value_expr
        )
    }
}

#[cfg(test)]
mod test_expr {
    use crate::parser::expr::*;
    use crate::parser::statement::*;
    use crate::parser::typing::*;
    use combine::Parser;
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

    #[test]
    fn test_dict() {
        assert_eq!(
            expr().parse("{{ x=1, z = 2 }}"),
            Ok((
                AnonymousStruct(vec![
                    ("x".to_string(), Val(Nat(1))),
                    ("z".to_string(), Val(Nat(2)))
                ]),
                ""
            )),
        );
        assert_eq!(
            expr().parse(
                "{{
                x= 1,
                z = \"hoge\",
            }}"
            ),
            Ok((
                AnonymousStruct(vec![
                    ("x".to_string(), Val(Nat(1))),
                    ("z".to_string(), Val(Str("hoge".to_string())))
                ]),
                ""
            )),
        );
    }

    #[test]
    fn test_arrayed() {
        assert_eq!(expr().parse("[]"), Ok((Arrayed(vec![]), "")));
        assert_eq!(
            expr().parse("[1, 2, 3,]"),
            Ok((Arrayed(vec![Val(Nat(1)), Val(Nat(2)), Val(Nat(3))]), ""))
        );
        assert_eq!(
            expr().parse("[1, 2, 3]"),
            Ok((Arrayed(vec![Val(Nat(1)), Val(Nat(2)), Val(Nat(3))]), ""))
        );
        assert_eq!(
            expr().parse("[1, 2, 3]//comment"),
            Ok((Arrayed(vec![Val(Nat(1)), Val(Nat(2)), Val(Nat(3))]), ""))
        );
    }

    #[test]
    fn test_apply() {
        assert_eq!(
            expr().parse("X(1, -2, \"x\")"),
            Ok((
                Apply(
                    "X".to_string(),
                    vec![Val(Nat(1)), Val(Int(-2)), Val(Str("x".to_string()))]
                ),
                ""
            ))
        );
        assert_eq!(
            expr().parse(
                "X(1, // comment
            -2, \"x\")//comment"
            ),
            Ok((
                Apply(
                    "X".to_string(),
                    vec![Val(Nat(1)), Val(Int(-2)), Val(Str("x".to_string()))]
                ),
                ""
            ))
        );
    }

    #[test]
    fn test_field_apply() {
        assert_eq!(
            expr().parse("X { x=1, y=-2, z=\"x\"}"),
            Ok((
                FieledApply(
                    "X".to_string(),
                    vec![
                        ("x".to_string(), Val(Nat(1))),
                        ("y".to_string(), Val(Int(-2))),
                        ("z".to_string(), Val(Str("x".to_string())))
                    ]
                ),
                ""
            ))
        );
        assert_eq!(
            expr().parse(
                "X {//comment
                x=1, //comment
                // comment
                y=-2,//comment
                z=\"x\"
            } // comment"
            ),
            Ok((
                FieledApply(
                    "X".to_string(),
                    vec![
                        ("x".to_string(), Val(Nat(1))),
                        ("y".to_string(), Val(Int(-2))),
                        ("z".to_string(), Val(Str("x".to_string())))
                    ]
                ),
                ""
            ))
        );
    }

    #[test]
    fn test_blocked() {
        assert_eq!(
            expr().parse(
                "// block
                {
                    let x: Int = 1;
                    let y = -2;
                    x + y
                }
                "
            ),
            Ok((
                Blocked(Box::new(Config(
                    vec![
                        Statement::Let("x".to_string(), Typing::Int, Val(Nat(1))),
                        Statement::Let("y".to_string(), Typing::Any, Val(Int(-2))),
                    ],
                    Expr::Add(
                        Box::new(Val(Var("x".to_string()))),
                        Box::new(Val(Var("y".to_string())))
                    )
                ))),
                ""
            ))
        );
    }
}
