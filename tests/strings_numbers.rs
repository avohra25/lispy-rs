use lispy_rs::{global_env, run};

fn ev(src: &str) -> String {
    run(src, &global_env()).map(|v| v.to_string()).unwrap_or_else(|e| format!("ERR {e}"))
}

#[test]
fn type_predicates() {
    assert_eq!(ev("(list (number? 1) (string? \"a\") (symbol? 'a) (boolean? #f) (list? '()))"), "(#t #t #t #t #t)");
    assert_eq!(ev("(list (number? \"1\") (string? 'a) (symbol? \"a\") (boolean? 0) (list? 1))"), "(#f #f #f #f #f)");
}

#[test]
fn integer_arithmetic_follows_scheme_signs() {
    assert_eq!(ev("(list (quotient 17 5) (remainder 17 5) (modulo 17 5))"), "(3 2 2)");
    assert_eq!(ev("(list (quotient -7 2) (remainder -7 2) (modulo -7 2))"), "(-3 -1 1)");
    assert_eq!(ev("(modulo 7 -2)"), "-1");
    assert!(ev("(modulo 1 0)").contains("division by zero"));
    assert!(ev("(quotient 1.5 1)").contains("expected integers"));
    assert_eq!(ev("(list (abs -4) (min 3 1 2) (max 3 1 2))"), "(4 1 3)");
    assert!(ev("(min)").contains("needs arguments"));
}

#[test]
fn string_procedures() {
    assert_eq!(ev("(string-append \"foo\" \"-\" \"bar\")"), "\"foo-bar\"");
    assert_eq!(ev("(string-append)"), "\"\"");
    assert_eq!(ev("(string-length \"héllo\")"), "5");
    assert_eq!(ev("(substring \"héllo\" 1 3)"), "\"él\"");
    assert_eq!(ev("(substring \"hello\" 2)"), "\"llo\"");
    assert!(ev("(substring \"hi\" 1 5)").contains("out of bounds"));
    assert_eq!(ev("(string=? \"a\" \"a\" \"a\")"), "#t");
    assert!(ev("(string-append \"a\" 1)").contains("expected string"));
}

#[test]
fn conversions() {
    assert_eq!(ev("(number->string 42)"), "\"42\"");
    assert_eq!(ev("(+ 1 (string->number \"2.5\"))"), "3.5");
    assert_eq!(ev("(string->number \"nope\")"), "#f");
    assert_eq!(ev("(symbol->string 'abc)"), "\"abc\"");
    assert_eq!(ev("(equal? (string->symbol \"x\") 'x)"), "#t");
}

#[test]
fn error_raises_a_user_error() {
    assert_eq!(ev("(define (safe-div a b) (if (= b 0) (error \"safe-div: zero divisor for\" a) (/ a b))) (safe-div 1 0)"),
               "ERR safe-div: zero divisor for 1");
}

#[test]
fn library_example_runs() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/library.scm")).unwrap();
    assert!(run(&src, &global_env()).is_ok());
}
