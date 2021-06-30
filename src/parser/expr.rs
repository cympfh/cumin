use crate::parser::cumin::*;
use crate::parser::typing::*;
use crate::parser::util::*;
use crate::parser::value::*;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space1,
    combinator::{map, opt, peek},
    multi::{fold_many0, separated_list0, separated_list1},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Val(Value),
    Var(String),
    Apply(String, Vec<Expr>),
    FieledApply(String, Vec<(String, Expr)>),
    AnonymousStruct(Vec<(String, Typing, Expr)>),
    Concat(Box<Expr>, Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Minus(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Xor(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Equal(Box<Expr>, Box<Expr>),
    Less(Box<Expr>, Box<Expr>),
    Arrayed(Vec<Expr>),
    Tuple(Vec<Expr>),
    Blocked(Box<Cumin>),
    AsCast(Box<Expr>, Typing),
}

// <EXPR> ::= <AS>
// <LOGIC> ::= <AB> {==, !=, <, >, <=, >=} <AB> | <AB>
// <AB> ::= <TERM> {and,or,xor,+,-} <TERM> | <TERM>
// <TERM> ::= <AS> {*,/,**} <AS> | <AS>
// <AS> ::= <FACTOR> as <FACTOR> | <FACTOR>
// <FACTOR> ::= ( <EXPR> ) | -<TERM> | not <TERM>
//            | f(x) | S{x=x} | { ... } | Z::X | {{ ... }}
//            | <EXPR> as <TYPE> | [ <EXPR> ,... ]

pub fn expr(input: &str) -> IResult<&str, Expr> {
    terminated(logic_expr, commentable_spaces)(input)
}

pub fn logic_expr(input: &str) -> IResult<&str, Expr> {
    let compare = map(
        tuple((
            terminated(ab_expr, commentable_spaces),
            terminated(
                alt((
                    tag("=="),
                    tag("!="),
                    tag("<="),
                    tag(">="),
                    tag("<"),
                    tag(">"),
                )),
                commentable_spaces,
            ),
            terminated(ab_expr, commentable_spaces),
        )),
        |(x, op, y)| match op {
            "==" => Expr::Equal(Box::new(x), Box::new(y)),
            "!=" => Expr::Not(Box::new(Expr::Equal(Box::new(x), Box::new(y)))),
            "<=" => Expr::Not(Box::new(Expr::Less(Box::new(y), Box::new(x)))),
            ">=" => Expr::Not(Box::new(Expr::Less(Box::new(x), Box::new(y)))),
            "<" => Expr::Less(Box::new(x), Box::new(y)),
            ">" => Expr::Less(Box::new(y), Box::new(x)),
            _ => panic!(),
        },
    );
    alt((compare, ab_expr))(input)
}

fn ab_expr(input: &str) -> IResult<&str, Expr> {
    let (input, x) = term(input)?;
    let (input, _) = commentable_spaces(input)?;
    fold_many0(
        tuple((
            terminated(
                alt((
                    tag("and"),
                    tag("or"),
                    tag("xor"),
                    tag("++"),
                    tag("+"),
                    tag("-"),
                )),
                commentable_spaces,
            ),
            term,
        )),
        x,
        |acc, (op, val)| match op {
            "and" => Expr::And(Box::new(acc), Box::new(val)),
            "or" => Expr::Or(Box::new(acc), Box::new(val)),
            "xor" => Expr::Xor(Box::new(acc), Box::new(val)),
            "++" => Expr::Concat(Box::new(acc), Box::new(val)),
            "+" => Expr::Add(Box::new(acc), Box::new(val)),
            "-" => Expr::Sub(Box::new(acc), Box::new(val)),
            _ => panic!(),
        },
    )(input)
}

fn term(input: &str) -> IResult<&str, Expr> {
    let (input, x) = as_expr(input)?;
    let (input, _) = commentable_spaces(input)?;
    fold_many0(
        tuple((
            terminated(
                alt((tag("**"), tag("*"), tag("/"), tag("%"))),
                commentable_spaces,
            ),
            as_expr,
        )),
        x,
        |acc, (op, val)| match op {
            "**" => Expr::Pow(Box::new(acc), Box::new(val)),
            "*" => Expr::Mul(Box::new(acc), Box::new(val)),
            "/" => Expr::Div(Box::new(acc), Box::new(val)),
            "%" => Expr::Mod(Box::new(acc), Box::new(val)),
            _ => panic!(),
        },
    )(input)
}

fn as_expr(input: &str) -> IResult<&str, Expr> {
    // <expr> as <typing>
    let as_expr = map(
        tuple((
            terminated(factor, commentable_spaces),
            terminated(tag("as"), commentable_spaces),
            typing,
        )),
        |(e, _, typ)| Expr::AsCast(Box::new(e), typ),
    );
    alt((as_expr, factor))(input)
}

fn factor(input: &str) -> IResult<&str, Expr> {
    let parened = map(
        tuple((
            terminated(tag("("), commentable_spaces),
            terminated(expr, commentable_spaces),
            tag(")"),
        )),
        |(_, e, _)| e,
    );
    let minused = map(preceded(tag("-"), ab_expr), |e| Expr::Minus(Box::new(e)));
    let notted = map(
        preceded(
            preceded(tag("not"), peek(alt((space1, tag("("))))),
            preceded(spaces, term),
        ),
        |e| Expr::Not(Box::new(e)),
    );

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
                        terminated(identifier, commentable_spaces),
                        terminated(tag("="), commentable_spaces),
                        terminated(expr, commentable_spaces),
                    )),
                    |(name, _, e)| (name, e),
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

    // ( <expr> , )
    let tuple_expr = map(
        tuple((
            tag("("),
            separated_list1(tuple((tag(","), commentable_spaces)), expr),
            tag(")"),
        )),
        |item| Expr::Tuple(item.1),
    );

    // <value>
    let avalue = map(value, Expr::Val);

    // <variable>
    let vvalue = map(identifier, Expr::Var);

    terminated(
        alt((
            avalue,
            notted,
            minused,
            parened,
            dict_expr,
            blocked_expr,
            arrayed_expr,
            apply_expr,
            tuple_expr,
            field_apply_expr,
            vvalue,
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
        assert_expr!("x // var", Expr::Var("x".to_string()));
    }

    #[test]
    fn test_concat() {
        assert_expr!(
            "[] ++ []",
            Expr::Concat(
                Box::new(Expr::Arrayed(vec![])),
                Box::new(Expr::Arrayed(vec![])),
            )
        );
        assert_expr!(
            "[] ++ [1] ++ [2]",
            Expr::Concat(
                Box::new(Expr::Concat(
                    Box::new(Expr::Arrayed(vec![])),
                    Box::new(Expr::Arrayed(vec![Expr::Val(Nat(1))])),
                )),
                Box::new(Expr::Arrayed(vec![Expr::Val(Nat(2))])),
            )
        );
    }

    #[test]
    fn test_arith() {
        assert_expr!("1 // one", Val(Nat(1)));
        assert_expr!("( 1 )", Val(Nat(1)));
        assert_expr!("-1", Val(Int(-1)));
        assert_expr!("0 + 1", Add(Box::new(Val(Nat(0))), Box::new(Val(Nat(1)))));
        assert_expr!(
            "0 + x",
            Add(Box::new(Val(Nat(0))), Box::new(Expr::Var("x".to_string())))
        );
        assert_expr!(
            "x + 2",
            Add(Box::new(Expr::Var("x".to_string())), Box::new(Val(Nat(2))))
        );
        assert_expr!(
            "x + y + z",
            Add(
                Box::new(Add(
                    Box::new(Expr::Var("x".to_string())),
                    Box::new(Expr::Var("y".to_string())),
                )),
                Box::new(Expr::Var("z".to_string())),
            )
        );
        assert_expr!(
            "x - y",
            Sub(
                Box::new(Expr::Var("x".to_string())),
                Box::new(Expr::Var("y".to_string())),
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
                    Box::new(Expr::Var("x".to_string())),
                    Box::new(Expr::Var("y".to_string())),
                )),
                Box::new(Expr::Var("z".to_string()))
            )
        );
        assert_expr!("5 % 2", Mod(Box::new(Val(Nat(5))), Box::new(Val(Nat(2)))));
        assert_expr!("5 %2", Mod(Box::new(Val(Nat(5))), Box::new(Val(Nat(2)))));
        assert_expr!("5% 2", Mod(Box::new(Val(Nat(5))), Box::new(Val(Nat(2)))));
        assert_expr!("5%2", Mod(Box::new(Val(Nat(5))), Box::new(Val(Nat(2)))));
        assert_expr!("1+-1", Add(Box::new(Val(Nat(1))), Box::new(Val(Int(-1)))));
        assert_expr!("1 / 2", Div(Box::new(Val(Nat(1))), Box::new(Val(Nat(2)))));
        assert_expr!("1  /2", Div(Box::new(Val(Nat(1))), Box::new(Val(Nat(2)))));
        assert_expr!("1/  2", Div(Box::new(Val(Nat(1))), Box::new(Val(Nat(2)))));
        assert_expr!("1/2", Div(Box::new(Val(Nat(1))), Box::new(Val(Nat(2)))));
        assert_expr!(
            "1 + 2 - 3",
            Sub(
                Box::new(Add(Box::new(Val(Nat(1))), Box::new(Val(Nat(2))),)),
                Box::new(Val(Nat(3)))
            )
        );
        assert_expr!(
            "1 * 2 * 3 / 4",
            Div(
                Box::new(Mul(
                    Box::new(Mul(Box::new(Val(Nat(1))), Box::new(Val(Nat(2))),)),
                    Box::new(Val(Nat(3)))
                )),
                Box::new(Val(Nat(4)))
            )
        );
        assert_expr!(
            "1 + 2 * 3",
            Add(
                Box::new(Val(Nat(1))),
                Box::new(Mul(Box::new(Val(Nat(2))), Box::new(Val(Nat(3))),))
            )
        );
        assert_expr!(
            "(1 + 2) * ((3) / 4 - 5)",
            Mul(
                Box::new(Add(Box::new(Val(Nat(1))), Box::new(Val(Nat(2))),)),
                Box::new(Sub(
                    Box::new(Div(Box::new(Val(Nat(3))), Box::new(Val(Nat(4))),)),
                    Box::new(Val(Nat(5)))
                ))
            )
        );
        assert_expr!("-(-2)", Minus(Box::new(Val(Int(-2)))));
        assert_expr!("-x", Minus(Box::new(Expr::Var("x".to_string()))));
        assert_expr!(
            "f(x) + 1",
            Add(
                Box::new(Apply("f".to_string(), vec![Expr::Var("x".to_string())])),
                Box::new(Val(Nat(1)))
            )
        );
        assert_expr!(
            "f(x) + g(z)",
            Add(
                Box::new(Apply("f".to_string(), vec![Expr::Var("x".to_string())])),
                Box::new(Apply("g".to_string(), vec![Expr::Var("z".to_string())])),
            )
        );
    }

    #[test]
    fn test_bool_expression() {
        assert_expr!("true", Val(Bool(true)));
        assert_expr!("false", Val(Bool(false)));
        assert_expr!("not false", Not(Box::new(Val(Bool(false)))));
        assert_expr!("not(false)", Not(Box::new(Val(Bool(false)))));
        assert_expr!("notfalse", Var("notfalse".to_string()));
        assert_expr!(
            "true or false",
            Or(Box::new(Val(Bool(true))), Box::new(Val(Bool(false))))
        );
        assert_expr!(
            "true xor false",
            Xor(Box::new(Val(Bool(true))), Box::new(Val(Bool(false))))
        );
        assert_expr!(
            "(a or not b) xor (not c and d)",
            Xor(
                Box::new(Or(
                    Box::new(Expr::Var("a".to_string())),
                    Box::new(Not(Box::new(Expr::Var("b".to_string()))))
                )),
                Box::new(And(
                    Box::new(Not(Box::new(Expr::Var("c".to_string())))),
                    Box::new(Expr::Var("d".to_string()))
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
                    Box::new(Expr::Var("x".to_string())),
                    Box::new(Expr::Var("y".to_string()))
                )
            )))
        );
    }

    #[test]
    fn test_as_cast() {
        assert_expr!("1 as Int", AsCast(Box::new(Val(Nat(1))), Typing::Int));
        assert_expr!(
            "{ 1 } as Int",
            AsCast(
                Box::new(Blocked(Box::new(Cumin(vec![], Val(Nat(1)))))),
                Typing::Int
            )
        );
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
        assert_expr!(
            "f(1+1) as Int",
            AsCast(
                Box::new(Apply(
                    "f".to_string(),
                    vec![Add(Box::new(Val(Nat(1))), Box::new(Val(Nat(1))))]
                )),
                Typing::Int
            )
        );
        assert_expr!(
            "f(1) + 2 as Int",
            Add(
                Box::new(Apply("f".to_string(), vec![Val(Nat(1))])),
                Box::new(AsCast(Box::new(Val(Nat(2))), Typing::Int))
            )
        );
    }

    #[test]
    fn test_bool() {
        assert_expr!("true", Val(Bool(true)));
        assert_expr!("not x", Not(Box::new(Expr::Var("x".to_string()))));
        assert_expr!(
            "not true or true",
            Or(
                Box::new(Not(Box::new(Val(Bool(true))))),
                Box::new(Val(Bool(true)))
            )
        );
        assert_expr!(
            "true or not true",
            Or(
                Box::new(Val(Bool(true))),
                Box::new(Not(Box::new(Val(Bool(true)))))
            )
        );
        assert_expr!(
            "x and y",
            And(
                Box::new(Expr::Var("x".to_string())),
                Box::new(Expr::Var("y".to_string()))
            )
        );
        assert_expr!(
            "true and false or true xor false",
            Xor(
                Box::new(Or(
                    Box::new(And(Box::new(Val(Bool(true))), Box::new(Val(Bool(false))))),
                    Box::new(Val(Bool(true)))
                )),
                Box::new(Val(Bool(false)))
            )
        );
        assert_expr!(
            "true and (false or not true)",
            And(
                Box::new(Val(Bool(true))),
                Box::new(Or(
                    Box::new(Val(Bool(false))),
                    Box::new(Not(Box::new(Val(Bool(true)))))
                ))
            )
        );
    }

    #[test]
    fn test_compare() {
        assert_expr!(
            "1 == 2",
            Equal(Box::new(Val(Nat(1))), Box::new(Val(Nat(2))))
        );
        assert_expr!(
            "1 <= 2",
            Not(Box::new(Less(Box::new(Val(Nat(2))), Box::new(Val(Nat(1))))))
        );
        assert_expr!(
            "1 + 1 == 2 - 0",
            Equal(
                Box::new(Add(Box::new(Val(Nat(1))), Box::new(Val(Nat(1))))),
                Box::new(Sub(Box::new(Val(Nat(2))), Box::new(Val(Nat(0)))))
            )
        );
        assert_expr!(
            "(1 <= 2) == false",
            Equal(
                Box::new(Not(Box::new(Less(
                    Box::new(Val(Nat(2))),
                    Box::new(Val(Nat(1)))
                )))),
                Box::new(Val(Bool(false)))
            )
        );
    }

    #[test]
    fn test_var() {
        assert_expr!("hoge", Expr::Var("hoge".to_string()));
        assert_expr!("_hoge0", Expr::Var("_hoge0".to_string()));
    }

    #[test]
    fn test_tuple() {
        assert_expr!("(1, 2)", Expr::Tuple(vec![Val(Nat(1)), Val(Nat(2)),]));
    }
}
