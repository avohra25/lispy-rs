//! lispy: a small Scheme-flavoured Lisp interpreter.
pub mod builtins;
pub mod eval;
pub mod parser;

pub use eval::{Env, Value};

/// A global environment preloaded with the builtins.
pub fn global_env() -> Env {
    let env = Env::new(None);
    builtins::install(&env);
    env
}

/// Parse and evaluate every form in `src`, returning the last value.
pub fn run(src: &str, env: &Env) -> Result<Value, String> {
    let mut last = Value::Bool(false);
    for form in parser::parse_all(src)? {
        last = eval::eval(&form, env)?;
    }
    Ok(last)
}
