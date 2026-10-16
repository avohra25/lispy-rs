use lispy_rs::{global_env, run};

fn ev(src: &str) -> String {
    run(src, &global_env()).map(|v| v.to_string()).unwrap_or_else(|e| format!("ERR {e}"))
}

#[test]
fn list_primitives() {
    assert_eq!(ev("(cons 1 (list 2 3))"), "(1 2 3)");
    assert_eq!(ev("(car (cdr '(a b c)))"), "b");
    assert_eq!(ev("(length '(1 2 3 4))"), "4");
    assert_eq!(ev("(null? '())"), "#t");
    assert!(ev("(car '())").starts_with("ERR"));
}

#[test]
fn user_defined_higher_order_functions() {
    let src = "(define (map f xs) (if (null? xs) '() (cons (f (car xs)) (map f (cdr xs)))))
               (define (filter p xs) (if (null? xs) '()
                 (if (p (car xs)) (cons (car xs) (filter p (cdr xs))) (filter p (cdr xs)))))
               (filter (lambda (x) (> x 4)) (map (lambda (x) (* x x)) '(1 2 3 4)))";
    assert_eq!(ev(src), "(9 16)");
}

#[test]
fn example_program_runs() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/fib.scm")).unwrap();
    assert!(run(&src, &global_env()).is_ok());
}
