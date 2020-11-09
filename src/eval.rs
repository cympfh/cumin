use crate::json::*;
use crate::parser::config::*;
use crate::parser::expr::*;
use crate::parser::statement::*;
use crate::parser::value::*;

use std::collections::HashMap;

use Statement::*;

pub fn eval(config: Config) -> JSON {
    let mut env = Environ::new();

    // collect struct
    for stmt in config.0.iter() {
        match stmt {
            Struct(name, fields) => {
                env.structs.insert(name.clone(), fields.clone());
            }
            _ => (),
        }
    }

    // collect enums
    for stmt in config.0.iter() {
        match stmt {
            Enum(name, variants) => {
                env.enums.insert(name.clone(), variants.clone());
            }
            _ => (),
        }
    }

    // collect let
    for stmt in config.0.iter() {
        match stmt {
            Let(id, ty, expr) => {
                let val = eval_expr(&env, expr);
                env.vars.insert(id.clone(), (ty.clone(), val));
            }
            _ => (),
        }
    }

    JSON::from_cumin(eval_expr(&env, &config.1))
}

fn eval_expr(env: &Environ, expr: &Expr) -> Value {
    use Expr::*;
    use Value::*;
    match expr {
        Val(Nat(x)) => Nat(*x),
        Val(Int(x)) => Int(*x),
        Val(Str(s)) => Str(s.to_string()),
        Val(Var(v)) => match env.vars.get(v) {
            Some((_, val)) => (*val).clone(),
            None => panic!("Undefined variable {}", v),
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
        Apply(f, args) => {
            if let Some(fields) = env.structs.get(f) {
                assert!(fields.len() == args.len());
                let n = fields.len();
                let items: Vec<(String, Value)> = (0..n)
                    .map(|i| {
                        let (name, _ty) = &fields[i];
                        let val = eval_expr(&env, &args[i]);
                        (name.to_string(), val)
                    })
                    .collect();
                Dict(items)
            } else {
                panic!("Cannot resolve name {}", f)
            }
        }
        Add(x, y) => {
            let a = eval_expr(&env, &x);
            let b = eval_expr(&env, &y);
            match (a, b) {
                (Nat(x), Nat(y)) => Nat(x + y),
                (Nat(x), Int(y)) => Int(x as i128 + y),
                (Int(x), Nat(y)) => Int(x + y as i128),
                (Int(x), Int(y)) => Int(x + y),
                (Str(x), Str(y)) => {
                    let mut z = x;
                    z.push_str(&y);
                    Str(z)
                }
                _ => panic!("Cant compute {:?} + {:?}", x, y),
            }
        }
    }
}

struct Environ {
    structs: HashMap<String, Vec<(String, String)>>,
    enums: HashMap<String, Vec<String>>,
    vars: HashMap<String, (String, Value)>,
}

impl Environ {
    fn new() -> Self {
        let structs = HashMap::new();
        let enums = HashMap::new();
        let vars = HashMap::new();
        Self {
            structs,
            enums,
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
}
