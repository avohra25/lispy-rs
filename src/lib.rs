//! lispy: a small Scheme-flavoured Lisp interpreter.
pub mod builtins;
pub mod eval;
pub mod parser;

pub use eval::{Env, Value};

/// Library procedures written in lispy itself (map, filter, fold, ...).
pub const PRELUDE: &str = include_str!("prelude.scm");

/// A global environment preloaded with the builtins and the prelude.
pub fn global_env() -> Env {
    let env = Env::new(None);
    builtins::install(&env);
    run(PRELUDE, &env).expect("prelude must evaluate cleanly");
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
