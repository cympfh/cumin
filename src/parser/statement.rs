use crate::parser::expr::*;
use combine::error::ParseError;
use combine::parser::char::{alpha_num, char, space, spaces, string};
use combine::parser::combinator::attempt;
use combine::stream::Stream;
use combine::{choice, many, many1, none_of, parser, sep_by, Parser};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Let(String, String, Expr),
    Struct(String, Vec<(String, String)>),
    Enum(String, Vec<String>),
}

fn comment<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    spaces()
        .with(string("//"))
        .with(many::<String, _, _>(none_of("\n".chars())))
        .with(spaces())
}

fn commentable_spaces<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice!(attempt(comment()), spaces())
}

parser! {
    pub fn stmt[Input]()(Input) -> Statement where [Input: Stream<Token=char>] {

        // let id = expr;
        let let_stmt = (
            commentable_spaces(),
            string("let"),
            space(),
            spaces(),
            many1(alpha_num()),
            spaces(),
            char('='),
            spaces(),
            expr(),
            spaces(),
            char(';'),
            commentable_spaces(),
        )
            .map(|t| Statement::Let(t.4, "Any".to_string(), t.8));

        // let id: type = expr;
        let let_typed_stmt = (
            commentable_spaces(),
            string("let"),
            space(),
            spaces(),
            many1(alpha_num()),
            spaces(),
            char(':'),
            spaces(),
            many1(alpha_num()),
            spaces(),
            char('='),
            spaces(),
            expr(),
            spaces(),
            char(';'),
            commentable_spaces(),
        )
            .map(|t| Statement::Let(t.4, t.8, t.12));

        // struct id { id: id, id: id } -- comma sparated
        let struct_stmt = {
            let struct_inner = sep_by(
                (
                    commentable_spaces(),
                    many1(alpha_num()),
                    spaces(),
                    char(':'),
                    spaces(),
                    many1(alpha_num()),
                    commentable_spaces()
                ).map(|t| (t.1, t.5)), char(','));
            (
                commentable_spaces(),
                string("struct"),
                space(),
                spaces(),
                many1(alpha_num()),
                spaces(),
                char('{'),
                commentable_spaces(),
                struct_inner,
                commentable_spaces(),
                char('}'),
                commentable_spaces(),
            )
                .map(|t| Statement::Struct(t.4, t.8))
        };

        // struct -- comma trailing
        let struct_comma_stmt = {
            let struct_inner = many(
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
                struct_inner,
                commentable_spaces(),
                char('}'),
                commentable_spaces(),
            )
                .map(|t| Statement::Struct(t.4, t.8))
        };

        // enum -- comma separated
        let enum_stmst = {
            let inner = sep_by(
                (
                commentable_spaces(),
                many1(alpha_num()),
                commentable_spaces()
                ).map(|t| t.1),
                char(','));
            (
                commentable_spaces(),
                string("enum"),
                space(),
                spaces(),
                many1(alpha_num()),
                spaces(),
                char('{'),
                inner,
                char('}'),
                commentable_spaces()
            )
                .map(|t|
                    Statement::Enum(t.4,t.7))
        };

        // enum -- comma trailing
        let enum_comma_stmst = {
            let inner = many1(
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
                inner,
                char('}'),
                commentable_spaces()
            )
                .map(|t|
                    Statement::Enum(t.4,t.7))
        };

        choice!(
            attempt(struct_comma_stmt),
            attempt(struct_stmt),
            attempt(let_typed_stmt),
            attempt(let_stmt),
            attempt(enum_comma_stmst),
            attempt(enum_stmst)
        )
    }
}

#[cfg(test)]
mod test_statement {
    use crate::parser::statement::*;
    use crate::parser::value::*;
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
    fn test_comment() {
        assert_eq!(comment().parse("// hoge"), Ok(((), "")));
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
