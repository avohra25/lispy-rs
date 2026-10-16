use std::io::{self, BufRead, Write};
use std::{env, fs, process};

use lispy_rs::{global_env, parser, run};

fn main() {
    let env = global_env();
    if let Some(path) = env::args().nth(1) {
        let src = fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("lispy: cannot read {path}: {e}");
            process::exit(2);
        });
        if let Err(e) = run(&src, &env) {
            eprintln!("error: {e}");
            process::exit(1);
        }
        return;
    }

    // REPL: accumulate lines until parens balance, then evaluate.
    let stdin = io::stdin();
    let mut buf = String::new();
    prompt(&buf);
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        buf.push_str(&line);
        buf.push('\n');
        if needs_more(&buf) {
            prompt(&buf);
            continue;
        }
        match run(&buf, &env) {
            Ok(v) => println!("{v}"),
            Err(e) => eprintln!("error: {e}"),
        }
        buf.clear();
        prompt(&buf);
    }
}

fn needs_more(src: &str) -> bool {
    matches!(parser::parse_all(src), Err(e) if e.contains("missing ')'") || e.contains("unterminated"))
}

fn prompt(buf: &str) {
    print!("{}", if buf.is_empty() { "lispy> " } else { "  ...  " });
    io::stdout().flush().ok();
}
