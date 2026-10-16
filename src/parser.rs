//! Tokenizer and reader: turns source text into `Expr` trees.
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(f64),
    Bool(bool),
    Sym(String),
    Str(String),
    List(Vec<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Num(n) => write!(f, "{n}"),
            Expr::Bool(b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
            Expr::Sym(s) => write!(f, "{s}"),
            Expr::Str(s) => write!(f, "{s:?}"),
            Expr::List(xs) => {
                let parts: Vec<String> = xs.iter().map(|x| x.to_string()).collect();
                write!(f, "({})", parts.join(" "))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LParen,
    RParen,
    Quote,
    Atom(String),
    Str(String),
}

pub fn tokenize(src: &str) -> Result<Vec<Token>, String> {
    let mut out = Vec::new();
    let mut chars = src.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            '(' => { chars.next(); out.push(Token::LParen); }
            ')' => { chars.next(); out.push(Token::RParen); }
            '\'' => { chars.next(); out.push(Token::Quote); }
            ';' => { while let Some(c) = chars.next() { if c == '\n' { break; } } }
            '"' => {
                chars.next();
                let mut s = String::new();
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some('\\') => match chars.next() {
                            Some('n') => s.push('\n'),
                            Some(e) => s.push(e),
                            None => return Err("unterminated string".into()),
                        },
                        Some(ch) => s.push(ch),
                        None => return Err("unterminated string".into()),
                    }
                }
                out.push(Token::Str(s));
            }
            c if c.is_whitespace() => { chars.next(); }
            _ => {
                let mut s = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() || "()'\";".contains(c) { break; }
                    s.push(c);
                    chars.next();
                }
                out.push(Token::Atom(s));
            }
        }
    }
    Ok(out)
}

/// Parse every top-level form in `src`.
pub fn parse_all(src: &str) -> Result<Vec<Expr>, String> {
    let toks = tokenize(src)?;
    let mut pos = 0;
    let mut forms = Vec::new();
    while pos < toks.len() {
        forms.push(read(&toks, &mut pos)?);
    }
    Ok(forms)
}

fn read(toks: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let tok = toks.get(*pos).ok_or("unexpected end of input")?;
    *pos += 1;
    match tok {
        Token::LParen => {
            let mut items = Vec::new();
            loop {
                match toks.get(*pos) {
                    None => return Err("missing ')'".into()),
                    Some(Token::RParen) => { *pos += 1; return Ok(Expr::List(items)); }
                    Some(_) => items.push(read(toks, pos)?),
                }
            }
        }
        Token::RParen => Err("unexpected ')'".into()),
        Token::Quote => Ok(Expr::List(vec![Expr::Sym("quote".into()), read(toks, pos)?])),
        Token::Str(s) => Ok(Expr::Str(s.clone())),
        Token::Atom(a) => Ok(match a.as_str() {
            "#t" => Expr::Bool(true),
            "#f" => Expr::Bool(false),
            _ => a.parse::<f64>().map(Expr::Num).unwrap_or_else(|_| Expr::Sym(a.clone())),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_lists() {
        let f = parse_all("(+ 1 (* 2 3))").unwrap();
        assert_eq!(f[0].to_string(), "(+ 1 (* 2 3))");
    }

    #[test]
    fn quote_and_strings_and_comments() {
        let f = parse_all("'(a \"hi\\n\") ; comment\n#t").unwrap();
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].to_string(), "(quote (a \"hi\\n\"))");
        assert_eq!(f[1], Expr::Bool(true));
    }

    #[test]
    fn reports_unbalanced_parens() {
        assert!(parse_all("(+ 1").is_err());
        assert!(parse_all(")").is_err());
    }
}
