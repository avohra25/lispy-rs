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

fn list(a: &[Value]) -> Result<Value, String> { Ok(Value::List(a.to_vec())) }

fn as_list<'a>(name: &str, v: Option<&'a Value>) -> Result<&'a [Value], String> {
    match v {
        Some(Value::List(xs)) => Ok(xs),
        other => Err(format!("{name}: expected list, got {}", other.map(|v| v.to_string()).unwrap_or_default())),
    }
}

fn car(a: &[Value]) -> Result<Value, String> {
    as_list("car", a.first())?.first().cloned().ok_or("car: empty list".into())
}

fn cdr(a: &[Value]) -> Result<Value, String> {
    let xs = as_list("cdr", a.first())?;
    if xs.is_empty() { return Err("cdr: empty list".into()); }
    Ok(Value::List(xs[1..].to_vec()))
}

fn cons(a: &[Value]) -> Result<Value, String> {
    let head = a.first().cloned().ok_or("cons: needs 2 arguments")?;
    let mut out = vec![head];
    out.extend_from_slice(as_list("cons", a.get(1))?);
    Ok(Value::List(out))
}

fn is_null(a: &[Value]) -> Result<Value, String> {
    Ok(Value::Bool(matches!(a.first(), Some(Value::List(xs)) if xs.is_empty())))
}

fn length(a: &[Value]) -> Result<Value, String> {
    Ok(Value::Num(as_list("length", a.first())?.len() as f64))
}

fn display(a: &[Value]) -> Result<Value, String> {
    let parts: Vec<String> = a.iter().map(|v| match v {
        Value::Str(s) => s.clone(),
        other => other.to_string(),
    }).collect();
    println!("{}", parts.join(" "));
    Ok(Value::Bool(true))
}

fn reverse(a: &[Value]) -> Result<Value, String> {
    let mut xs = as_list("reverse", a.first())?.to_vec();
    xs.reverse();
    Ok(Value::List(xs))
}

fn append(a: &[Value]) -> Result<Value, String> {
    let mut out = Vec::new();
    for v in a { out.extend_from_slice(as_list("append", Some(v))?); }
    Ok(Value::List(out))
}

fn index(name: &str, v: Option<&Value>) -> Result<usize, String> {
    match v {
        Some(Value::Num(n)) if *n >= 0.0 && n.fract() == 0.0 => Ok(*n as usize),
        other => Err(format!("{name}: expected a non-negative integer, got {}",
            other.map(|v| v.to_string()).unwrap_or_default())),
    }
}

fn list_ref(a: &[Value]) -> Result<Value, String> {
    let xs = as_list("list-ref", a.first())?;
    let k = index("list-ref", a.get(1))?;
    xs.get(k).cloned().ok_or(format!("list-ref: index {k} out of range for length {}", xs.len()))
}

/// (iota n [start [step]]) => n numbers counting from start (default 0) by step (default 1).
fn iota(a: &[Value]) -> Result<Value, String> {
    let n = index("iota", a.first())?;
    let rest = nums(&a[1.min(a.len())..])?;
    let (start, step) = (rest.first().copied().unwrap_or(0.0), rest.get(1).copied().unwrap_or(1.0));
    Ok(Value::List((0..n).map(|i| Value::Num(start + step * i as f64)).collect()))
}

fn apply_placeholder(_: &[Value]) -> Result<Value, String> {
    // The evaluator intercepts calls to `apply` before reaching this.
    Err("apply: internal error".into())
}

fn is_procedure(a: &[Value]) -> Result<Value, String> {
    Ok(Value::Bool(matches!(a.first(), Some(Value::Builtin(..) | Value::Lambda(_)))))
}

pub fn install(env: &Env) {
    let table: &[(&'static str, crate::eval::Builtin)] = &[
        ("+", add), ("-", sub), ("*", mul), ("/", div),
        ("<", lt), (">", gt), ("<=", le), (">=", ge), ("=", num_eq),
        ("equal?", equal), ("not", not),
        ("list", list), ("car", car), ("cdr", cdr), ("cons", cons),
        ("null?", is_null), ("length", length), ("display", display),
        ("apply", apply_placeholder), ("reverse", reverse), ("append", append),
        ("list-ref", list_ref), ("iota", iota), ("procedure?", is_procedure),
    ];
    for (name, f) in table {
        env.define(name, Value::Builtin(name, *f));
    }
}
