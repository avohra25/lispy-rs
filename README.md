# lispy-rs

A small, dependency-free, Scheme-flavoured Lisp interpreter written in Rust.
It has lexical closures, proper tail calls, a multi-line REPL, and a script runner.

```scheme
lispy> (define (make-counter)
  ...    (let ((n 0)) (lambda () (set! n (+ n 1)) n)))
lispy> (define c (make-counter))
lispy> (c) (c)
2
```

## Running

```sh
cargo run                       # interactive REPL
cargo run -- examples/fib.scm   # run a script
cargo test                      # unit + integration tests
```

## Language

| Kind            | Supported                                                         |
|-----------------|-------------------------------------------------------------------|
| Atoms           | numbers (f64), `#t` / `#f`, strings, symbols, `'quote`            |
| Special forms   | `define` (incl. `(define (f x) ...)`), `lambda`, `if`, `let`, `begin`, `set!`, `and`, `or`, `quote` |
| Builtins        | `+ - * /`, `< > <= >= =`, `equal?`, `not`, `list car cdr cons null? length`, `display` |
| Comments        | `;` to end of line                                                 |

Only `#f` is false, as in Scheme.

## Design notes

- **Pipeline:** `parser.rs` tokenizes and reads source into an `Expr` AST.
  `eval.rs` walks that AST directly. `builtins.rs` registers primitives as plain
  `fn(&[Value]) -> Result<Value, String>` pointers.
- **Environments** are `Rc<RefCell<Scope>>` chains. Closures capture their
  defining `Env` by reference-counted pointer, so `set!` on a captured variable
  is visible to every closure that shares it.
- **Tail calls:** `eval` is a loop. `if`, `let`, `begin` and lambda application
  rebind the current `expr`/`env` and `continue` instead of recursing. A
  100,000-deep tail-recursive loop therefore runs in constant Rust stack (see
  `tests/eval.rs`). Non-tail recursion still uses the host stack.
- **Errors** are `Result<_, String>` all the way up. The REPL reports them and
  keeps going; the script runner exits non-zero.
- **The REPL** keeps reading lines until the buffer parses (balanced parens and
  closed strings), so multi-line definitions work.

Deliberate omissions to keep it small: no macros, no cons cells or dotted
pairs (lists are `Vec`s), and no garbage collection beyond `Rc`, so reference
cycles created through `set!` leak.

## License

MIT
