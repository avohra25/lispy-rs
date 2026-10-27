; A tour of the standard library: cond, named let, variadic procedures,
; apply, the list prelude and string procedures.

(define (fizzbuzz n)
  (cond ((= (modulo n 15) 0) "FizzBuzz")
        ((= (modulo n 3) 0) "Fizz")
        ((= (modulo n 5) 0) "Buzz")
        (else (number->string n))))
(display (map fizzbuzz (iota 15 1)))

; named let as a loop: sum of squares of the odd numbers below 20
(display "odd squares:"
  (let loop ((i 0) (acc 0))
    (cond ((= i 20) acc)
          ((= (modulo i 2) 1) (loop (+ i 1) (+ acc (* i i))))
          (else (loop (+ i 1) acc)))))

; variadic procedures and apply
(define (average x . more)
  (/ (apply + x more) (+ 1 (length more))))
(display "average:" (average 3 4 8))

; words joined with a separator
(define (join sep words)
  (if (null? words)
      ""
      (foldl (lambda (acc w) (string-append acc sep w)) (car words) (cdr words))))
(display (join ", " (map symbol->string '(alpha beta gamma))))

; an association list as a tiny lookup table
(define stock '((apples 3) (pears 0) (plums 12)))
(display "in stock:" (map car (filter (lambda (row) (> (car (cdr row)) 0)) stock)))
