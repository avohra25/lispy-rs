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
cargo run -- examples/library.scm  # tour of cond, named let, apply and the list library
cargo test                      # unit + integration tests
```

## Language

| Kind            | Supported                                                         |
|-----------------|-------------------------------------------------------------------|
| Atoms           | numbers (f64), `#t` / `#f`, strings, symbols, `'quote`            |
| Special forms   | `define` (incl. `(define (f x) ...)`), `lambda`, `if`, `cond` (with `else`), `when`, `unless`, `let`, `let*`, named `let`, `begin`, `set!`, `and`, `or`, `quote` |
| Parameters      | fixed `(a b)`, rest `(a b . more)`, or all-args `args`: `(lambda args ...)`, `(define (f . xs) ...)` |
| Arithmetic      | `+ - * /`, `< > <= >= =`, `abs min max`, `quotient remainder modulo` (Scheme sign rules) |
| Predicates      | `equal? not`, `null? number? string? symbol? boolean? list? procedure?` |
| Lists           | `list car cdr cons length reverse append list-ref iota`, plus the prelude below |
| Strings         | `string-append string-length substring string=?`, `number->string string->number symbol->string string->symbol` |
| Control         | `apply` (tail-call safe), `error`, `display`                       |
| Comments        | `;` to end of line                                                 |

### Standard library prelude

Higher-order procedures that need to call closures are written in lispy itself
in [`src/prelude.scm`](src/prelude.scm) and loaded into every global
environment: `map` (any number of lists, stops at the shortest), `for-each`,
`filter`, `foldl`, `foldr`, `reduce`, `member` and `assoc`. They loop with
tail calls, so they don't grow the Rust stack on long lists, and a user
`define` can shadow any of them.

```scheme
lispy> (define (average x . more) (/ (apply + x more) (+ 1 (length more))))
lispy> (average 3 4 8)
5
lispy> (let loop ((i 0) (acc '())) (if (= i 3) (reverse acc) (loop (+ i 1) (cons (* i i) acc))))
(0 1 4)
lispy> (map + '(1 2 3) '(10 20 30))
(11 22 33)
```

Only `#f` is false, as in Scheme.

## Design notes

- **Pipeline:** `parser.rs` tokenizes and reads source into an `Expr` AST.
  `eval.rs` walks that AST directly. `builtins.rs` registers primitives as plain
  `fn(&[Value]) -> Result<Value, String>` pointers, and `prelude.scm` is
  evaluated into the global environment at startup.
- **Environments** are `Rc<RefCell<Scope>>` chains. Closures capture their
  defining `Env` by reference-counted pointer, so `set!` on a captured variable
  is visible to every closure that shares it.
- **Tail calls:** `eval` is a loop. `if`, `cond`, `when`/`unless`, every `let`
  form, `begin`, `apply` and lambda application
  rebind the current `expr`/`env` and `continue` instead of recursing. A
  100,000-deep tail-recursive loop therefore runs in constant Rust stack (see
  `tests/eval.rs`). Non-tail recursion still uses the host stack.
- **Errors** are `Result<_, String>` all the way up. The REPL reports them and
  keeps going; the script runner exits non-zero.
- **The REPL** keeps reading lines until the buffer parses (balanced parens and
  closed strings), so multi-line definitions work.

Deliberate omissions to keep it small: no macros, no cons cells or dotted
pairs (lists are `Vec`s, so `cons` and `cdr` copy and walking a list with
them is quadratic; prefer the native `reverse`, `append`, `list-ref` and
`iota` for big lists), and no garbage collection beyond `Rc`, so reference
cycles created through `set!` leak.

## License

MIT
