use crate::parser::expr::*;
use crate::parser::util::*;
use combine::parser::char::{alpha_num, char, space, spaces, string};
use combine::parser::combinator::attempt;
use combine::stream::Stream;
use combine::{choice, many, many1, parser, sep_by};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let(String, String, Expr),
    Struct(String, Vec<(String, String)>),
    Enum(String, Vec<String>),
}

parser! {
    pub fn stmt[Input]()(Input) -> Statement where [Input: Stream<Token=char>] {

        // let id = expr;
        // let id: type = expr;
        let let_stmt = {
            let typing = choice!(
                attempt((spaces(), char(':'), spaces(), many1(alpha_num()), spaces()).map(|t| t.3)),
                spaces().map(|_| "Any".to_string())
                );
            (
                commentable_spaces(),
                string("let"),
                space(),
                spaces(),
                identifier(),
                typing,
                char('='),
                spaces(),
                expr(),
                spaces(),
                char(';'),
                commentable_spaces(),
            )
                .map(|t| Statement::Let(t.4, t.5, t.8))
        };

        // struct id { id: id, id: id [,] }
        let struct_stmt = {
            let inner_separated = sep_by(
                (
                    commentable_spaces(),
                    many1(alpha_num()),
                    spaces(),
                    char(':'),
                    spaces(),
                    many1(alpha_num()),
                    commentable_spaces()
                ).map(|t| (t.1, t.5)), char(','));
            let inner_trailing = many(
                (
                    commentable_spaces(),
                    many1(alpha_num()),
                    spaces(),
                    char(':'),
                    spaces(),
                    many1(alpha_num()),
                    spaces(),
                    char(','),
                    commentable_spaces(),
                ).map(|t| (t.1, t.5)));
            (
                commentable_spaces(),
                string("struct"),
                space(),
                spaces(),
                many1(alpha_num()),
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

        // enum -- comma separated
        let enum_stmst = {
            let inner_separated = sep_by(
                (
                commentable_spaces(),
                many1(alpha_num()),
                commentable_spaces()
                ).map(|t| t.1),
                char(','));
            let inner_trailing = many1(
                (
                commentable_spaces(),
                many1(alpha_num()),
                commentable_spaces(),
                char(','),
                commentable_spaces(),
                ).map(|t| t.1)
            );
            (
                commentable_spaces(),
                string("enum"),
                space(),
                spaces(),
                many1(alpha_num()),
                spaces(),
                char('{'),
                choice!(attempt(inner_trailing), inner_separated),
                char('}'),
                commentable_spaces()
            )
                .map(|t|
                    Statement::Enum(t.4,t.7))
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
            Ok((Let("s".to_string(), "Any".to_string(), Val(Int(-2))), ""))
        );
        assert_eq!(
            stmt().parse("let z: Nat = 3;"),
            Ok((Let("z".to_string(), "Nat".to_string(), Val(Nat(3))), ""))
        );
        assert_eq!(
            stmt().parse("let s:Nat=2; "),
            Ok((Let("s".to_string(), "Nat".to_string(), Val(Nat(2))), ""))
        );
        assert_eq!(
            stmt().parse("let name = \"hoge\" ; "),
            Ok((
                Let(
                    "name".to_string(),
                    "Any".to_string(),
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
                        ("x".to_string(), "Int".to_string()),
                        ("y".to_string(), "Int".to_string()),
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
                        ("x".to_string(), "Int".to_string()),
                        ("y".to_string(), "Int".to_string()),
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
