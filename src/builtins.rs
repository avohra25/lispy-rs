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

macro_rules! predicate {
    ($name:ident, $pat:pat) => {
        fn $name(a: &[Value]) -> Result<Value, String> {
            Ok(Value::Bool(matches!(a.first(), Some($pat))))
        }
    };
}
predicate!(is_number, Value::Num(_));
predicate!(is_string, Value::Str(_));
predicate!(is_symbol, Value::Sym(_));
predicate!(is_boolean, Value::Bool(_));
predicate!(is_list, Value::List(_));

fn num1(name: &str, a: &[Value]) -> Result<f64, String> {
    match a {
        [Value::Num(n)] => Ok(*n),
        _ => Err(format!("{name}: expected one number")),
    }
}

fn abs(a: &[Value]) -> Result<Value, String> { Ok(Value::Num(num1("abs", a)?.abs())) }

fn extreme(name: &str, a: &[Value], pick: fn(f64, f64) -> f64) -> Result<Value, String> {
    let n = nums(a)?;
    let (first, rest) = n.split_first().ok_or(format!("{name}: needs arguments"))?;
    Ok(Value::Num(rest.iter().fold(*first, |acc, x| pick(acc, *x))))
}
fn min(a: &[Value]) -> Result<Value, String> { extreme("min", a, f64::min) }
fn max(a: &[Value]) -> Result<Value, String> { extreme("max", a, f64::max) }

/// Integer division helpers. `quotient`/`remainder` truncate toward zero;
/// `modulo` takes the sign of the divisor, as in Scheme.
fn int_div(name: &str, a: &[Value], f: fn(f64, f64) -> f64) -> Result<Value, String> {
    match nums(a)?.as_slice() {
        [_, d] if *d == 0.0 => Err(format!("{name}: division by zero")),
        [x, d] if x.fract() == 0.0 && d.fract() == 0.0 => Ok(Value::Num(f(*x, *d))),
        [_, _] => Err(format!("{name}: expected integers")),
        _ => Err(format!("{name}: expected 2 arguments")),
    }
}
fn quotient(a: &[Value]) -> Result<Value, String> { int_div("quotient", a, |x, d| (x / d).trunc()) }
fn remainder(a: &[Value]) -> Result<Value, String> { int_div("remainder", a, |x, d| x % d) }
fn modulo(a: &[Value]) -> Result<Value, String> { int_div("modulo", a, |x, d| ((x % d) + d) % d) }

fn strs<'a>(name: &str, a: &'a [Value]) -> Result<Vec<&'a str>, String> {
    a.iter()
        .map(|v| match v {
            Value::Str(s) => Ok(s.as_str()),
            other => Err(format!("{name}: expected string, got {other}")),
        })
        .collect()
}

fn string_append(a: &[Value]) -> Result<Value, String> {
    Ok(Value::Str(strs("string-append", a)?.concat()))
}

fn string_length(a: &[Value]) -> Result<Value, String> {
    match strs("string-length", a)?.as_slice() {
        [s] => Ok(Value::Num(s.chars().count() as f64)),
        _ => Err("string-length: expected one string".into()),
    }
}

/// (substring s start [end]) with character (not byte) indices.
fn substring(a: &[Value]) -> Result<Value, String> {
    let s = match a.first() {
        Some(Value::Str(s)) => s,
        _ => return Err("substring: expected a string".into()),
    };
    let chars: Vec<char> = s.chars().collect();
    let start = index("substring", a.get(1))?;
    let end = match a.get(2) { Some(_) => index("substring", a.get(2))?, None => chars.len() };
    if start > end || end > chars.len() {
        return Err(format!("substring: range {start}..{end} out of bounds for length {}", chars.len()));
    }
    Ok(Value::Str(chars[start..end].iter().collect()))
}

fn string_eq(a: &[Value]) -> Result<Value, String> {
    Ok(Value::Bool(strs("string=?", a)?.windows(2).all(|w| w[0] == w[1])))
}

fn number_to_string(a: &[Value]) -> Result<Value, String> {
    Ok(Value::Str(num1("number->string", a)?.to_string()))
}

/// Returns #f (rather than an error) when the string isn't a number.
fn string_to_number(a: &[Value]) -> Result<Value, String> {
    match strs("string->number", a)?.as_slice() {
        [s] => Ok(s.trim().parse::<f64>().map(Value::Num).unwrap_or(Value::Bool(false))),
        _ => Err("string->number: expected one string".into()),
    }
}

fn symbol_to_string(a: &[Value]) -> Result<Value, String> {
    match a {
        [Value::Sym(s)] => Ok(Value::Str(s.clone())),
        _ => Err("symbol->string: expected one symbol".into()),
    }
}

fn string_to_symbol(a: &[Value]) -> Result<Value, String> {
    match strs("string->symbol", a)?.as_slice() {
        [s] => Ok(Value::Sym(s.to_string())),
        _ => Err("string->symbol: expected one string".into()),
    }
}

/// (error "message" irritant ...) aborts evaluation with a user error.
fn error(a: &[Value]) -> Result<Value, String> {
    let parts: Vec<String> = a.iter().map(|v| match v {
        Value::Str(s) => s.clone(),
        other => other.to_string(),
    }).collect();
    Err(parts.join(" "))
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
        ("list-ref", list_ref), ("iota", iota),
        ("number?", is_number), ("string?", is_string), ("symbol?", is_symbol),
        ("boolean?", is_boolean), ("list?", is_list),
        ("abs", abs), ("min", min), ("max", max),
        ("quotient", quotient), ("remainder", remainder), ("modulo", modulo),
        ("string-append", string_append), ("string-length", string_length),
        ("substring", substring), ("string=?", string_eq),
        ("number->string", number_to_string), ("string->number", string_to_number),
        ("symbol->string", symbol_to_string), ("string->symbol", string_to_symbol),
        ("error", error), ("procedure?", is_procedure),
    ];
    for (name, f) in table {
        env.define(name, Value::Builtin(name, *f));
    }
}
