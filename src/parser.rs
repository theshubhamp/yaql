use yaql_core::ast::Value;
use crate::lexer::{LexError, Lexer, Tok};
use std::fmt;
use std::str::FromStr;

pub struct ParseError {
    pub msg: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

pub struct Parser<'input> {
    tokens: Vec<Result<(usize, Tok<'input>, usize), LexError>>,
    pos: usize,
}

impl<'input> Parser<'input> {
    pub fn new(input: &'input str) -> Self {
        let tokens: Vec<_> = Lexer::new(input).collect();
        Parser { tokens, pos: 0 }
    }

    pub fn parse(input: &'input str) -> Result<Value, ParseError> {
        let mut p = Parser::new(input);
        let v = p.parse_value()?;
        if let Some(t) = p.peek_tok()? {
            return Err(ParseError {
                msg: format!("Unexpected token `{}`", t),
            });
        }
        Ok(v)
    }

    fn peek_tok(&self) -> Result<Option<Tok<'input>>, ParseError> {
        match self.tokens.get(self.pos) {
            None => Ok(None),
            Some(Ok((_, t, _))) => Ok(Some(t.clone())),
            Some(Err(e)) => Err(ParseError { msg: format!("{}", e) }),
        }
    }

    fn advance(&mut self) -> Result<Tok<'input>, ParseError> {
        match self.tokens.get(self.pos) {
            Some(Ok((_, t, _))) => {
                self.pos += 1;
                Ok(t.clone())
            }
            Some(Err(e)) => Err(ParseError { msg: format!("{}", e) }),
            None => Err(ParseError { msg: "Unexpected EOF".to_string() }),
        }
    }

    fn expect(&mut self, want: &Tok) -> Result<(), ParseError> {
        match self.peek_tok()? {
            Some(ref t) if t == want => {
                self.pos += 1;
                Ok(())
            }
            Some(t) => Err(ParseError {
                msg: format!("Unexpected token `{}`, expected `{}`", t, want),
            }),
            None => Err(ParseError {
                msg: format!("Unexpected EOF, expected `{}`", want),
            }),
        }
    }

    fn is_closing(&self) -> Result<bool, ParseError> {
        match self.peek_tok()? {
            Some(Tok::RParen) | Some(Tok::RBracket) | Some(Tok::RBrace) | Some(Tok::IndexRBracket) => Ok(true),
            _ => Ok(false),
        }
    }

    fn parse_value(&mut self) -> Result<Value, ParseError> {
        self.parse_or()
    }

    /// and / or / -> (left-assoc)
    fn parse_or(&mut self) -> Result<Value, ParseError> {
        let mut left = self.parse_and()?;
        loop {
            match self.peek_tok()? {
                Some(Tok::Or) => {
                    self.advance()?;
                    let right = self.parse_and()?;
                    left = Value::BinaryOperator(Box::new(left), "or".to_string(), Box::new(right));
                }
                Some(Tok::Arrow) => {
                    self.advance()?;
                    let right = self.parse_or()?;
                    left = Value::Lambda(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Value, ParseError> {
        let mut left = self.parse_comparison()?;
        loop {
            match self.peek_tok()? {
                Some(Tok::And) => {
                    self.advance()?;
                    let right = self.parse_comparison()?;
                    left = Value::BinaryOperator(Box::new(left), "and".to_string(), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// > < >= <= != = in
    fn parse_comparison(&mut self) -> Result<Value, ParseError> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek_tok()? {
                Some(Tok::Gt) => ">",
                Some(Tok::Lt) => "<",
                Some(Tok::Ge) => ">=",
                Some(Tok::Le) => "<=",
                Some(Tok::Ne) => "!=",
                Some(Tok::Eq) => "=",
                Some(Tok::In) => "in",
                _ => break,
            };
            self.advance()?;
            let right = self.parse_additive()?;
            left = Value::BinaryOperator(Box::new(left), op.to_string(), Box::new(right));
        }
        Ok(left)
    }

    /// + -
    fn parse_additive(&mut self) -> Result<Value, ParseError> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek_tok()? {
                Some(Tok::Plus) => "+",
                Some(Tok::Minus) => "-",
                _ => break,
            };
            self.advance()?;
            let right = self.parse_multiplicative()?;
            left = Value::BinaryOperator(Box::new(left), op.to_string(), Box::new(right));
        }
        Ok(left)
    }

    /// * / mod
    fn parse_multiplicative(&mut self) -> Result<Value, ParseError> {
        let mut left = self.parse_match()?;
        loop {
            let op = match self.peek_tok()? {
                Some(Tok::Star) => "*",
                Some(Tok::Slash) => "/",
                Some(Tok::Mod) => "mod",
                _ => break,
            };
            self.advance()?;
            let right = self.parse_match()?;
            left = Value::BinaryOperator(Box::new(left), op.to_string(), Box::new(right));
        }
        Ok(left)
    }

    /// =~ !~
    fn parse_match(&mut self) -> Result<Value, ParseError> {
        let mut left = self.parse_postfix()?;
        loop {
            let op = match self.peek_tok()? {
                Some(Tok::Match) => "=~",
                Some(Tok::NotMatch) => "!~",
                _ => break,
            };
            self.advance()?;
            let right = self.parse_atom()?;
            left = Value::BinaryOperator(Box::new(left), op.to_string(), Box::new(right));
        }
        Ok(left)
    }

    /// Postfix: Atom ('.' Value | '?.' Value | '[' indices ']')*
    fn parse_postfix(&mut self) -> Result<Value, ParseError> {
        let mut left = self.parse_atom()?;
        loop {
            match self.peek_tok()? {
                Some(Tok::Dot) => {
                    self.advance()?;
                    let right = self.parse_method_target()?;
                    left = match right {
                        Value::FunctionCall(name, args, kwargs) => {
                            Value::MethodCall(Box::new(left), false, name, args, kwargs)
                        }
                        other => Value::BinaryOperator(Box::new(left), ".".to_string(), Box::new(other)),
                    };
                }
                Some(Tok::QuestionDot) => {
                    self.advance()?;
                    let right = self.parse_method_target()?;
                    left = match right {
                        Value::FunctionCall(name, args, kwargs) => {
                            Value::MethodCall(Box::new(left), true, name, args, kwargs)
                        }
                        other => Value::BinaryOperator(Box::new(left), "?.".to_string(), Box::new(other)),
                    };
                }
                Some(Tok::IndexLBracket) => {
                    self.advance()?;
                    let indices = self.parse_comma_list()?;
                    self.expect(&Tok::IndexRBracket)?;
                    left = Value::Index(Box::new(left), indices);
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// Atom
    fn parse_atom(&mut self) -> Result<Value, ParseError> {
        match self.peek_tok()? {
            Some(Tok::Not) => {
                self.advance()?;
                let v = self.parse_atom()?;
                Ok(Value::UnaryOperator("not".to_string(), Box::new(v)))
            }
            Some(Tok::LParen) => {
                self.advance()?;
                let v = self.parse_value()?;
                self.expect(&Tok::RParen)?;
                Ok(v)
            }
            Some(Tok::Number(s)) => {
                let s = s.to_string();
                self.advance()?;
                if s.contains('.') {
                    Ok(Value::FloatLiteral(f64::from_str(&s).unwrap()))
                } else {
                    Ok(Value::IntLiteral(i64::from_str(&s).unwrap()))
                }
            }
            Some(Tok::SingleQuotedString(s)) | Some(Tok::DoubleQuotedString(s)) | Some(Tok::BacktickString(s)) => {
                let s = s.to_string();
                self.advance()?;
                Ok(Value::StringLiteral(s))
            }
            Some(Tok::True) => { self.advance()?; Ok(Value::BooleanLiteral(true)) }
            Some(Tok::False) => { self.advance()?; Ok(Value::BooleanLiteral(false)) }
            Some(Tok::Null) => { self.advance()?; Ok(Value::NullLiteral) }
            Some(Tok::Identifier(s)) => {
                let s = s.to_string();
                self.advance()?;
                Ok(Value::StringLiteral(s))
            }
            Some(Tok::Dollar(s)) => {
                let s = s.to_string();
                self.advance()?;
                Ok(Value::Dollar(s.strip_prefix('$').unwrap_or(&s).to_string()))
            }
            Some(Tok::FunctionName(s)) => {
                let name = s.to_string();
                self.advance()?;
                self.expect(&Tok::LParen)?;
                let (args, kwargs) = self.parse_arg_list()?;
                self.expect(&Tok::RParen)?;
                Ok(Value::FunctionCall(name, args, kwargs))
            }
            Some(Tok::LBracket) => {
                self.advance()?;
                let items = self.parse_comma_list()?;
                self.expect(&Tok::RBracket)?;
                Ok(Value::List(items))
            }
            Some(Tok::LBrace) => {
                self.advance()?;
                let entries = self.parse_mapping_list()?;
                self.expect(&Tok::RBrace)?;
                Ok(Value::Dict(entries))
            }
            Some(t) => Err(ParseError { msg: format!("Unexpected token `{}`", t) }),
            None => Err(ParseError { msg: "Unexpected EOF".to_string() }),
        }
    }

    fn parse_method_target(&mut self) -> Result<Value, ParseError> {
        match self.peek_tok()? {
            Some(Tok::FunctionName(name)) => {
                let name = name.to_string();
                self.advance()?;
                self.expect(&Tok::LParen)?;
                let (args, kwargs) = self.parse_arg_list()?;
                self.expect(&Tok::RParen)?;
                Ok(Value::FunctionCall(name, args, kwargs))
            }
            Some(Tok::Identifier(s)) => {
                let s = s.to_string();
                self.advance()?;
                Ok(Value::StringLiteral(s))
            }
            Some(t) => Err(ParseError { msg: format!("Unexpected token `{}`", t) }),
            None => Err(ParseError { msg: "Unexpected EOF".to_string() }),
        }
    }

    /// Comma-separated list of Values, stopping at closing bracket.
    fn parse_comma_list(&mut self) -> Result<Vec<Value>, ParseError> {
        let mut items = Vec::new();
        if self.is_closing()? { return Ok(items); }
        let first = self.parse_value()?;
        match self.peek_tok()? {
            Some(Tok::MappingArrow) => {
                self.advance()?;
                let val = self.parse_value()?;
                items.push(Value::BinaryOperator(Box::new(first), "=>".to_string(), Box::new(val)));
            }
            _ => { items.push(first); }
        }
        loop {
            match self.peek_tok()? {
                Some(Tok::Comma) => {
                    self.advance()?;
                    if self.is_closing()? { break; }
                    let key_or_arg = self.parse_value()?;
                    match self.peek_tok()? {
                        Some(Tok::MappingArrow) => {
                            self.advance()?;
                            let val = self.parse_value()?;
                            items.push(Value::BinaryOperator(Box::new(key_or_arg), "=>".to_string(), Box::new(val)));
                        }
                        _ => { items.push(key_or_arg); }
                    }
                }
                _ => break,
            }
        }
        Ok(items)
    }

    /// Parse function argument list: positional args then kwargs.
    fn parse_arg_list(&mut self) -> Result<(Vec<Value>, Vec<(Value, Value)>), ParseError> {
        let mut args = Vec::new();
        let mut kwargs = Vec::new();
        if self.is_closing()? { return Ok((args, kwargs)); }

        // Parse first item — could be arg, kwarg, or empty (,,)
        if let Some(Tok::Comma) = self.peek_tok()? {
            args.push(Value::NullLiteral);
        } else {
            let first = self.parse_value()?;
            match self.peek_tok()? {
                Some(Tok::MappingArrow) => {
                    self.advance()?;
                    let val = self.parse_value()?;
                    kwargs.push((first, val));
                }
                _ => { args.push(first); }
            }
        }

        loop {
            match self.peek_tok()? {
                Some(Tok::Comma) => {
                    self.advance()?;
                    if self.is_closing()? { break; }
                    // Handle empty arg (,,)
                    if let Some(Tok::Comma) = self.peek_tok()? {
                        args.push(Value::NullLiteral);
                        continue;
                    }
                    let key_or_arg = self.parse_value()?;
                    match self.peek_tok()? {
                        Some(Tok::MappingArrow) => {
                            self.advance()?;
                            let val = self.parse_value()?;
                            kwargs.push((key_or_arg, val));
                        }
                        _ => { args.push(key_or_arg); }
                    }
                }
                _ => break,
            }
        }
        Ok((args, kwargs))
    }

    /// Comma-separated list of key=>value mappings (for dict literals).
    fn parse_mapping_list(&mut self) -> Result<Vec<(Value, Value)>, ParseError> {
        let mut entries = Vec::new();
        if self.is_closing()? { return Ok(entries); }
        entries.push(self.parse_mapping()?);
        loop {
            match self.peek_tok()? {
                Some(Tok::Comma) => {
                    self.advance()?;
                    if self.is_closing()? { break; }
                    entries.push(self.parse_mapping()?);
                }
                _ => break,
            }
        }
        Ok(entries)
    }

    fn parse_mapping(&mut self) -> Result<(Value, Value), ParseError> {
        let key = self.parse_value()?;
        self.expect(&Tok::MappingArrow)?;
        let val = self.parse_value()?;
        Ok((key, val))
    }
}