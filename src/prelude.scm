; Higher-order library procedures, written in lispy and loaded into every
; global environment. They need to call closures, so they live here rather
; than in builtins.rs. Each one accumulates in a tail call and reverses at the
; end, so long lists don't grow the Rust stack.

(define (foldl f acc xs)
  (if (null? xs) acc (foldl f (f acc (car xs)) (cdr xs))))

(define (foldr f init xs)
  (foldl (lambda (acc x) (f x acc)) init (reverse xs)))

(define (reduce f init xs)
  (if (null? xs) init (foldl f (car xs) (cdr xs))))

(define (any-null? lists)
  (cond ((null? lists) #f) ((null? (car lists)) #t) (else (any-null? (cdr lists)))))

(define (map1 f xs acc)
  (if (null? xs) (reverse acc) (map1 f (cdr xs) (cons (f (car xs)) acc))))

; (map f xs ...) stops at the shortest list, like R7RS.
(define (map f xs . more)
  (if (null? more)
      (map1 f xs '())
      (let loop ((lists (cons xs more)) (acc '()))
        (if (any-null? lists)
            (reverse acc)
            (loop (map1 cdr lists '())
                  (cons (apply f (map1 car lists '())) acc))))))

(define (for-each f xs . more)
  (apply map f xs more)
  #t)

(define (filter keep? xs)
  (let loop ((xs xs) (acc '()))
    (cond ((null? xs) (reverse acc))
          ((keep? (car xs)) (loop (cdr xs) (cons (car xs) acc)))
          (else (loop (cdr xs) acc)))))

(define (member x xs)
  (cond ((null? xs) #f) ((equal? x (car xs)) xs) (else (member x (cdr xs)))))

(define (assoc key alist)
  (cond ((null? alist) #f)
        ((equal? key (car (car alist))) (car alist))
        (else (assoc key (cdr alist)))))
