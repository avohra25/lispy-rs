; naive and iterative fibonacci
(define (fib n) (if (< n 2) n (+ (fib (- n 1)) (fib (- n 2)))))
(define (fib-fast n a b) (if (= n 0) a (fib-fast (- n 1) b (+ a b))))
(display "fib 20 =" (fib 20))
(display "fib 80 =" (fib-fast 80 0 1))
(define (map f xs) (if (null? xs) '() (cons (f (car xs)) (map f (cdr xs)))))
(display (map (lambda (x) (* x x)) (list 1 2 3 4 5)))
