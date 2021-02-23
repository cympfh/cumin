use crate::parser::expr::*;
use crate::parser::typing::*;
use crate::parser::util::*;
use nom::combinator;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    combinator::{map, opt},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, terminated, tuple},
    IResult,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let(String, Typing, Expr),
    Struct(String, Vec<(String, Typing, Option<Expr>)>), // StructName, [(name, type, default)]
    Enum(String, Vec<String>),
    Type(String, Vec<Typing>),
    Import(String),
    Fun(String, Vec<(String, Typing, Option<Expr>)>, Expr),
}

pub fn stmt(input: &str) -> IResult<&str, Statement> {
    // let id = expr;
    // let id: typing = expr;
    let let_stmt = {
        let type_annotation = alt((
            map(
                tuple((tag(":"), commentable_spaces, typing, commentable_spaces)),
                |(_, _, typ, _)| typ,
            ),
            combinator::value(Typing::Any, commentable_spaces),
        ));
        map(
            tuple((
                tag("let"),
                commentable_spaces,
                identifier,
                type_annotation,
                tag("="),
                commentable_spaces,
                expr,
                tag(";"),
            )),
            |(_, _, name, typ, _, _, e, _)| Statement::Let(name, typ, e),
        )
    };

    // struct id { id: typing [= expr] [,] }
    let struct_stmt = {
        let inner = separated_list0(
            tuple((tag(","), commentable_spaces)),
            map(
                tuple((
                    identifier,
                    commentable_spaces,
                    opt(map(
                        tuple((tag(":"), commentable_spaces, typing, commentable_spaces)),
                        |(_, _, typ, _)| typ,
                    )),
                    opt(map(
                        tuple((tag("="), commentable_spaces, expr, commentable_spaces)),
                        |(_, _, e, _)| e,
                    )),
                )),
                |(name, _, typ, default_value)| (name, typ.unwrap_or(Typing::Any), default_value),
            ),
        );
        map(
            tuple((
                tag("struct"),
                commentable_spaces,
                identifier,
                commentable_spaces,
                tag("{"),
                commentable_spaces,
                inner,
                opt(tuple((tag(","), commentable_spaces))),
                tag("}"),
            )),
            |(_, _, name, _, _, _, items, _, _)| Statement::Struct(name, items),
        )
    };

    // enum id { id, id [,] }
    let enum_stmst = {
        let inner = separated_list0(
            tuple((tag(","), commentable_spaces)),
            terminated(identifier, commentable_spaces),
        );
        map(
            tuple((
                tag("enum"),
                commentable_spaces,
                identifier,
                commentable_spaces,
                tag("{"),
                commentable_spaces,
                inner,
                opt(tuple((tag(","), commentable_spaces))),
                tag("}"),
            )),
            |(_, _, name, _, _, _, items, _, _)| Statement::Enum(name, items),
        )
    };

    // type <id> = <Typing> | ... | <Typing> ;
    let type_stmt = {
        let typelist = separated_list1(
            tuple((tag("|"), commentable_spaces)),
            terminated(typing, commentable_spaces),
        );
        map(
            tuple((
                tag("type"),
                commentable_spaces,
                identifier,
                commentable_spaces,
                tag("="),
                commentable_spaces,
                typelist,
                tag(";"),
            )),
            |(_, _, name, _, _, _, typs, _)| Statement::Type(name, typs),
        )
    };

    // use "<path>" ;
    let use_stmt = map(
        tuple((
            tag("use"),
            commentable_spaces,
            delimited(tag("\""), is_not("\""), tag("\"")),
            commentable_spaces,
            tag(";"),
        )),
        |(_, _, path, _, _)| Statement::Import(path.to_string()),
    );

    // let fname(args) = code;
    let let_fn_stmt = {
        let args = separated_list0(
            tuple((tag(","), commentable_spaces)),
            map(
                tuple((
                    identifier,
                    commentable_spaces,
                    tag(":"),
                    commentable_spaces,
                    typing,
                    commentable_spaces,
                    opt(map(
                        tuple((tag("="), commentable_spaces, expr, commentable_spaces)),
                        |(_, _, e, _)| e,
                    )),
                )),
                |(name, _, _, _, typ, _, default_value)| (name, typ, default_value),
            ),
        );
        map(
            tuple((
                tag("let"),
                commentable_spaces,
                identifier,
                commentable_spaces,
                delimited(tag("("), args, tag(")")),
                commentable_spaces,
                tag("="),
                commentable_spaces,
                expr,
                tag(";"),
            )),
            |(_, _, fname, _, args, _, _, _, body, _)| Statement::Fun(fname, args, body),
        )
    };
    // fn fname(args) = code;
    let fn_stmt = {
        let args = separated_list0(
            tuple((tag(","), commentable_spaces)),
            map(
                tuple((
                    identifier,
                    commentable_spaces,
                    tag(":"),
                    commentable_spaces,
                    typing,
                    commentable_spaces,
                    opt(map(
                        tuple((tag("="), commentable_spaces, expr, commentable_spaces)),
                        |(_, _, e, _)| e,
                    )),
                )),
                |(name, _, _, _, typ, _, default_value)| (name, typ, default_value),
            ),
        );
        map(
            tuple((
                tag("fn"),
                commentable_spaces,
                identifier,
                commentable_spaces,
                delimited(tag("("), args, tag(")")),
                commentable_spaces,
                tag("="),
                commentable_spaces,
                expr,
                tag(";"),
            )),
            |(_, _, fname, _, args, _, _, _, body, _)| Statement::Fun(fname, args, body),
        )
    };

    terminated(
        alt((
            fn_stmt,
            let_fn_stmt,
            let_stmt,
            struct_stmt,
            enum_stmst,
            type_stmt,
            use_stmt,
        )),
        commentable_spaces,
    )(input)
}

#[cfg(test)]
mod test_statement {
    use crate::parser::statement::*;
    use crate::parser::value::*;
    use Expr::*;
    use Statement::*;
    use Value::*;

    macro_rules! assert_stmt {
        ($code: expr, $expected: expr) => {
            assert_eq!(stmt($code), Ok(("", $expected)))
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
        // without type-annotation & with default values
        assert_stmt!(
            "struct P { x: Int, y: Int = 2, z = 2 }",
            Struct(
                "P".to_string(),
                vec![
                    ("x".to_string(), Typing::Int, None),
                    ("y".to_string(), Typing::Int, Some(Val(Nat(2)))),
                    ("z".to_string(), Typing::Any, Some(Val(Nat(2)))),
                ]
            )
        );
    }

    #[test]
    fn test_enum() {
        assert_stmt!("enum A{}", Enum("A".to_string(), vec![]));
        assert_stmt!("enum A{B}", Enum("A".to_string(), vec!["B".to_string()]));
        assert_stmt!("enum A{B,}", Enum("A".to_string(), vec!["B".to_string()]));
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

    #[test]
    fn test_type() {
        assert_stmt!(
            "type T = A;",
            Type("T".to_string(), vec![Typing::UserTyping("A".to_string())])
        );
        assert_stmt!(
            "type T = A | B | Int;",
            Type(
                "T".to_string(),
                vec![
                    Typing::UserTyping("A".to_string()),
                    Typing::UserTyping("B".to_string()),
                    Typing::Int,
                ]
            )
        );
    }

    #[test]
    fn test_use() {
        assert_stmt!(
            "use \"hoge/fuga/piyo\";",
            Import("hoge/fuga/piyo".to_string())
        );
        assert_stmt!(
            "use \"hoge/fuga/piyo\" ; // import",
            Import("hoge/fuga/piyo".to_string())
        );
    }

    #[test]
    fn test_fn() {
        assert_stmt!(
            "fn zero() = 0;",
            Fun("zero".to_string(), vec![], Val(Nat(0)))
        );
        assert_stmt!(
            "let zero() = 0;",
            Fun("zero".to_string(), vec![], Val(Nat(0)))
        );
        assert_stmt!(
            "fn zero(x: Int) = x; // identity",
            Fun(
                "zero".to_string(),
                vec![("x".to_string(), Typing::Int, None)],
                Expr::Var("x".to_string())
            )
        );
        assert_stmt!(
            "fn zero(x: Int, y: Int = 2) = x + y; // identity",
            Fun(
                "zero".to_string(),
                vec![
                    ("x".to_string(), Typing::Int, None),
                    ("y".to_string(), Typing::Int, Some(Val(Nat(2)))),
                ],
                Add(
                    Box::new(Expr::Var("x".to_string())),
                    Box::new(Expr::Var("y".to_string()))
                )
            )
        );
    }
}
