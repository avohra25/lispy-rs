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

#[test]
fn cond_when_unless() {
    let classify = "(define (sign n) (cond ((< n 0) 'neg) ((= n 0) 'zero) (else 'pos)))";
    assert_eq!(ev(&format!("{classify} (list (sign -3) (sign 0) (sign 7))")), "(neg zero pos)");
    assert_eq!(ev("(cond (#f 1) (42))"), "42");
    assert_eq!(ev("(cond (#f 1))"), "#f");
    assert!(ev("(cond (else 1) (#t 2))").contains("else must be the last"));
    assert_eq!(ev("(define x 0) (when (> 3 2) (set! x 5) (+ x 1))"), "6");
    assert_eq!(ev("(unless (> 3 2) 'never)"), "#f");
    assert_eq!(ev("(unless #f 'ran)"), "ran");
}

#[test]
fn let_star_sees_earlier_bindings() {
    assert_eq!(ev("(let* ((x 2) (y (* x 10))) (+ x y))"), "22");
    // plain let evaluates inits in the outer scope
    assert_eq!(ev("(define x 1) (let ((x 2) (y x)) y)"), "1");
    assert!(ev("(let ((x)) x)").contains("bad binding"));
}

#[test]
fn named_let_loops_in_constant_stack() {
    assert_eq!(ev("(let loop ((i 0) (acc 0)) (if (= i 100000) acc (loop (+ i 1) (+ acc i))))"), "4999950000");
    assert_eq!(ev("(let fact ((n 5)) (if (< n 2) 1 (* n (fact (- n 1)))))"), "120");
}

#[test]
fn cond_tail_position_does_not_overflow() {
    let src = "(define (count n) (cond ((= n 0) 'done) (else (count (- n 1))))) (count 100000)";
    assert_eq!(ev(src), "done");
}
