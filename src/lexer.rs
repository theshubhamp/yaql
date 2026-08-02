use std::fmt;
use std::str::Chars;

pub type Spanned<'input> = Result<(usize, Tok<'input>, usize), LexError>;

#[derive(Debug, Clone, PartialEq)]
pub enum Tok<'input> {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    IndexLBracket,
    IndexRBracket,
    Comma,
    Dot,
    QuestionDot,
    Plus,
    Minus,
    Star,
    Slash,
    Mod,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    Match,
    NotMatch,
    And,
    Or,
    Not,
    In,
    Arrow,
    MappingArrow,
    Number(&'input str),
    SingleQuotedString(&'input str),
    DoubleQuotedString(&'input str),
    BacktickString(&'input str),
    True,
    False,
    Null,
    Identifier(&'input str),
    Dollar(&'input str),
    FunctionName(&'input str),
}

impl<'input> fmt::Display for Tok<'input> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Tok::LParen => write!(f, "("),
            Tok::RParen => write!(f, ")"),
            Tok::LBrace => write!(f, "{{"),
            Tok::RBrace => write!(f, "}}"),
            Tok::LBracket => write!(f, "["),
            Tok::RBracket => write!(f, "]"),
            Tok::IndexLBracket => write!(f, "["),
            Tok::IndexRBracket => write!(f, "]"),
            Tok::Comma => write!(f, ","),
            Tok::Dot => write!(f, "."),
            Tok::QuestionDot => write!(f, "?."),
            Tok::Plus => write!(f, "+"),
            Tok::Minus => write!(f, "-"),
            Tok::Star => write!(f, "*"),
            Tok::Slash => write!(f, "/"),
            Tok::Mod => write!(f, "mod"),
            Tok::Lt => write!(f, "<"),
            Tok::Gt => write!(f, ">"),
            Tok::Le => write!(f, "<="),
            Tok::Ge => write!(f, ">="),
            Tok::Eq => write!(f, "="),
            Tok::Ne => write!(f, "!="),
            Tok::Match => write!(f, "=~"),
            Tok::NotMatch => write!(f, "!~"),
            Tok::And => write!(f, "and"),
            Tok::Or => write!(f, "or"),
            Tok::Not => write!(f, "not"),
            Tok::In => write!(f, "in"),
            Tok::Arrow => write!(f, "->"),
            Tok::MappingArrow => write!(f, "=>"),
            Tok::Number(s) => write!(f, "{}", s),
            Tok::SingleQuotedString(s) | Tok::DoubleQuotedString(s) | Tok::BacktickString(s) => write!(f, "{}", s),
            Tok::True => write!(f, "true"),
            Tok::False => write!(f, "false"),
            Tok::Null => write!(f, "null"),
            Tok::Identifier(s) => write!(f, "{}", s),
            Tok::Dollar(s) => write!(f, "{}", s),
            Tok::FunctionName(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug)]
pub struct LexError {
    pub pos: usize,
    pub ch: char,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unexpected character {:?} at position {}", self.ch, self.pos)
    }
}

pub struct Lexer<'input> {
    input: &'input str,
    chars: Chars<'input>,
    pos: usize,
    prev_tok: Option<Tok<'input>>,
    bracket_stack: Vec<BracketKind>,
}

#[derive(Clone, Copy, PartialEq)]
enum BracketKind {
    List,
    Index,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Lexer {
            input,
            chars: input.chars(),
            pos: 0,
            prev_tok: None,
            bracket_stack: Vec::new(),
        }
    }

    fn peek_byte_after(&self, consumed: usize) -> Option<char> {
        self.input[self.pos + consumed..].chars().next()
    }

    fn slice(&self, start: usize, end: usize) -> &'input str {
        &self.input[start..end]
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let start = self.pos;
            let ch = match self.chars.next() {
                Some(c) => {
                    self.pos += c.len_utf8();
                    c
                }
                None => return None,
            };

            if ch.is_whitespace() {
                continue;
            }

            // Multi-char operators
            match ch {
                '?' if self.peek_byte_after(0) == Some('.') => {
                    self.chars.next();
                    self.pos += 1;
                    let tok = Tok::QuestionDot;
                    self.prev_tok = Some(tok.clone());
                    return Some(Ok((start, tok, self.pos)));
                }
                '-' if self.peek_byte_after(0) == Some('>') => {
                    self.chars.next();
                    self.pos += 1;
                    let tok = Tok::Arrow;
                    self.prev_tok = Some(tok.clone());
                    return Some(Ok((start, tok, self.pos)));
                }
                '=' if self.peek_byte_after(0) == Some('>') => {
                    self.chars.next();
                    self.pos += 1;
                    let tok = Tok::MappingArrow;
                    self.prev_tok = Some(tok.clone());
                    return Some(Ok((start, tok, self.pos)));
                }
                '=' if self.peek_byte_after(0) == Some('~') => {
                    self.chars.next();
                    self.pos += 1;
                    let tok = Tok::Match;
                    self.prev_tok = Some(tok.clone());
                    return Some(Ok((start, tok, self.pos)));
                }
                '!' if self.peek_byte_after(0) == Some('~') => {
                    self.chars.next();
                    self.pos += 1;
                    let tok = Tok::NotMatch;
                    self.prev_tok = Some(tok.clone());
                    return Some(Ok((start, tok, self.pos)));
                }
                '!' if self.peek_byte_after(0) == Some('=') => {
                    self.chars.next();
                    self.pos += 1;
                    let tok = Tok::Ne;
                    self.prev_tok = Some(tok.clone());
                    return Some(Ok((start, tok, self.pos)));
                }
                '<' if self.peek_byte_after(0) == Some('=') => {
                    self.chars.next();
                    self.pos += 1;
                    let tok = Tok::Le;
                    self.prev_tok = Some(tok.clone());
                    return Some(Ok((start, tok, self.pos)));
                }
                '>' if self.peek_byte_after(0) == Some('=') => {
                    self.chars.next();
                    self.pos += 1;
                    let tok = Tok::Ge;
                    self.prev_tok = Some(tok.clone());
                    return Some(Ok((start, tok, self.pos)));
                }
                _ => {}
            }

            // Single-char tokens
            let tok = match ch {
                '(' => Tok::LParen,
                ')' => Tok::RParen,
                '{' => Tok::LBrace,
                '}' => Tok::RBrace,
                ',' => Tok::Comma,
                '.' => Tok::Dot,
                '+' => {
                    if is_expr_start(&self.prev_tok) {
                        if let Some(c) = self.peek_byte_after(0) {
                            if c.is_ascii_digit() {
                                return Some(self.lex_number(start));
                            }
                        }
                    }
                    Tok::Plus
                }
                '-' => {
                    if is_expr_start(&self.prev_tok) {
                        if let Some(c) = self.peek_byte_after(0) {
                            if c.is_ascii_digit() {
                                return Some(self.lex_number(start));
                            }
                        }
                    }
                    Tok::Minus
                }
                '*' => Tok::Star,
                '/' => Tok::Slash,
                '<' => Tok::Lt,
                '>' => Tok::Gt,
                '=' => Tok::Eq,
                '[' => {
                    if is_index_position(&self.prev_tok) {
                        self.bracket_stack.push(BracketKind::Index);
                        Tok::IndexLBracket
                    } else {
                        self.bracket_stack.push(BracketKind::List);
                        Tok::LBracket
                    }
                }
                ']' => {
                    match self.bracket_stack.pop() {
                        Some(BracketKind::Index) => Tok::IndexRBracket,
                        Some(BracketKind::List) => Tok::RBracket,
                        None => Tok::RBracket,
                    }
                }
                '$' => return Some(self.lex_dollar(start)),
                '\'' | '"' | '`' => return Some(self.lex_string(start, ch)),
                _ if ch.is_ascii_digit() => return Some(self.lex_number(start)),
                _ if ch.is_alphabetic() || ch == '_' => return Some(self.lex_word(start)),
                _ => return Some(Err(LexError { pos: start, ch })),
            };

            self.prev_tok = Some(tok.clone());
            return Some(Ok((start, tok, self.pos)));
        }
    }
}

impl<'input> Lexer<'input> {
    fn lex_number(&mut self, start: usize) -> Spanned<'input> {
        // Handle optional leading '-' (already consumed as first char)
        let mut has_dot = false;
        // If the number started with '-', the first char is '-' and digits follow
        while let Some(c) = self.peek_byte_after(0) {
            if c.is_ascii_digit() {
                self.chars.next();
                self.pos += c.len_utf8();
            } else if c == '.' && !has_dot {
                has_dot = true;
                self.chars.next();
                self.pos += 1;
            } else {
                break;
            }
        }
        let s = self.slice(start, self.pos);
        let tok = Tok::Number(s);
        self.prev_tok = Some(tok.clone());
        Ok((start, tok, self.pos))
    }

    fn lex_word(&mut self, start: usize) -> Spanned<'input> {
        while let Some(c) = self.peek_byte_after(0) {
            if c.is_alphanumeric() || c == '_' {
                self.chars.next();
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
        let word = self.slice(start, self.pos);
        let tok = match word {
            "true" => Tok::True,
            "false" => Tok::False,
            "null" => Tok::Null,
            "and" => Tok::And,
            "or" => Tok::Or,
            "not" => Tok::Not,
            "in" => Tok::In,
            "mod" => Tok::Mod,
            _ => {
                // Check if followed by `(` → function name
                if self.peek_byte_after(0) == Some('(') {
                    Tok::FunctionName(word)
                } else {
                    Tok::Identifier(word)
                }
            }
        };
        self.prev_tok = Some(tok.clone());
        Ok((start, tok, self.pos))
    }

    fn lex_dollar(&mut self, start: usize) -> Spanned<'input> {
        while let Some(c) = self.peek_byte_after(0) {
            if c.is_alphanumeric() || c == '_' {
                self.chars.next();
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
        let s = self.slice(start, self.pos);
        let tok = Tok::Dollar(s);
        self.prev_tok = Some(tok.clone());
        Ok((start, tok, self.pos))
    }

    fn lex_string(&mut self, start: usize, quote: char) -> Spanned<'input> {
        while let Some(c) = self.chars.next() {
            self.pos += c.len_utf8();
            if c == '\\' {
                // Skip escaped char
                if let Some(esc) = self.chars.next() {
                    self.pos += esc.len_utf8();
                }
            } else if c == quote {
                break;
            }
        }
        let s = self.slice(start, self.pos);
        let tok = match quote {
            '\'' => Tok::SingleQuotedString(s),
            '"' => Tok::DoubleQuotedString(s),
            '`' => Tok::BacktickString(s),
            _ => unreachable!(),
        };
        self.prev_tok = Some(tok.clone());
        Ok((start, tok, self.pos))
    }
}

fn is_index_position(prev: &Option<Tok>) -> bool {
    match prev {
        None => false,
        Some(t) => matches!(t,
            Tok::RParen | Tok::RBracket | Tok::IndexRBracket
            | Tok::Dollar(_) | Tok::True | Tok::False | Tok::Null
            | Tok::Number(_) | Tok::SingleQuotedString(_)
            | Tok::DoubleQuotedString(_) | Tok::BacktickString(_)
            | Tok::Identifier(_)
        ),
    }
}

fn is_expr_start(prev: &Option<Tok>) -> bool {
    match prev {
        None => true,
        Some(t) => matches!(t,
            Tok::LParen | Tok::LBrace | Tok::LBracket | Tok::IndexLBracket
            | Tok::Comma | Tok::MappingArrow | Tok::Dot | Tok::QuestionDot
            | Tok::Plus | Tok::Minus | Tok::Star | Tok::Slash | Tok::Mod
            | Tok::Lt | Tok::Gt | Tok::Le | Tok::Ge | Tok::Eq | Tok::Ne
            | Tok::Match | Tok::NotMatch | Tok::And | Tok::Or | Tok::Not
            | Tok::In | Tok::Arrow
        ),
    }
}