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
                .map(|t| Statement::Let(t.3, t.4, t.7))
        };

        // struct id { id: typing [= expr] [,] }
        let struct_stmt = {
            let inner_separated = sep_by::<Vec<(String, Typing, Option<Expr>)>, _, _, _>(
                (
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
                ).map(|t| (t.0, t.4, t.6)),
                char(',').with(commentable_spaces()));
            let inner_trailing = many::<Vec<(String, Typing, Option<Expr>)>, _, _>(
                (
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
                ).map(|t| (t.0, t.4, t.6)));
            (
                string("struct"),
                space(),
                spaces(),
                identifier(),
                spaces(),
                char('{'),
                commentable_spaces(),
                choice!(attempt(inner_trailing), inner_separated),
                char('}'),
                commentable_spaces(),
            )
                .map(|t| Statement::Struct(t.3, t.7))
        };

        // enum id { id, id [,] }
        let enum_stmst = {
            let inner_separated = sep_by::<Vec<String>, _, _, _>(
                (
                    identifier(),
                    commentable_spaces(),
                ).map(|t: (String, ())| t.0),
                char(',').with(commentable_spaces()));
            let inner_trailing = many1::<Vec<String>, _, _>(
                (
                    identifier(),
                    commentable_spaces(),
                    char(','),
                    commentable_spaces(),
                ).map(|t: (String, (), char, ())| t.0)
            );
            (
                string("enum"),
                space(),
                spaces(),
                identifier(),
                spaces(),
                char('{'),
                commentable_spaces(),
                choice!(attempt(inner_trailing), inner_separated),
                char('}'),
                commentable_spaces()
            )
                .map(|t| Statement::Enum(t.3, t.7))
        };

        choice!(
            attempt(struct_stmt),
            attempt(let_stmt),
            enum_stmst
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

    macro_rules! assert_stmt {
        ($code: expr, $expected: expr) => {
            assert_eq!(stmt().parse($code).ok().unwrap().0, $expected);
        };
    }

    #[test]
    fn test_let() {
        assert_stmt!(
            "let s = -2;",
            Let("s".to_string(), Typing::Any, Val(Int(-2)))
        );
        assert_stmt!(
            "let z: Nat = 3;",
            Let("z".to_string(), Typing::Nat, Val(Nat(3)))
        );
        assert_stmt!(
            "let s:Nat=2; ",
            Let("s".to_string(), Typing::Nat, Val(Nat(2)))
        );
        assert_stmt!(
            "let name = \"hoge\" ; ",
            Let(
                "name".to_string(),
                Typing::Any,
                Val(Str("hoge".to_string()))
            )
        );
    }

    #[test]
    fn test_struct() {
        assert_stmt!("struct X {} ", Struct("X".to_string(), vec![]));
        assert_stmt!("struct X {} // comment", Struct("X".to_string(), vec![]));
        assert_stmt!(
            "struct Point { x: Int, y:Int} ",
            Struct(
                "Point".to_string(),
                vec![
                    ("x".to_string(), Typing::Int, None),
                    ("y".to_string(), Typing::Int, None),
                ]
            )
        );
        // comma-trailing
        assert_stmt!(
            "struct Point { x: Int, y:Int, } ",
            Struct(
                "Point".to_string(),
                vec![
                    ("x".to_string(), Typing::Int, None),
                    ("y".to_string(), Typing::Int, None),
                ]
            )
        );
        // with default values
        assert_stmt!(
            "struct Point {
                name: String = \"hoge\",
                x: Int, y:Int=0, } ",
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
            )
        );
    }

    #[test]
    fn test_enum() {
        // comma-separating
        assert_stmt!(
            "enum Z {
                A,B, C,D
            }
            ",
            Enum(
                "Z".to_string(),
                vec![
                    "A".to_string(),
                    "B".to_string(),
                    "C".to_string(),
                    "D".to_string(),
                ]
            )
        );
        // comma-trailing
        assert_stmt!(
            "enum Z{
                Z1,//,,,
                Z2,
            }
            ",
            Enum("Z".to_string(), vec!["Z1".to_string(), "Z2".to_string()])
        );
    }
}
