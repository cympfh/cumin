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
        Val(Nat(x)) => Nat(*x),
        Val(Int(x)) => Int(*x),
        Val(Float(x)) => Float(*x),
        Val(Str(s)) => Str(s.to_string()),
        Val(Var(v)) => match env.vars.get(v) {
            Some((_, val)) => (*val).clone(),
            None => panic!("Undefined variable {}", v),
        },
        Val(Env(v, default_value)) => match (env.env_vars.get(v), default_value) {
            (Some(val), _) => Str(val.to_string()),
            (None, Some(def)) => Str(def.to_string()),
            _ => panic!("Undefined env variable {}", v),
        },
        Val(Dict(items)) => Dict(items.clone()),
        Val(EnumVariant(s, t)) => {
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
        Val(Array(elements)) => Array(elements.to_vec()),
        Apply(f, args) => {
            if let Some(fields) = env.structs.get(f) {
                assert!(fields.len() == args.len());
                let n = fields.len();
                let items: Vec<(String, Value)> = (0..n)
                    .map(|i| {
                        let (name, _ty, _default) = &fields[i];
                        let val = eval_expr(&env, &args[i]);
                        (name.to_string(), val)
                    })
                    .collect();
                Dict(items)
            } else {
                panic!("Cannot resolve name {}", f)
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
mod test_eval {
    use crate::eval::*;
    use Expr::*;
    use Value::*;

    #[test]
    fn test() {
        let conf = Config(vec![], Val(Int(1)));
        assert_eq!(eval(conf), JSON::Int(1));

        let conf = Config(vec![], Add(Box::new(Val(Int(-1))), Box::new(Val(Nat(3)))));
        assert_eq!(eval(conf), JSON::Int(2));
    }

    #[test]
    fn test_enum() {
        let conf = Config(
            vec![Enum(
                "X".to_string(),
                vec!["Zoo".to_string(), "Park".to_string()],
            )],
            Val(EnumVariant("X".to_string(), "Park".to_string())),
        );
        assert_eq!(eval(conf), JSON::Str("Park".to_string()));
    }

    #[test]
    fn test_fielded_apply() {
        let conf = Config(
            vec![Struct(
                "P".to_string(),
                vec![
                    ("x".to_string(), Typing::Int, None),
                    ("y".to_string(), Typing::Int, None),
                ],
            )],
            FieledApply(
                "P".to_string(),
                vec![
                    ("y".to_string(), Val(Int(2))),
                    ("x".to_string(), Val(Int(1))),
                ],
            ),
        );
        assert_eq!(
            eval(conf),
            JSON::Dict(vec![
                ("x".to_string(), JSON::Int(1)),
                ("y".to_string(), JSON::Int(2)),
            ])
        );
    }

    #[test]
    fn test_fielded_apply_with_default() {
        let conf = Config(
            vec![Struct(
                "P".to_string(),
                vec![
                    ("x".to_string(), Typing::Int, Some(Val(Int(42)))),
                    ("y".to_string(), Typing::Int, None),
                ],
            )],
            FieledApply("P".to_string(), vec![("y".to_string(), Val(Int(2)))]),
        );
        assert_eq!(
            eval(conf),
            JSON::Dict(vec![
                ("x".to_string(), JSON::Int(42)),
                ("y".to_string(), JSON::Int(2)),
            ])
        );
    }
}
