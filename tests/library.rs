use lispy_rs::{global_env, run};

fn ev(src: &str) -> String {
    run(src, &global_env()).map(|v| v.to_string()).unwrap_or_else(|e| format!("ERR {e}"))
}

#[test]
fn variadic_lambdas_and_rest_parameters() {
    assert_eq!(ev("((lambda args args) 1 2 3)"), "(1 2 3)");
    assert_eq!(ev("((lambda args args))"), "()");
    assert_eq!(ev("(define (f a . rest) (list a rest)) (f 1 2 3)"), "(1 (2 3))");
    assert_eq!(ev("(define (f a . rest) rest) (f 1)"), "()");
    assert_eq!(ev("(define (g . xs) (length xs)) (g 'a 'b)"), "2");
    assert!(ev("(define (f a . rest) a) (f)").contains("at least 1"));
    assert!(ev("(lambda (a . b c) a)").contains("exactly one rest parameter"));
}

#[test]
fn apply_spreads_its_last_argument() {
    assert_eq!(ev("(apply + '(1 2 3))"), "6");
    assert_eq!(ev("(apply + 1 2 '(3 4))"), "10");
    assert_eq!(ev("(apply (lambda (a b) (- a b)) '(10 4))"), "6");
    assert_eq!(ev("(apply apply (list + '(1 2)))"), "3");
    assert!(ev("(apply + 1)").contains("last argument must be a list"));
    assert!(ev("(apply)").starts_with("ERR"));
    assert_eq!(ev("(procedure? car) (procedure? (lambda () 1))"), "#t");
    assert_eq!(ev("(procedure? 'car)"), "#f");
}

#[test]
fn apply_in_tail_position_is_a_tail_call() {
    let src = "(define (down n) (if (= n 0) 'done (apply down (list (- n 1))))) (down 100000)";
    assert_eq!(ev(src), "done");
}

#[test]
fn prelude_list_library() {
    assert_eq!(ev("(map (lambda (x) (* x x)) '(1 2 3))"), "(1 4 9)");
    assert_eq!(ev("(map + '(1 2 3) '(10 20 30 40))"), "(11 22 33)");
    assert_eq!(ev("(filter (lambda (x) (> x 2)) '(1 5 2 7))"), "(5 7)");
    assert_eq!(ev("(foldl - 0 '(1 2 3))"), "-6");
    assert_eq!(ev("(foldr cons '() '(1 2 3))"), "(1 2 3)");
    assert_eq!(ev("(reduce + 0 '())"), "0");
    assert_eq!(ev("(reduce * 0 '(2 3 4))"), "24");
    assert_eq!(ev("(append '(1) '() '(2 3) '(4))"), "(1 2 3 4)");
    assert_eq!(ev("(append)"), "()");
    assert_eq!(ev("(reverse '(a b c))"), "(c b a)");
    assert_eq!(ev("(list-ref '(a b c) 2)"), "c");
    assert_eq!(ev("(member 2 '(1 2 3))"), "(2 3)");
    assert_eq!(ev("(member 9 '(1 2 3))"), "#f");
    assert_eq!(ev("(assoc 'b '((a 1) (b 2)))"), "(b 2)");
    assert_eq!(ev("(iota 4)"), "(0 1 2 3)");
    assert_eq!(ev("(iota 3 1)"), "(1 2 3)");
    assert_eq!(ev("(iota 3 0 5)"), "(0 5 10)");
    assert!(ev("(list-ref '(a) 3)").contains("out of range"));
    assert!(ev("(iota -1)").contains("non-negative integer"));
    assert_eq!(ev("(define n 0) (for-each (lambda (x) (set! n (+ n x))) '(1 2 3)) n"), "6");
}

#[test]
fn prelude_handles_long_lists() {
    // Deep enough that non-tail recursion would overflow the Rust stack.
    assert_eq!(ev("(length (map (lambda (x) (+ x 1)) (iota 5000)))"), "5000");
    assert_eq!(ev("(foldl + 0 (filter (lambda (x) (> x 4997)) (iota 5000)))"), "9997");
}

#[test]
fn user_definitions_can_shadow_the_prelude() {
    assert_eq!(ev("(define (map f xs) 'mine) (map car '((1)))"), "mine");
}
