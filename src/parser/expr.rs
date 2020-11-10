use crate::parser::comment::*;
use crate::parser::value::*;
use combine::parser::char::{alpha_num, char, spaces, string};
use combine::parser::combinator::attempt;
use combine::stream::Stream;
use combine::{between, choice, many, many1, parser, sep_by};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Val(Value),
    Apply(String, Vec<Expr>),
    FieledApply(String, Vec<(String, Expr)>),
    AnonymousStruct(Vec<(String, Expr)>),
    Add(Box<Expr>, Box<Expr>),
    Arrayed(Vec<Expr>),
}

parser! {
    pub fn expr[Input]()(Input) -> Expr where [Input: Stream<Token=char>] {
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
                many1(alpha_num()),
                char('('),
                choice!(attempt(inner_trailing), inner_sep),
                char(')')
            ).map(|t| Expr::Apply(t.0, t.2))
        };

        // F { x=val, }
        let fielded_apply_expr = {
            let inner_sep = sep_by::<Vec<_>, _, _, _>(
                (
                    commentable_spaces(),
                    many1(alpha_num()),
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
                    many1(alpha_num()),
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
                many1(alpha_num()),
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
                    spaces(),
                    many1(alpha_num()),
                    spaces(),
                    char('='),
                    spaces(),
                    expr(),
                    spaces()
                ).map(|t| (t.1, t.5)),
                char(',')
            );
            let inner_trailing = many::<Vec<_>, _, _>(
                (
                    spaces(),
                    many1(alpha_num()),
                    spaces(),
                    char('='),
                    spaces(),
                    expr(),
                    spaces(),
                    char(','),
                    spaces()
                ).map(|t| (t.1, t.5))
            );
            between(
                string("{{"),
                string("}}"),
                choice!(attempt(inner_trailing), inner_sep),
            ).map(Expr::AnonymousStruct)
        };

        // _ + _
        let add_expr = (value(), spaces(), char('+'), spaces(), expr())
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
            between(
                char('['),
                char(']'),
                choice!(attempt(inner_trailing), inner_sep),
            ).map(Expr::Arrayed)
        };

        choice!(
            attempt(apply_expr),
            attempt(add_expr),
            attempt(dict_expr),
            attempt(fielded_apply_expr),
            arrayed_expr,
            value_expr
        )
    }
}

#[cfg(test)]
mod test_expr {
    use crate::parser::expr::*;
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
    }
}
