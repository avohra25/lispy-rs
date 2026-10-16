//! Primitive procedures installed in the global environment.
use crate::eval::{Env, Value};

fn nums(args: &[Value]) -> Result<Vec<f64>, String> {
    args.iter()
        .map(|a| match a {
            Value::Num(n) => Ok(*n),
            other => Err(format!("expected number, got {other}")),
        })
        .collect()
}

fn fold(args: &[Value], init: f64, f: fn(f64, f64) -> f64) -> Result<Value, String> {
    Ok(Value::Num(nums(args)?.into_iter().fold(init, f)))
}

fn add(a: &[Value]) -> Result<Value, String> { fold(a, 0.0, |x, y| x + y) }
fn mul(a: &[Value]) -> Result<Value, String> { fold(a, 1.0, |x, y| x * y) }

fn sub(a: &[Value]) -> Result<Value, String> {
    let n = nums(a)?;
    match n.split_first() {
        None => Err("-: needs arguments".into()),
        Some((x, [])) => Ok(Value::Num(-x)),
        Some((x, rest)) => Ok(Value::Num(rest.iter().fold(*x, |acc, y| acc - y))),
    }
}

fn div(a: &[Value]) -> Result<Value, String> {
    let n = nums(a)?;
    match n.split_first() {
        None => Err("/: needs arguments".into()),
        Some((x, rest)) => {
            if rest.contains(&0.0) { return Err("/: division by zero".into()); }
            Ok(Value::Num(rest.iter().fold(*x, |acc, y| acc / y)))
        }
    }
}

fn cmp(a: &[Value], f: fn(f64, f64) -> bool) -> Result<Value, String> {
    let n = nums(a)?;
    Ok(Value::Bool(n.windows(2).all(|w| f(w[0], w[1]))))
}

fn lt(a: &[Value]) -> Result<Value, String> { cmp(a, |x, y| x < y) }
fn gt(a: &[Value]) -> Result<Value, String> { cmp(a, |x, y| x > y) }
fn le(a: &[Value]) -> Result<Value, String> { cmp(a, |x, y| x <= y) }
fn ge(a: &[Value]) -> Result<Value, String> { cmp(a, |x, y| x >= y) }
fn num_eq(a: &[Value]) -> Result<Value, String> { cmp(a, |x, y| x == y) }

fn equal(a: &[Value]) -> Result<Value, String> {
    Ok(Value::Bool(a.windows(2).all(|w| w[0] == w[1])))
}

fn not(a: &[Value]) -> Result<Value, String> {
    Ok(Value::Bool(!a.first().map(Value::truthy).unwrap_or(false)))
}

pub fn install(env: &Env) {
    let table: &[(&'static str, crate::eval::Builtin)] = &[
        ("+", add), ("-", sub), ("*", mul), ("/", div),
        ("<", lt), (">", gt), ("<=", le), (">=", ge), ("=", num_eq),
        ("equal?", equal), ("not", not),
    ];
    for (name, f) in table {
        env.define(name, Value::Builtin(name, *f));
    }
}
