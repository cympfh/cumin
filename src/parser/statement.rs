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
                attempt(spaces().skip(char(':')).skip(spaces()).with(typing()).skip(spaces())),
                spaces().map(|_| Typing::Any)
            );
            commentable_spaces()
                .skip(string("let"))
                .skip(space())
                .skip(spaces())
                .with(identifier())
                .and(type_annotation)
                .skip(char('='))
                .skip(spaces())
                .and(expr())
                .skip(spaces())
                .skip(char(';'))
                .skip(commentable_spaces())
                .map(|((id, typ), e)| Statement::Let(id, typ, e))
        };

        // struct id { id: typing [= expr] [,] }
        let struct_stmt = {
            let inner_separated = sep_by::<Vec<(String, Typing, Option<Expr>)>, _, _, _>(
                commentable_spaces()
                    .with(identifier())
                    .skip(spaces())
                    .skip(char(':'))
                    .skip(spaces())
                    .and(typing())
                    .skip(commentable_spaces())
                    .and(optional(char('=').with(spaces()).with(expr())))
                    .skip(commentable_spaces())
                    .map(|((id, typ), e)| (id, typ, e)),
                char(','));
            let inner_trailing = many::<Vec<(String, Typing, Option<Expr>)>, _, _>(
                commentable_spaces()
                    .with(identifier())
                    .skip(spaces())
                    .skip(char(':'))
                    .skip(spaces())
                    .and(typing())
                    .skip(spaces())
                    .and(optional(char('=').with(spaces()).with(expr())))
                    .skip(commentable_spaces())
                    .skip(char(','))
                    .skip(commentable_spaces())
                    .map(|((id, typ), e)| (id, typ, e)));
            commentable_spaces()
                .skip(string("struct"))
                .skip(space())
                .skip(spaces())
                .with(identifier())
                .skip(spaces())
                .skip(char('{'))
                .skip(commentable_spaces())
                .and(choice!(attempt(inner_trailing), inner_separated))
                .skip(commentable_spaces())
                .skip(char('}'))
                .skip(commentable_spaces())
                .map(|t| Statement::Struct(t.0, t.1))
        };

        // enum id { id, id [,] }
        let enum_stmst = {
            let inner_separated = sep_by(
                commentable_spaces()
                    .with(identifier())
                    .skip(commentable_spaces()),
                char(','));
            let inner_trailing = many1(
                commentable_spaces()
                    .with(identifier())
                    .skip(commentable_spaces())
                    .skip(char(','))
                    .skip(commentable_spaces()));
            commentable_spaces()
                .skip(string("enum"))
                .skip(space())
                .skip(spaces())
                .with(identifier())
                .skip(spaces())
                .skip(char('{'))
                .and(choice!(attempt(inner_trailing), inner_separated))
                .skip(char('}'))
                .skip(commentable_spaces())
                .map(|t| Statement::Enum(t.0, t.1))
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
