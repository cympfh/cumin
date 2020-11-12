use crate::parser::expr::*;
use crate::parser::typing::*;
use crate::parser::util::*;
use combine::error::ParseError;
use combine::parser::char::{char, space, spaces, string};
use combine::parser::combinator::attempt;
use combine::stream::Stream;
use combine::{choice, many, many1, optional, parser, sep_by};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let(String, Typing, Expr),
    Struct(String, Vec<(String, Typing, Option<Expr>)>), // StructName, [(name, type, default)]
    Enum(String, Vec<String>),
}

parser! {
    pub fn stmt[Input]()(Input) -> Statement
    where [
        Input: Stream<Token = char>,
        Input::Error: ParseError<char, Input::Range, Input::Position>,
    ]
    {
        // let id = expr;
        // let id: typing = expr;
        let let_stmt = {
            let type_annotation = choice!(
                attempt((spaces(), char(':'), spaces(), typing(), spaces()).map(|t| t.3)),
                spaces().map(|_: ()| Typing::Any)
            );
            (
                commentable_spaces(),
                string("let"),
                space(),
                spaces(),
                identifier(),
                type_annotation,
                char('='),
                spaces(),
                expr(),
                spaces(),
                char(';'),
                commentable_spaces(),
            )
                .map(|t| Statement::Let(t.4, t.5, t.8))
        };

        // struct id { id: typing [= expr] [,] }
        let struct_stmt = {
            let inner_separated = sep_by::<Vec<(String, Typing, Option<Expr>)>, _, _, _>(
                (
                    commentable_spaces(),
                    identifier(),
                    spaces(),
                    char(':'),
                    spaces(),
                    typing(),
                    commentable_spaces(),
                    optional(
                        (
                            char('='),
                            spaces(),
                            expr(),
                        ).map(|t| t.2)),
                    commentable_spaces()
                ).map(|t| (t.1, t.5, t.7)),
                char(','));
            let inner_trailing = many::<Vec<(String, Typing, Option<Expr>)>, _, _>(
                (
                    commentable_spaces(),
                    identifier(),
                    spaces(),
                    char(':'),
                    spaces(),
                    typing(),
                    spaces(),
                    optional(
                        (
                            char('='),
                            spaces(),
                            expr(),
                        ).map(|t| t.2)),
                    commentable_spaces(),
                    char(','),
                    commentable_spaces(),
                ).map(|t| (t.1, t.5, t.7)));
            (
                commentable_spaces(),
                string("struct"),
                space(),
                spaces(),
                identifier(),
                spaces(),
                char('{'),
                commentable_spaces(),
                choice!(attempt(inner_trailing), inner_separated),
                commentable_spaces(),
                char('}'),
                commentable_spaces(),
            )
                .map(|t| Statement::Struct(t.4, t.8))
        };

        // enum id { id, id [,] }
        let enum_stmst = {
            let inner_separated = sep_by::<Vec<String>, _, _, _>(
                (
                    commentable_spaces(),
                    identifier(),
                    commentable_spaces(),
                ).map(|t: ((), String, ())| t.1),
                char(','));
            let inner_trailing = many1::<Vec<String>, _, _>(
                (
                    commentable_spaces(),
                    identifier(),
                    commentable_spaces(),
                    char(','),
                    commentable_spaces(),
                ).map(|t: ((), String, (), char, ())| t.1)
            );
            (
                commentable_spaces(),
                string("enum"),
                space(),
                spaces(),
                identifier(),
                spaces(),
                char('{'),
                choice!(attempt(inner_trailing), inner_separated),
                char('}'),
                commentable_spaces()
            )
                .map(|t| Statement::Enum(t.4, t.7))
        };

        choice!(
            attempt(struct_stmt),
            attempt(let_stmt),
            attempt(enum_stmst)
        )
    }
}

#[cfg(test)]
mod test_statement {
    use crate::parser::statement::*;
    use crate::parser::value::*;
    use combine::Parser;
    use Expr::*;
    use Statement::*;
    use Value::*;

    #[test]
    fn test_let() {
        assert_eq!(
            stmt().parse("let s = -2;"),
            Ok((Let("s".to_string(), Typing::Any, Val(Int(-2))), ""))
        );
        assert_eq!(
            stmt().parse("let z: Nat = 3;"),
            Ok((Let("z".to_string(), Typing::Nat, Val(Nat(3))), ""))
        );
        assert_eq!(
            stmt().parse("let s:Nat=2; "),
            Ok((Let("s".to_string(), Typing::Nat, Val(Nat(2))), ""))
        );
        assert_eq!(
            stmt().parse("let name = \"hoge\" ; "),
            Ok((
                Let(
                    "name".to_string(),
                    Typing::Any,
                    Val(Str("hoge".to_string()))
                ),
                ""
            ))
        );
    }

    #[test]
    fn test_struct() {
        assert_eq!(
            stmt().parse("struct X {} "),
            Ok((Struct("X".to_string(), vec![]), ""))
        );
        assert_eq!(
            stmt().parse(
                "// comment
                struct X {}"
            ),
            Ok((Struct("X".to_string(), vec![]), ""))
        );
        assert_eq!(
            stmt().parse("struct Point { x: Int, y:Int} "),
            Ok((
                Struct(
                    "Point".to_string(),
                    vec![
                        ("x".to_string(), Typing::Int, None),
                        ("y".to_string(), Typing::Int, None),
                    ]
                ),
                ""
            ))
        );
        // comma-trailing
        assert_eq!(
            stmt().parse("struct Point { x: Int, y:Int, } "),
            Ok((
                Struct(
                    "Point".to_string(),
                    vec![
                        ("x".to_string(), Typing::Int, None),
                        ("y".to_string(), Typing::Int, None),
                    ]
                ),
                ""
            ))
        );
        // with default values
        assert_eq!(
            stmt().parse(
                "struct Point {
                name: String = \"hoge\",
                x: Int, y:Int=0, } "
            ),
            Ok((
                Struct(
                    "Point".to_string(),
                    vec![
                        (
                            "name".to_string(),
                            Typing::String,
                            Some(Val(Str("hoge".to_string())))
                        ),
                        ("x".to_string(), Typing::Int, None),
                        ("y".to_string(), Typing::Int, Some(Val(Nat(0)))),
                    ]
                ),
                ""
            ))
        );
    }

    #[test]
    fn test_enum() {
        // comma-separating
        assert_eq!(
            stmt().parse(
                "
            enum Z {
                A,B, C,D
            }
            "
            ),
            Ok((
                Enum(
                    "Z".to_string(),
                    vec![
                        "A".to_string(),
                        "B".to_string(),
                        "C".to_string(),
                        "D".to_string(),
                    ]
                ),
                ""
            ))
        );
        // comma-trailing
        assert_eq!(
            stmt().parse(
                "
            enum Z{
                Z1,//,,,
                Z2,
            }
            "
            ),
            Ok((
                Enum("Z".to_string(), vec!["Z1".to_string(), "Z2".to_string()]),
                ""
            ))
        );
    }
}
