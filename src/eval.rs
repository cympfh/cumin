use crate::builtins;
use crate::json::*;
use crate::parser::config::*;
use crate::parser::expr::*;
use crate::parser::statement::*;
use crate::parser::typing::*;
use crate::parser::value::*;
use std::env;

use std::collections::HashMap;

use Statement::*;

pub fn eval(config: Config) -> JSON {
    let mut env = Environ::new();
    let val = eval_conf(&mut env, &config);
    JSON::from_cumin(val)
}

fn eval_conf(env: &mut Environ, conf: &Config) -> Value {
    // collect struct
    for stmt in conf.0.iter() {
        match stmt {
            Struct(name, fields) => {
                env.structs.insert(name.clone(), fields.clone());
            }
            _ => (),
        }
    }

    // collect enums
    for stmt in conf.0.iter() {
        match stmt {
            Enum(name, variants) => {
                env.enums.insert(name.clone(), variants.clone());
            }
            _ => (),
        }
    }

    // collect let
    for stmt in conf.0.iter() {
        match stmt {
            Let(id, ty, expr) => {
                let val = eval_expr(&env, expr);
                env.vars.insert(id.clone(), (ty.clone(), val));
            }
            _ => (),
        }
    }

    eval_expr(&env, &conf.1)
}

fn eval_expr(env: &Environ, expr: &Expr) -> Value {
    use Expr::*;
    use Value::*;
    match expr {
        Val(value) => eval_value(&env, value),
        Apply(name, args) => {
            let values: Vec<Value> = args.iter().map(|x| eval_expr(&env, &x)).collect();
            match name.as_str() {
                "Some" => {
                    assert!(values.len() == 1);
                    Just(Box::new(values[0].clone()))
                }
                "not" => {
                    assert!(values.len() == 1);
                    let e = Not(Box::new(Val(values[0].clone())));
                    eval_expr(&env, &e)
                }
                "concat" => builtins::concat(&values),
                "reverse" => {
                    assert!(values.len() == 1);
                    builtins::reverse(&values[0])
                }
                _ if env.structs.contains_key(name) => {
                    let fields = env.structs.get(name).unwrap();
                    assert!(fields.len() == values.len());
                    let items: Vec<(String, Value)> = fields
                        .iter()
                        .zip(values.iter())
                        .map(|((name, _ty, _default), value)| (name.to_string(), value.clone()))
                        .collect();
                    Dict(items)
                }
                _ => panic!("Cannot resolve name {}", name),
            }
        }
        FieledApply(f, items) => {
            if let Some(fields) = env.structs.get(f) {
                let args: std::collections::HashMap<String, Expr> = items.iter().cloned().collect();
                let items: Vec<(String, Value)> = fields
                    .iter()
                    .map(|(name, _ty, default)| {
                        if let Some(arg) = args.get(&name.to_string()) {
                            (name.to_string(), eval_expr(&env, &arg))
                        } else {
                            if let Some(e) = default {
                                (name.to_string(), eval_expr(&env, e))
                            } else {
                                panic!("Cannot find field {}", name)
                            }
                        }
                    })
                    .collect();
                Dict(items)
            } else {
                panic!("Cannot resolve name {}", f)
            }
        }
        AnonymousStruct(items) => {
            let items = items
                .iter()
                .map(|(name, val)| (name.to_string(), eval_expr(&env, &val)))
                .collect();
            Dict(items)
        }
        Add(x, y) => {
            let a = eval_expr(&env, &x);
            let b = eval_expr(&env, &y);
            match (a, b) {
                (Nat(x), Nat(y)) => Nat(x + y),
                (Nat(x), Int(y)) => Int(x as i128 + y),
                (Nat(x), Float(y)) => Float(x as f64 + y),
                (Int(x), Nat(y)) => Int(x + y as i128),
                (Int(x), Int(y)) => Int(x + y),
                (Int(x), Float(y)) => Float(x as f64 + y),
                (Float(x), Nat(y)) => Float(x + y as f64),
                (Float(x), Int(y)) => Float(x + y as f64),
                (Float(x), Float(y)) => Float(x + y),
                (Str(x), Str(y)) => {
                    let mut z = x;
                    z.push_str(&y);
                    Str(z)
                }
                _ => panic!("Cant compute {:?} + {:?}", x, y),
            }
        }
        Sub(x, y) => {
            let a = eval_expr(&env, &x);
            let b = eval_expr(&env, &y);
            match (a, b) {
                (Nat(x), Nat(y)) => {
                    if x >= y {
                        Nat(x - y)
                    } else {
                        Int(x as i128 - y as i128)
                    }
                }
                (Nat(x), Int(y)) => Int(x as i128 - y),
                (Nat(x), Float(y)) => Float(x as f64 - y),
                (Int(x), Nat(y)) => Int(x - y as i128),
                (Int(x), Int(y)) => Int(x - y),
                (Int(x), Float(y)) => Float(x as f64 - y),
                (Float(x), Nat(y)) => Float(x - y as f64),
                (Float(x), Int(y)) => Float(x - y as f64),
                (Float(x), Float(y)) => Float(x - y),
                _ => panic!("Cant compute {:?} - {:?}", x, y),
            }
        }
        Mul(x, y) => {
            let a = eval_expr(&env, &x);
            let b = eval_expr(&env, &y);
            match (a, b) {
                (Nat(x), Nat(y)) => Nat(x * y),
                (Nat(x), Int(y)) => Int(x as i128 * y),
                (Nat(x), Float(y)) => Float(x as f64 * y),
                (Int(x), Nat(y)) => Int(x * y as i128),
                (Int(x), Int(y)) => Int(x * y),
                (Int(x), Float(y)) => Float(x as f64 * y),
                (Float(x), Nat(y)) => Float(x * y as f64),
                (Float(x), Int(y)) => Float(x * y as f64),
                (Float(x), Float(y)) => Float(x * y),
                _ => panic!("Cant compute {:?} * {:?}", x, y),
            }
        }
        Div(x, y) => {
            let a = eval_expr(&env, &x);
            let b = eval_expr(&env, &y);
            match (a, b) {
                (Nat(x), Nat(y)) => Nat(x / y),
                (Nat(x), Int(y)) => Int(x as i128 / y),
                (Nat(x), Float(y)) => Float(x as f64 / y),
                (Int(x), Nat(y)) => Int(x / y as i128),
                (Int(x), Int(y)) => Int(x / y),
                (Int(x), Float(y)) => Float(x as f64 / y),
                (Float(x), Nat(y)) => Float(x / y as f64),
                (Float(x), Int(y)) => Float(x / y as f64),
                (Float(x), Float(y)) => Float(x / y),
                _ => panic!("Cant compute {:?} / {:?}", x, y),
            }
        }
        Pow(x, y) => {
            let a = eval_expr(&env, &x);
            let b = eval_expr(&env, &y);
            match (a, b) {
                (Nat(x), Nat(y)) => Nat(x.pow(y as u32)),
                (Nat(x), Int(y)) => {
                    if y >= 0 {
                        Nat(x.pow(y as u32))
                    } else {
                        Float((x as f64).powi(y as i32))
                    }
                }
                (Nat(x), Float(y)) => Float((x as f64).powf(y)),
                (Int(x), Nat(y)) => Int(x.pow(y as u32)),
                (Int(x), Int(y)) => {
                    if y >= 0 {
                        Int(x.pow(y as u32))
                    } else {
                        Float((x as f64).powi(y as i32))
                    }
                }
                (Int(x), Float(y)) => Float((x as f64).powf(y)),
                (Float(x), Nat(y)) => Float(x.powi(y as i32)),
                (Float(x), Int(y)) => Float(x.powi(y as i32)),
                (Float(x), Float(y)) => Float(x.powf(y)),
                _ => panic!("Cant compute {:?} / {:?}", x, y),
            }
        }
        Minus(x) => {
            let a = eval_expr(&env, &x);
            match a {
                Nat(x) => Int(-(x as i128)),
                Int(x) => Int(-x),
                Float(x) => Float(-x),
                _ => panic!("Cant compute -({:?})", x),
            }
        }
        And(x, y) => {
            let a = eval_expr(&env, &x);
            let b = eval_expr(&env, &y);
            match (a, b) {
                (Bool(x), Bool(y)) => Bool(x && y),
                _ => panic!("Cant compute {:?} and {:?}", x, y),
            }
        }
        Or(x, y) => {
            let a = eval_expr(&env, &x);
            let b = eval_expr(&env, &y);
            match (a, b) {
                (Bool(x), Bool(y)) => Bool(x || y),
                _ => panic!("Cant compute {:?} or {:?}", x, y),
            }
        }
        Xor(x, y) => {
            let a = eval_expr(&env, &x);
            let b = eval_expr(&env, &y);
            match (a, b) {
                (Bool(x), Bool(y)) => Bool(x ^ y),
                _ => panic!("Cant compute {:?} xor {:?}", x, y),
            }
        }
        Not(x) => {
            let a = eval_expr(&env, &x);
            match a {
                Bool(x) => Bool(!x),
                _ => panic!("Cant compute not {:?}", x),
            }
        }
        Equal(x, y) => {
            let a = eval_expr(&env, &x);
            let b = eval_expr(&env, &y);
            match (a, b) {
                (Nat(x), Nat(y)) => Bool(x == y),
                (Nat(x), Int(y)) => Bool(x as i128 == y),
                (Int(x), Nat(y)) => Bool(x == y as i128),
                (Int(x), Int(y)) => Bool(x == y),
                (Float(x), Float(y)) => Bool(x == y),
                (Bool(x), Bool(y)) => Bool(x == y),
                _ => panic!("Cant compare {:?} == {:?}", x, y),
            }
        }
        Less(x, y) => {
            let a = eval_expr(&env, &x);
            let b = eval_expr(&env, &y);
            match (a, b) {
                (Nat(x), Nat(y)) => Bool(x < y),
                (Nat(x), Int(y)) => Bool((x as i128) < y),
                (Int(x), Nat(y)) => Bool(x < y as i128),
                (Int(x), Int(y)) => Bool(x < y),
                (Float(x), Float(y)) => Bool(x < y),
                _ => panic!("Cant compare {:?} < {:?}", x, y),
            }
        }
        Arrayed(elements) => {
            let elements = elements.iter().map(|e| eval_expr(&env, &e)).collect();
            Array(elements)
        }
        Blocked(conf_inner) => {
            let mut env_inner: Environ = (*env).clone();
            eval_conf(&mut env_inner, &conf_inner)
        }
        AsCast(expr, typ) => {
            let val = eval_expr(&env, &expr);
            val.cast(typ)
        }
    }
}

fn eval_value(env: &Environ, value: &Value) -> Value {
    use Value::*;
    match value {
        Var(v) => match env.vars.get(v) {
            Some((_, val)) => (*val).clone(),
            None => panic!("Undefined variable {}", v),
        },
        Env(v, default_value) => match (env.env_vars.get(v), default_value) {
            (Some(val), _) => Str(val.to_string()),
            (None, Some(def)) => Str(def.to_string()),
            _ => panic!("Undefined env variable {}", v),
        },
        EnumVariant(s, t) => {
            // check existence
            let ok = if let Some(variants) = env.enums.get(s) {
                variants.iter().any(|v| v == t)
            } else {
                false
            };
            if !ok {
                panic!("Not found Enum {}::{}", s, t);
            }
            EnumVariant(s.to_string(), t.to_string())
        }
        _ => value.clone(),
    }
}

#[derive(Clone)]
struct Environ {
    structs: HashMap<String, Vec<(String, Typing, Option<Expr>)>>,
    enums: HashMap<String, Vec<String>>,
    vars: HashMap<String, (Typing, Value)>,
    env_vars: HashMap<String, String>,
}

impl Environ {
    pub fn new() -> Self {
        let structs = HashMap::new();
        let enums = HashMap::new();
        let env_vars = env::vars().collect();
        let vars = HashMap::new();
        Self {
            structs,
            enums,
            env_vars,
            vars,
        }
    }
}

#[cfg(test)]
mod test_eval_from_parse {
    use crate::eval::eval;
    use crate::json::JSON;
    use crate::parser::config::config;
    use combine::parser::Parser;

    macro_rules! assert_eval {
        ($code:expr, $json:expr) => {
            assert_eq!(eval(config().parse($code).unwrap().0), $json);
        };
    }

    #[test]
    fn test_numbers() {
        assert_eval!("-1", JSON::Int(-1));
        assert_eval!("-1 + 3", JSON::Int(2));
        assert_eval!("-1 / 2", JSON::Int(0));
        assert_eval!("1 + 2 * 3", JSON::Nat(7));
        assert_eval!("(1 + 2) * 3", JSON::Nat(9));
    }

    #[test]
    fn test_bools() {
        assert_eval!(
            "[true or true, true or false, false or true, false or false]",
            JSON::Array(vec![
                JSON::Bool(true),
                JSON::Bool(true),
                JSON::Bool(true),
                JSON::Bool(false)
            ])
        );
        assert_eval!(
            "[true and true, true and false, false and true, false and false]",
            JSON::Array(vec![
                JSON::Bool(true),
                JSON::Bool(false),
                JSON::Bool(false),
                JSON::Bool(false)
            ])
        );
        assert_eval!(
            "[true xor true, true xor false, false xor true, false xor false]",
            JSON::Array(vec![
                JSON::Bool(false),
                JSON::Bool(true),
                JSON::Bool(true),
                JSON::Bool(false)
            ])
        );
    }

    #[test]
    fn test_builtins() {
        assert_eval!("Some(1)", JSON::Nat(1));
        assert_eval!("Some(1 + 2)", JSON::Nat(3));
        assert_eval!("not(true)", JSON::Bool(false));
        assert_eval!("concat()", JSON::Array(vec![]));
        assert_eval!("concat([1])", JSON::Array(vec![JSON::Nat(1)]));
        assert_eval!(
            "concat([1], [2])",
            JSON::Array(vec![JSON::Nat(1), JSON::Nat(2)])
        );
        assert_eval!(
            "concat([1], [2], [3])",
            JSON::Array(vec![JSON::Nat(1), JSON::Nat(2), JSON::Nat(3)])
        );
        assert_eval!(
            "reverse([1, 2, 3])",
            JSON::Array(vec![JSON::Nat(3), JSON::Nat(2), JSON::Nat(1)])
        );
    }

    #[test]
    fn test_compare() {
        assert_eval!("let x = 2; x == 2", JSON::Bool(true));
        assert_eval!("let x = 2; 2 < x + 1", JSON::Bool(true));
    }

    #[test]
    fn test_optional() {
        assert_eval!(
            "[None, Some(1)]",
            JSON::Array(vec![JSON::Null, JSON::Nat(1)])
        );
    }

    #[test]
    fn test_fielded_apply() {
        assert_eval!(
            "struct P { x: Nat, y: Nat } P{ x = 1, y = 2 }",
            JSON::Dict(vec![
                ("x".to_string(), JSON::Nat(1)),
                ("y".to_string(), JSON::Nat(2)),
            ])
        );
        assert_eval!(
            "struct P { x: Nat, y: Nat } P{ y = 2, x = 1 }",
            JSON::Dict(vec![
                ("x".to_string(), JSON::Nat(1)),
                ("y".to_string(), JSON::Nat(2)),
            ])
        );
        assert_eval!(
            "struct P { x: Nat = 42, y: Nat } P{ x = 1, y = 2 }",
            JSON::Dict(vec![
                ("x".to_string(), JSON::Nat(1)),
                ("y".to_string(), JSON::Nat(2)),
            ])
        );
    }

    #[test]
    fn test_enum() {
        assert_eval!(
            "enum X { Zoo, Park } X::Park",
            JSON::Str("Park".to_string())
        );
        assert_eval!("enum X { Zoo, Park } X::Zoo", JSON::Str("Zoo".to_string()));
    }
}
