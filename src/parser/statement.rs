use crate::parser::expr::*;
use combine::parser::char::{alpha_num, char, space, spaces, string};
use combine::parser::combinator::attempt;
use combine::stream::Stream;
use combine::{choice, many, many1, parser, sep_by, Parser};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Let(String, Expr),
    Struct(String, Vec<(String, String)>),
}

parser! {
    pub fn stmt[Input]()(Input) -> Statement where [Input: Stream<Token=char>] {

        // let id = expr;
        let let_stmt = (
            spaces(),
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
            spaces(),
        )
            .map(|t| Statement::Let(t.4, t.8));

        // struct id { id: id, id: id } -- comma sparated
        let struct_stmt = {
            let struct_inner = sep_by(
                (
                    spaces(),
                    many1(alpha_num()),
                    spaces(),
                    char(':'),
                    spaces(),
                    many1(alpha_num()),
                    spaces()
                ).map(|t| (t.1, t.5)), char(','));
            (
                spaces(),
                string("struct"),
                space(),
                spaces(),
                many1(alpha_num()),
                spaces(),
                char('{'),
                spaces(),
                struct_inner,
                spaces(),
                char('}'),
                spaces(),
            )
                .map(|t| Statement::Struct(t.4, t.8))
        };

        // struct -- comma trailing
        let struct_stmt_comma = {
            let struct_inner = many(
                (
                    spaces(),
                    many1(alpha_num()),
                    spaces(),
                    char(':'),
                    spaces(),
                    many1(alpha_num()),
                    spaces(),
                    char(','),
                    spaces(),
                ).map(|t| (t.1, t.5)));
            (
                spaces(),
                string("struct"),
                space(),
                spaces(),
                many1(alpha_num()),
                spaces(),
                char('{'),
                spaces(),
                struct_inner,
                spaces(),
                char('}'),
                spaces(),
            )
                .map(|t| Statement::Struct(t.4, t.8))
        };

        choice!(
            attempt(struct_stmt_comma),
            attempt(struct_stmt),
            attempt(let_stmt)
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
            Ok((Let("s".to_string(), Val(Int(-2))), ""))
        );
        assert_eq!(
            stmt().parse("let s=2; "),
            Ok((Let("s".to_string(), Val(Nat(2))), ""))
        );
        assert_eq!(
            stmt().parse("let name = \"hoge\" ; "),
            Ok((Let("name".to_string(), Val(Str("hoge".to_string()))), ""))
        );
    }

    #[test]
    fn test_struct() {
        assert_eq!(
            stmt().parse("struct X {} "),
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
}
