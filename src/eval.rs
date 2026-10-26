//! Evaluator: lexical environments, closures and proper tail calls.
use crate::parser::Expr;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

pub type Builtin = fn(&[Value]) -> Result<Value, String>;

#[derive(Clone)]
pub enum Value {
    Num(f64),
    Bool(bool),
    Str(String),
    Sym(String),
    List(Vec<Value>),
    Builtin(&'static str, Builtin),
    Lambda(Rc<Lambda>),
}

pub struct Lambda {
    params: Vec<String>,
    body: Vec<Expr>,
    env: Env,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Num(n) => write!(f, "{n}"),
            Value::Bool(b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
            Value::Str(s) => write!(f, "{s:?}"),
            Value::Sym(s) => write!(f, "{s}"),
            Value::List(xs) => {
                let p: Vec<String> = xs.iter().map(|x| x.to_string()).collect();
                write!(f, "({})", p.join(" "))
            }
            Value::Builtin(n, _) => write!(f, "#<builtin {n}>"),
            Value::Lambda(_) => write!(f, "#<lambda>"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, o: &Self) -> bool {
        use Value::*;
        match (self, o) {
            (Num(a), Num(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (Str(a), Str(b)) | (Sym(a), Sym(b)) => a == b,
            (List(a), List(b)) => a == b,
            (Lambda(a), Lambda(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Value {
    pub fn truthy(&self) -> bool {
        !matches!(self, Value::Bool(false))
    }
}

/// A chain of scopes; cheap to clone (shared via Rc).
#[derive(Clone)]
pub struct Env(Rc<RefCell<Scope>>);

struct Scope {
    vars: HashMap<String, Value>,
    parent: Option<Env>,
}

impl Env {
    pub fn new(parent: Option<Env>) -> Self {
        Env(Rc::new(RefCell::new(Scope { vars: HashMap::new(), parent })))
    }
    pub fn define(&self, k: &str, v: Value) {
        self.0.borrow_mut().vars.insert(k.to_string(), v);
    }
    pub fn get(&self, k: &str) -> Option<Value> {
        let s = self.0.borrow();
        s.vars.get(k).cloned().or_else(|| s.parent.as_ref()?.get(k))
    }
    fn set(&self, k: &str, v: Value) -> Result<(), String> {
        let mut s = self.0.borrow_mut();
        if let Some(slot) = s.vars.get_mut(k) {
            *slot = v;
            return Ok(());
        }
        match &s.parent {
            Some(p) => p.set(k, v),
            None => Err(format!("set!: unbound symbol '{k}'")),
        }
    }
}

fn quote(e: &Expr) -> Value {
    match e {
        Expr::Num(n) => Value::Num(*n),
        Expr::Bool(b) => Value::Bool(*b),
        Expr::Str(s) => Value::Str(s.clone()),
        Expr::Sym(s) => Value::Sym(s.clone()),
        Expr::List(xs) => Value::List(xs.iter().map(quote).collect()),
    }
}

fn sym(e: &Expr) -> Result<String, String> {
    match e {
        Expr::Sym(s) => Ok(s.clone()),
        other => Err(format!("expected symbol, got {other}")),
    }
}

/// Parse a `((name init) ...)` binding list.
fn bindings(form: &str, e: &Expr) -> Result<Vec<(String, Expr)>, String> {
    let bs = match e {
        Expr::List(bs) => bs,
        other => return Err(format!("{form}: bad bindings {other}")),
    };
    bs.iter()
        .map(|b| match b {
            Expr::List(kv) if kv.len() == 2 => Ok((sym(&kv[0])?, kv[1].clone())),
            other => Err(format!("{form}: bad binding {other}")),
        })
        .collect()
}

/// Evaluate all but the last expression of a body for effect and return the
/// last one, so the caller can continue with it as a tail call.
fn body_tail(body: &[Expr], env: &Env) -> Result<Expr, String> {
    let (last, init) = body.split_last().ok_or("empty body")?;
    for e in init { eval(e, env)?; }
    Ok(last.clone())
}

fn arity(form: &str, args: &[Expr], n: usize) -> Result<(), String> {
    if args.len() < n {
        return Err(format!("{form}: expected at least {n} arguments"));
    }
    Ok(())
}

/// Evaluate `expr` in `env`. Tail positions loop instead of recursing,
/// so deeply tail-recursive programs run in constant Rust stack.
pub fn eval(expr: &Expr, env: &Env) -> Result<Value, String> {
    let mut expr = expr.clone();
    let mut env = env.clone();
    loop {
        let list = match &expr {
            Expr::Num(n) => return Ok(Value::Num(*n)),
            Expr::Bool(b) => return Ok(Value::Bool(*b)),
            Expr::Str(s) => return Ok(Value::Str(s.clone())),
            Expr::Sym(s) => return env.get(s).ok_or(format!("unbound symbol '{s}'")),
            Expr::List(xs) if xs.is_empty() => return Ok(Value::List(vec![])),
            Expr::List(xs) => xs.clone(),
        };
        let (head, args) = (&list[0], &list[1..]);
        if let Expr::Sym(s) = head {
            match s.as_str() {
                "quote" => { arity("quote", args, 1)?; return Ok(quote(&args[0])); }
                "if" => {
                    arity("if", args, 2)?;
                    expr = if eval(&args[0], &env)?.truthy() {
                        args[1].clone()
                    } else if let Some(e) = args.get(2) {
                        e.clone()
                    } else {
                        return Ok(Value::Bool(false));
                    };
                    continue;
                }
                "define" => {
                    arity("define", args, 2)?;
                    match &args[0] {
                        // (define (f x y) body...) sugar
                        Expr::List(sig) if !sig.is_empty() => {
                            let name = sym(&sig[0])?;
                            let params = sig[1..].iter().map(sym).collect::<Result<_, _>>()?;
                            let l = Lambda { params, body: args[1..].to_vec(), env: env.clone() };
                            env.define(&name, Value::Lambda(Rc::new(l)));
                        }
                        target => {
                            let v = eval(&args[1], &env)?;
                            env.define(&sym(target)?, v);
                        }
                    }
                    return Ok(Value::Bool(true));
                }
                "set!" => {
                    arity("set!", args, 2)?;
                    let v = eval(&args[1], &env)?;
                    env.set(&sym(&args[0])?, v)?;
                    return Ok(Value::Bool(true));
                }
                "lambda" => {
                    arity("lambda", args, 2)?;
                    let params = match &args[0] {
                        Expr::List(ps) => ps.iter().map(sym).collect::<Result<_, _>>()?,
                        other => return Err(format!("lambda: bad parameter list {other}")),
                    };
                    let l = Lambda { params, body: args[1..].to_vec(), env: env.clone() };
                    return Ok(Value::Lambda(Rc::new(l)));
                }
                "let" if matches!(args.first(), Some(Expr::Sym(_))) => {
                    // Named let: (let loop ((i 0)) body...) binds `loop` to a
                    // procedure over the binding names, then calls it. Calls to
                    // `loop` in tail position reuse the evaluator loop.
                    arity("let", args, 3)?;
                    let name = sym(&args[0])?;
                    let pairs = bindings("let", &args[1])?;
                    let mut vals = Vec::with_capacity(pairs.len());
                    for (_, init) in &pairs { vals.push(eval(init, &env)?); }
                    let scope = Env::new(Some(env.clone()));
                    let params = pairs.into_iter().map(|(k, _)| k).collect();
                    let l = Rc::new(Lambda { params, body: args[2..].to_vec(), env: scope.clone() });
                    scope.define(&name, Value::Lambda(l.clone()));
                    let new = Env::new(Some(scope));
                    for (p, v) in l.params.iter().zip(vals) { new.define(p, v); }
                    expr = body_tail(&l.body, &new)?;
                    env = new;
                    continue;
                }
                "let" | "let*" => {
                    // `let` evaluates every init in the outer scope; `let*`
                    // evaluates each one in a scope that sees the earlier names.
                    arity(s, args, 2)?;
                    let sequential = s == "let*";
                    let new = Env::new(Some(env.clone()));
                    for (k, init) in bindings(s, &args[0])? {
                        let v = eval(&init, if sequential { &new } else { &env })?;
                        new.define(&k, v);
                    }
                    env = new;
                    expr = body_tail(&args[1..], &env)?;
                    continue;
                }
                "cond" => {
                    // (cond (test body...) ... (else body...)). A clause with no
                    // body yields the test value. The chosen body is a tail position.
                    let mut chosen = None;
                    for (i, clause) in args.iter().enumerate() {
                        let parts = match clause {
                            Expr::List(ps) if !ps.is_empty() => ps,
                            other => return Err(format!("cond: bad clause {other}")),
                        };
                        if matches!(&parts[0], Expr::Sym(e) if e == "else") {
                            if i != args.len() - 1 { return Err("cond: else must be the last clause".into()); }
                            chosen = Some(Err(parts[1..].to_vec()));
                            break;
                        }
                        let test = eval(&parts[0], &env)?;
                        if test.truthy() {
                            chosen = Some(if parts.len() == 1 { Ok(test) } else { Err(parts[1..].to_vec()) });
                            break;
                        }
                    }
                    match chosen {
                        None => return Ok(Value::Bool(false)),
                        Some(Ok(v)) => return Ok(v),
                        Some(Err(body)) if body.is_empty() => return Ok(Value::Bool(false)),
                        Some(Err(body)) => { expr = body_tail(&body, &env)?; continue; }
                    }
                }
                "when" | "unless" => {
                    arity(s, args, 2)?;
                    let t = eval(&args[0], &env)?.truthy();
                    if t != (s == "when") { return Ok(Value::Bool(false)); }
                    expr = body_tail(&args[1..], &env)?;
                    continue;
                }
                "begin" => {
                    if args.is_empty() { return Ok(Value::Bool(false)); }
                    for e in &args[..args.len() - 1] { eval(e, &env)?; }
                    expr = args[args.len() - 1].clone();
                    continue;
                }
                "and" => {
                    let mut last = Value::Bool(true);
                    for e in args { last = eval(e, &env)?; if !last.truthy() { break; } }
                    return Ok(last);
                }
                "or" => {
                    for e in args { let v = eval(e, &env)?; if v.truthy() { return Ok(v); } }
                    return Ok(Value::Bool(false));
                }
                _ => {}
            }
        }
        let f = eval(head, &env)?;
        let vals = args.iter().map(|a| eval(a, &env)).collect::<Result<Vec<_>, _>>()?;
        match f {
            Value::Builtin(_, func) => return func(&vals),
            Value::Lambda(l) => {
                if l.params.len() != vals.len() {
                    return Err(format!("expected {} args, got {}", l.params.len(), vals.len()));
                }
                let new = Env::new(Some(l.env.clone()));
                for (p, v) in l.params.iter().zip(vals) { new.define(p, v); }
                for e in &l.body[..l.body.len() - 1] { eval(e, &new)?; }
                expr = l.body[l.body.len() - 1].clone();
                env = new;
            }
            other => return Err(format!("not a procedure: {other}")),
        }
    }
}
