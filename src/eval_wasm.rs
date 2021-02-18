use crate::builtins;
use crate::json::*;
use crate::parser::{cumin::*, expr::*, statement::*, typing::*, value::*};
use crate::{assert_args_eq, assert_args_leq, bail_type_error};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use Statement::*;

pub fn eval_wasm(cumin: Cumin) -> Result<JSON> {
    let mut env = Environ::new();
    let val = eval_conf(&mut env, &cumin)?;
    Ok(JSON::from_cumin(val))
}

fn eval_conf(env: &mut Environ, conf: &Cumin) -> Result<Value> {
    // Hoisting types
    for stmt in conf.0.iter() {
        match stmt {
            Type(name, types) => {
                let _ = env
                    .types
                    .insert(name.to_string(), types.iter().cloned().collect());
            }
            _ => (),
        }
    }

    // Hoisting struct
    for stmt in conf.0.iter() {
        match stmt {
            Struct(name, fields) => {
                env.structs.insert(name.clone(), fields.clone());
            }
            _ => (),
        }
    }

    // Hoisting enums
    for stmt in conf.0.iter() {
        match stmt {
            Enum(name, variants) => {
                env.enums.insert(name.clone(), variants.clone());
            }
            _ => (),
        }
    }

    // Evaluating let, functions, load-modules
    for stmt in conf.0.iter() {
        match stmt {
            Fun(name, args, body) => {
                env.funs
                    .insert(name.clone(), (env.clone(), args.to_vec(), body.clone()));
            }
            Import(_) => panic!("Cannot use import in WASM ;_;"),
            Let(id, typ, expr) => {
                let val = eval_expr(&env, expr)?.cast(typ)?;
                env.vars.insert(id.clone(), (typ.clone(), val));
            }
            _ => (),
        }
    }

    eval_expr(&env, &conf.1)
}

fn eval_expr(env: &Environ, expr: &Expr) -> Result<Value> {
    use Expr::*;
    use Value::*;
    match expr {
        Val(value) => eval_value(&env, value),
        Var(v) => match env.vars.get(v) {
            Some((_, val)) => Ok((*val).clone()),
            None => bail!("Undefined variable `{}`.", v),
        },
        Apply(fname, args) => {
            let values: Vec<Value> = args
                .iter()
                .map(|x| eval_expr(&env, &x))
                .collect::<Result<_>>()?;
            match fname.as_str() {
                "Some" => {
                    assert_args_eq!("Some", values.len(), 1);
                    let val = values[0].clone();
                    let typ = val.type_of();
                    Ok(Optional(typ, Box::new(Some(val))))
                }
                "not" => {
                    assert_args_eq!("not", values.len(), 1);
                    let e = Not(Box::new(Val(values[0].clone())));
                    eval_expr(&env, &e)
                }
                "concat" => builtins::concat(&values),
                "reverse" => {
                    assert_args_eq!("reverse", values.len(), 1);
                    builtins::reverse(&values[0])
                }
                // Struct Apply
                _ if env.structs.contains_key(fname) => {
                    let fields = env.structs.get(fname).unwrap();
                    assert_args_leq!(fname, values.len(), fields.len());
                    let n = values.len();
                    let mut items = vec![];
                    for ((name, typ, _default), value) in fields[..n].iter().zip(values.iter()) {
                        let val = value.cast(typ)?;
                        items.push((name.to_string(), val.clone()));
                    }
                    for (name, typ, default) in fields[n..].iter() {
                        if let Some(e) = default {
                            let value = eval_expr(&env, e)?;
                            let val = value.cast(typ)?;
                            items.push((name.to_string(), val.clone()));
                        } else {
                            bail!("Not supplied Field `{}` for Struct `{}`", name, fname);
                        }
                    }
                    Ok(Dict(Some(fname.to_string()), items))
                }
                // Type Apply
                _ if env.types.contains_key(fname) => {
                    assert_args_eq!(fname, values.len(), 1);
                    let value = values[0].clone();
                    let typ = values[0].type_of();
                    // up-cast
                    for variant_typ in env.types.get(fname).unwrap().iter() {
                        if let Ok(val) = value.cast(variant_typ) {
                            return Ok(Wrapped(
                                Typing::UserTyping(fname.to_string()),
                                Box::new(val),
                            ));
                        } else {
                            continue;
                        }
                    }
                    bail!("Cannot up-cast `{:?}` <: `{}`.", typ, fname.to_string());
                }
                // Function Apply
                _ if env.funs.contains_key(fname) => {
                    let (env_inner, args, body) = env.funs.get(fname).unwrap();
                    assert_args_leq!(fname, values.len(), args.len());
                    let mut env_inner = env_inner.clone();
                    let n = values.len();
                    for ((name, typ, _default), value) in args[..n].iter().zip(values.iter()) {
                        let val = value.cast(typ)?;
                        env_inner.vars.insert(name.to_string(), (typ.clone(), val));
                    }
                    for (name, typ, default) in args[n..].iter() {
                        if let Some(e) = default {
                            let val = eval_expr(&env, &e)?;
                            let val = val.cast(typ)?;
                            env_inner.vars.insert(name.to_string(), (typ.clone(), val));
                        } else {
                            bail!("Not supplied Arg `{}` for Function `{}`.", name, fname);
                        }
                    }
                    eval_expr(&mut env_inner, body)
                }
                _ => bail!("Cannot resolve name `{}`.", fname),
            }
        }
        FieledApply(fname, items) => {
            if let Some(fields) = env.structs.get(fname) {
                let args: HashMap<String, Expr> = items.iter().cloned().collect();
                assert_args_leq!(fname, args.len(), fields.len());
                let mut values: Vec<(String, Value)> = vec![];
                for (name, typ, default) in fields {
                    if let Some(arg) = args.get(&name.to_string()) {
                        let val = eval_expr(&env, &arg)?.cast(&typ)?;
                        values.push((name.to_string(), val));
                    } else {
                        if let Some(e) = default {
                            let val = eval_expr(&env, &e)?.cast(&typ)?;
                            values.push((name.to_string(), val));
                        } else {
                            bail!("Not supplied Field `{}` for Struct `{}`", name, fname);
                        }
                    }
                }
                {
                    // check undefined fields given
                    let defined_fields: HashSet<String> =
                        fields.iter().map(|(name, _, _)| name).cloned().collect();
                    for (name, _) in args.iter() {
                        if !defined_fields.contains(name) {
                            bail!("Undefined Field `{}` supplied for Struct `{}`", name, fname)
                        }
                    }
                }
                Ok(Dict(Some(fname.to_string()), values))
            } else if let Some((env_inner, fields, body)) = env.funs.get(fname) {
                let args: HashMap<String, Expr> = items.iter().cloned().collect();
                assert_args_leq!(fname, args.len(), fields.len());
                let mut env_inner = env_inner.clone();
                for (name, typ, default) in fields.iter() {
                    if let Some(arg) = args.get(&name.to_string()) {
                        let val = eval_expr(&env, &arg)?.cast(&typ)?;
                        env_inner
                            .vars
                            .insert(name.to_string(), (typ.clone(), val.clone()));
                    } else {
                        if let Some(e) = default {
                            let val = eval_expr(&env, &e)?.cast(&typ)?;
                            env_inner.vars.insert(name.to_string(), (typ.clone(), val));
                        } else {
                            bail!("Not supplied Arg `{}` for Function `{}`.", name, fname);
                        }
                    }
                }
                {
                    // check undefined args given
                    let defined_fields: HashSet<String> =
                        fields.iter().map(|(name, _, _)| name).cloned().collect();
                    for (name, _) in args.iter() {
                        if !defined_fields.contains(name) {
                            bail!("Undefined Arg `{}` supplied for Function `{}`", name, fname)
                        }
                    }
                }
                eval_expr(&mut env_inner, body)
            } else {
                bail!("Cannot resolve name `{}`", fname)
            }
        }
        AnonymousStruct(items) => {
            let mut values = vec![];
            for (name, typ, val) in items.iter() {
                let val = eval_expr(&env, &val)?.cast(typ)?;
                values.push((name.to_string(), val.clone()));
            }
            Ok(Dict(None, values))
        }
        Concat(x, y) => {
            let a = eval_expr(&env, &x)?;
            let b = eval_expr(&env, &y)?;
            builtins::concat(&vec![a, b])
        }
        Add(x, y) => {
            let a = eval_expr(&env, &x)?;
            let b = eval_expr(&env, &y)?;
            let ret = match (a, b) {
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
                (x, y) => bail_type_error!(compute x "+" y),
            };
            Ok(ret)
        }
        Sub(x, y) => {
            let a = eval_expr(&env, &x)?;
            let b = eval_expr(&env, &y)?;
            let ret = match (a, b) {
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
                (x, y) => bail_type_error!(compute x "-" y),
            };
            Ok(ret)
        }
        Mul(x, y) => {
            let a = eval_expr(&env, &x)?;
            let b = eval_expr(&env, &y)?;
            let ret = match (a, b) {
                (Nat(x), Nat(y)) => Nat(x * y),
                (Nat(x), Int(y)) => Int(x as i128 * y),
                (Nat(x), Float(y)) => Float(x as f64 * y),
                (Int(x), Nat(y)) => Int(x * y as i128),
                (Int(x), Int(y)) => Int(x * y),
                (Int(x), Float(y)) => Float(x as f64 * y),
                (Float(x), Nat(y)) => Float(x * y as f64),
                (Float(x), Int(y)) => Float(x * y as f64),
                (Float(x), Float(y)) => Float(x * y),
                (x, y) => bail_type_error!(compute x "*" y),
            };
            Ok(ret)
        }
        Div(x, y) => {
            let a = eval_expr(&env, &x)?;
            let b = eval_expr(&env, &y)?;
            let ret = match (a, b) {
                (Nat(x), Nat(y)) => Nat(x / y),
                (Nat(x), Int(y)) => Int(x as i128 / y),
                (Nat(x), Float(y)) => Float(x as f64 / y),
                (Int(x), Nat(y)) => Int(x / y as i128),
                (Int(x), Int(y)) => Int(x / y),
                (Int(x), Float(y)) => Float(x as f64 / y),
                (Float(x), Nat(y)) => Float(x / y as f64),
                (Float(x), Int(y)) => Float(x / y as f64),
                (Float(x), Float(y)) => Float(x / y),
                (x, y) => bail_type_error!(compute x "/" y),
            };
            Ok(ret)
        }
        Pow(x, y) => {
            let a = eval_expr(&env, &x)?;
            let b = eval_expr(&env, &y)?;
            let ret = match (a, b) {
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
                (x, y) => bail_type_error!(compute x "**" y),
            };
            Ok(ret)
        }
        Minus(x) => {
            let a = eval_expr(&env, &x)?;
            let ret = match a {
                Nat(x) => Int(-(x as i128)),
                Int(x) => Int(-x),
                Float(x) => Float(-x),
                x => bail_type_error!(compute "-" x),
            };
            Ok(ret)
        }
        And(x, y) => {
            let a = eval_expr(&env, &x)?;
            let b = eval_expr(&env, &y)?;
            let ret = match (a, b) {
                (Bool(x), Bool(y)) => Bool(x && y),
                (x, y) => bail_type_error!(compute x "and" y),
            };
            Ok(ret)
        }
        Or(x, y) => {
            let a = eval_expr(&env, &x)?;
            let b = eval_expr(&env, &y)?;
            let ret = match (a, b) {
                (Bool(x), Bool(y)) => Bool(x || y),
                (x, y) => bail_type_error!(compute x "or" y),
            };
            Ok(ret)
        }
        Xor(x, y) => {
            let a = eval_expr(&env, &x)?;
            let b = eval_expr(&env, &y)?;
            let ret = match (a, b) {
                (Bool(x), Bool(y)) => Bool(x ^ y),
                (x, y) => bail_type_error!(compute x "xor" y),
            };
            Ok(ret)
        }
        Not(x) => {
            let a = eval_expr(&env, &x)?;
            let ret = match a {
                Bool(x) => Bool(!x),
                x => bail_type_error!(compute "not" x),
            };
            Ok(ret)
        }
        Equal(x, y) => {
            let a = eval_expr(&env, &x)?;
            let b = eval_expr(&env, &y)?;
            let ret = match (a, b) {
                (Nat(x), Nat(y)) => Bool(x == y),
                (Nat(x), Int(y)) => Bool(x as i128 == y),
                (Int(x), Nat(y)) => Bool(x == y as i128),
                (Int(x), Int(y)) => Bool(x == y),
                (Float(x), Float(y)) => Bool(x == y),
                (Bool(x), Bool(y)) => Bool(x == y),
                (x, y) => bail_type_error!(compute x "==" y),
            };
            Ok(ret)
        }
        Less(x, y) => {
            let a = eval_expr(&env, &x)?;
            let b = eval_expr(&env, &y)?;
            let ret = match (a, b) {
                (Nat(x), Nat(y)) => Bool(x < y),
                (Nat(x), Int(y)) => Bool((x as i128) < y),
                (Int(x), Nat(y)) => Bool(x < y as i128),
                (Int(x), Int(y)) => Bool(x < y),
                (Float(x), Float(y)) => Bool(x < y),
                (x, y) => bail_type_error!(compute x "<" y),
            };
            Ok(ret)
        }
        Arrayed(elements) => {
            let elements: Vec<Value> = elements
                .iter()
                .map(|e| eval_expr(&env, &e))
                .collect::<Result<_>>()?;
            // type-unification
            let mut element_type = Typing::Any;
            for elem in elements.iter() {
                if let Some(unified) = Typing::unify(&element_type, &elem.type_of()) {
                    element_type = unified;
                } else {
                    bail!("Cannot infer type of Array({:?}); Hint: Array cannot contain values with different types.", &elements);
                }
            }
            let mut values = vec![];
            for elem in elements.iter() {
                let val = elem.cast(&element_type)?;
                values.push(val);
            }
            Ok(Array(element_type, values))
        }
        Blocked(conf_inner) => {
            let mut env_inner: Environ = (*env).clone();
            eval_conf(&mut env_inner, &conf_inner)
        }
        AsCast(expr, typ) => {
            let val = eval_expr(&env, &expr)?;
            val.coerce(typ)
        }
    }
}

fn eval_value(env: &Environ, value: &Value) -> Result<Value> {
    use Value::*;
    match value {
        Env(_, _) => bail!("Cannot use $VAR in WASM ;_;"),
        EnumVariant(s, t) => {
            // check existence
            if let Some(variants) = env.enums.get(s) {
                if variants.iter().any(|v| v == t) {
                    Ok(EnumVariant(s.to_string(), t.to_string()))
                } else {
                    bail!("Enum {} doesnt have {}", s, t)
                }
            } else {
                bail!("Not found Enum {}", s)
            }
        }
        _ => Ok(value.clone()),
    }
}

#[derive(Clone)]
struct Environ {
    types: HashMap<String, Vec<Typing>>,
    structs: HashMap<String, Vec<(String, Typing, Option<Expr>)>>,
    enums: HashMap<String, Vec<String>>,
    vars: HashMap<String, (Typing, Value)>,
    funs: HashMap<String, (Environ, Vec<(String, Typing, Option<Expr>)>, Expr)>,
}

impl Environ {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            vars: HashMap::new(),
            funs: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod test_eval_from_parse {
    use crate::eval::eval;
    use crate::json::JSON;
    use crate::parser::cumin::cumin;

    macro_rules! assert_eval {
        ($code:expr, $json:expr) => {
            assert_eq!(eval(cumin($code).unwrap().1, None).unwrap(), $json);
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
    fn test_dict() {
        assert_eval!("{{}}", JSON::Dict(vec![]));
        assert_eval!(
            "{{ x = 1, y = 2, }}",
            JSON::Dict(vec![
                ("x".to_string(), JSON::Nat(1)),
                ("y".to_string(), JSON::Nat(2)),
            ])
        );
        assert_eval!(
            "{{ x: Float = 1, y = 2, }}",
            JSON::Dict(vec![
                ("x".to_string(), JSON::Float(1.0)),
                ("y".to_string(), JSON::Nat(2)),
            ])
        );
    }

    #[test]
    fn test_array() {
        use JSON::*;
        assert_eval!("[1, 2, 3]", Array(vec![Nat(1), Nat(2), Nat(3)]));
        assert_eval!(
            "[1, 2, 3, -1]",
            Array(vec![Int(1), Int(2), Int(3), Int(-1)])
        );
        assert_eval!("[None]", Array(vec![Null]));
        assert_eval!("[Some(1), Some(-1)]", Array(vec![Int(1), Int(-1)]));
        assert_eval!("[1, 2] ++ [] ++ [3]", Array(vec![Nat(1), Nat(2), Nat(3)]));
        assert_eval!(
            "reverse([2, 1]) ++ [] ++ [3]",
            Array(vec![Nat(1), Nat(2), Nat(3)])
        );
    }

    #[test]
    fn test_struct() {
        assert_eval!(
            "struct P { x: Nat, y: Nat } P(1, 2)",
            JSON::Dict(vec![
                ("x".to_string(), JSON::Nat(1)),
                ("y".to_string(), JSON::Nat(2)),
            ])
        );
        assert_eval!(
            "struct P { x: Nat, y: Nat = 100 } P(1, 2)",
            JSON::Dict(vec![
                ("x".to_string(), JSON::Nat(1)),
                ("y".to_string(), JSON::Nat(2)),
            ])
        );
        assert_eval!(
            "struct P { x: Nat, y: Nat = 100 } P(1)",
            JSON::Dict(vec![
                ("x".to_string(), JSON::Nat(1)),
                ("y".to_string(), JSON::Nat(100)),
            ])
        );
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
        assert_eval!(
            "struct P { x: Nat = 42, y: Nat } P{ y = 2 }",
            JSON::Dict(vec![
                ("x".to_string(), JSON::Nat(42)),
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

    #[test]
    fn test_type() {
        assert_eval!(
            "type T = Int | String; [T(1), T(\"hoge\")]",
            JSON::Array(vec![JSON::Int(1), JSON::Str("hoge".to_string())])
        );
    }

    macro_rules! assert_cannot_eval {
        ($code:expr) => {
            assert!(eval(cumin($code).unwrap().1, None).is_err());
        };
    }

    #[test]
    fn test_type_error() {
        assert_cannot_eval!("let n: Nat = -1; n");
        assert_cannot_eval!("let xs: Array<Nat> = [-1]; xs");
        assert_cannot_eval!("let xs: Option<Nat> = Some(-1); xs");
    }

    #[test]
    fn test_fn() {
        assert_eval!("fn f() = 1; f()", JSON::Nat(1));
        assert_eval!("let f() = 1; f()", JSON::Nat(1));
        assert_eval!("fn f(x: Int) = x - 1; f(3)", JSON::Int(2));
        assert_eval!("fn f(x: Int, y: Int = 0) = x - y; f(3)", JSON::Int(3));
        assert_eval!("fn f(x: Int, y: Int = 0) = x - y; f{x=3}", JSON::Int(3));
        assert_eval!(
            "fn f(x: Int, y: Int = 0) = x - y; f{y=2, x=3}",
            JSON::Int(1)
        );
        assert_eval!(
            "fn f(x: Int) = {{ x = x - 1 }}; f(3)",
            JSON::Dict(vec![("x".to_string(), JSON::Int(2))])
        );
        assert_eval!(
            "let f(x: Int) = x; fn g (x: Int) = f(x); g(2)",
            JSON::Int(2)
        );
    }
}
