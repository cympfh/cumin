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

    // collect let
    for stmt in config.0.iter() {
        match stmt {
            Let(id, expr) => {
                let val = eval_expr(&env, expr);
                env.vars.insert(id.clone(), val);
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
            Some(val) => (*val).clone(),
            None => panic!("Undefined variable {}", v),
        },
        Add(x, y) => {
            let a = eval_expr(&env, &x);
            let b = eval_expr(&env, &y);
            match (a, b) {
                (Nat(x), Nat(y)) => Nat(x + y),
                (Nat(x), Int(y)) => Int(x as i128 + y),
                (Int(x), Nat(y)) => Int(x + y as i128),
                (Int(x), Int(y)) => Int(x + y),
                _ => panic!("Cant compute {:?} + {:?}", x, y),
            }
        }
    }
}

struct Environ {
    structs: HashMap<String, Vec<(String, String)>>,
    vars: HashMap<String, Value>,
}

impl Environ {
    fn new() -> Self {
        let structs = HashMap::new();
        let vars = HashMap::new();
        Self { structs, vars }
    }
}

#[cfg(test)]
mod test_eval {
    use crate::eval::*;
    use crate::json::*;
    use crate::parser::expr::*;
    use crate::parser::value::*;
    use Expr::*;
    use Value::*;

    #[test]
    fn test() {
        let conf = Config(vec![], Val(Int(1)));
        assert_eq!(eval(conf), JSON::Int(1));

        let conf = Config(vec![], Add(Box::new(Val(Int(-1))), Box::new(Val(Nat(3)))));
        assert_eq!(eval(conf), JSON::Int(2));
    }
}
