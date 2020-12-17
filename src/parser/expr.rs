use crate::parser::cumin::*;
use crate::parser::logic::logic_expr;
use crate::parser::typing::*;
use crate::parser::util::*;
use crate::parser::value::*;

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, opt},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, terminated, tuple},
    IResult,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Val(Value),
    Apply(String, Vec<Expr>),
    FieledApply(String, Vec<(String, Expr)>),
    AnonymousStruct(Vec<(String, Typing, Expr)>),
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
    Blocked(Box<Cumin>),
    AsCast(Box<Expr>, Typing),
}

pub fn expr(input: &str) -> IResult<&str, Expr> {
    // <identifier>.<identifier> ( <expr>, )
    let apply_expr = map(
        tuple((
            separated_list1(tag("."), identifier),
            spaces,
            tag("("),
            commentable_spaces,
            separated_list0(tuple((tag(","), commentable_spaces)), expr),
            opt(tuple((tag(","), commentable_spaces))),
            tag(")"),
        )),
        |(fs, _, _, _, args, _, _)| {
            let n = fs.len();
            assert!(n > 0);
            let mut e = Expr::Apply(fs[n - 1].to_string(), args);
            for i in (0..n - 1).rev() {
                e = Expr::Apply(fs[i].to_string(), vec![e]);
            }
            e
        },
    );

    // <identifier>.<identifier> { <identifier> = <expr> }
    let field_apply_expr = map(
        tuple((
            separated_list1(tag("."), identifier),
            commentable_spaces,
            tag("{"),
            commentable_spaces,
            separated_list0(
                tuple((tag(","), commentable_spaces)),
                map(
                    tuple((
                        identifier,
                        commentable_spaces,
                        tag("="),
                        commentable_spaces,
                        expr,
                        commentable_spaces,
                    )),
                    |(name, _, _, _, e, _)| (name, e),
                ),
            ),
            opt(tuple((tag(","), commentable_spaces))),
            tag("}"),
        )),
        |(fs, _, _, _, args, _, _)| {
            let n = fs.len();
            assert!(n > 0);
            let mut e = Expr::FieledApply(fs[n - 1].to_string(), args);
            for i in (0..n - 1).rev() {
                e = Expr::Apply(fs[i].to_string(), vec![e]);
            }
            e
        },
    );

    // {{ <identifier> = <exp> , }}
    let dict_expr = map(
        tuple((
            tag("{{"),
            commentable_spaces,
            separated_list0(
                tuple((tag(","), commentable_spaces)),
                map(
                    tuple((
                        identifier,
                        commentable_spaces,
                        opt(map(
                            tuple((tag(":"), commentable_spaces, typing, commentable_spaces)),
                            |(_, _, typ, _)| typ,
                        )),
                        tag("="),
                        commentable_spaces,
                        expr,
                        commentable_spaces,
                    )),
                    |(name, _, typ, _, _, e, _)| (name, typ.unwrap_or(Typing::Any), e),
                ),
            ),
            opt(tuple((tag(","), commentable_spaces))),
            tag("}}"),
        )),
        |(_, _, items, _, _)| Expr::AnonymousStruct(items),
    );

    // { <cumin> }
    let blocked_expr = map(delimited(tag("{"), cumin, tag("}")), |cumin| {
        Expr::Blocked(Box::new(cumin))
    });

    // <expr> as <typing>
    let as_expr = map(
        tuple((
            alt((
                map(
                    tuple((
                        tag("("),
                        commentable_spaces,
                        expr,
                        commentable_spaces,
                        tag(")"),
                    )),
                    |(_, _, e, _, _)| e,
                ),
                map(value, Expr::Val),
            )),
            commentable_spaces,
            tag("as"),
            commentable_spaces,
            typing,
        )),
        |(e, _, _, _, typ)| Expr::AsCast(Box::new(e), typ),
    );

    // [ <expr> , ]
    let arrayed_expr = map(
        tuple((
            tag("["),
            commentable_spaces,
            separated_list0(
                tuple((tag(","), commentable_spaces)),
                terminated(expr, commentable_spaces),
            ),
            opt(tuple((tag(","), commentable_spaces))),
            tag("]"),
        )),
        |(_, _, elems, _, _)| Expr::Arrayed(elems),
    );

    // <value>
    let value_expr = map(value, |val| Expr::Val(val));

    terminated(
        alt((
            dict_expr,
            blocked_expr,
            arrayed_expr,
            apply_expr,
            field_apply_expr,
            as_expr,
            logic_expr,
            value_expr,
        )),
        commentable_spaces,
    )(input)
}

#[cfg(test)]
mod test_expr {
    use crate::parser::expr::*;
    use crate::parser::statement::*;
    use Expr::*;
    use Value::*;

    macro_rules! assert_expr {
        ($code: expr, $expected: expr) => {
            assert_eq!(expr($code), Ok(("", $expected)));
        };
    }

    #[test]
    fn test_value() {
        assert_expr!("1 // one", Val(Nat(1)));
        assert_expr!("-1 // one", Val(Int(-1)));
        assert_expr!(
            "true
            // one",
            Val(Bool(true))
        );
        assert_expr!("x // var", Val(Var("x".to_string())));
    }

    #[test]
    fn test_arith() {
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
                Box::new(And(
                    Box::new(Not(Box::new(Val(Var("c".to_string()))))),
                    Box::new(Val(Var("d".to_string())))
                ))
            )
        );
        assert_expr!(
            "not not(true)",
            Not(Box::new(Not(Box::new(Val(Bool(true))))))
        );
    }

    #[test]
    fn test_dict() {
        assert_expr!("{{ }}", AnonymousStruct(vec![]));
        assert_expr!(
            "{{x=1,}}",
            AnonymousStruct(vec![("x".to_string(), Typing::Any, Val(Nat(1)))])
        );
        assert_expr!(
            "{{x: Int = 1,}}",
            AnonymousStruct(vec![("x".to_string(), Typing::Int, Val(Nat(1)))])
        );
        assert_expr!(
            "{{ x=1, z = 2 }}",
            AnonymousStruct(vec![
                ("x".to_string(), Typing::Any, Val(Nat(1))),
                ("z".to_string(), Typing::Any, Val(Nat(2)))
            ])
        );
        assert_expr!(
            "{{
                x:Int= 1,
                z = \"hoge\",
                }}",
            AnonymousStruct(vec![
                ("x".to_string(), Typing::Int, Val(Nat(1))),
                ("z".to_string(), Typing::Any, Val(Str("hoge".to_string())))
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
        assert_expr!("f()", Apply("f".to_string(), vec![]));
        assert_expr!("f(1)", Apply("f".to_string(), vec![Val(Nat(1))]));
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
        assert_expr!("X{}", FieledApply("X".to_string(), vec![]));
        assert_expr!(
            "X{x=1}",
            FieledApply("X".to_string(), vec![("x".to_string(), Val(Nat(1)))])
        );
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
            Blocked(Box::new(Cumin(
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
