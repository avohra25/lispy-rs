use lispy_rs::{global_env, run};

fn ev(src: &str) -> String {
    run(src, &global_env()).map(|v| v.to_string()).unwrap_or_else(|e| format!("ERR {e}"))
}

#[test]
fn arithmetic() {
    assert_eq!(ev("(+ 1 2 (* 3 4))"), "15");
    assert_eq!(ev("(- 10 4 1)"), "5");
    assert_eq!(ev("(- 3)"), "-3");
    assert!(ev("(/ 1 0)").starts_with("ERR"));
}

#[test]
fn define_and_closures() {
    assert_eq!(ev("(define (make-adder n) (lambda (x) (+ x n))) ((make-adder 5) 10)"), "15");
    assert_eq!(ev("(define c 0) (define (inc!) (set! c (+ c 1)) c) (inc!) (inc!)"), "2");
}

#[test]
fn recursion_and_conditionals() {
    assert_eq!(ev("(define (fact n) (if (< n 2) 1 (* n (fact (- n 1))))) (fact 10)"), "3628800");
    assert_eq!(ev("(let ((x 2) (y 3)) (if (and (> y x) (not #f)) 'yes 'no))"), "yes");
}

#[test]
fn deep_tail_calls_do_not_overflow() {
    assert_eq!(ev("(define (loop n acc) (if (= n 0) acc (loop (- n 1) (+ acc 1)))) (loop 100000 0)"), "100000");
}

#[test]
fn errors_are_reported() {
    assert!(ev("(undefined-fn 1)").contains("unbound"));
    assert!(ev("(1 2)").contains("not a procedure"));
}
