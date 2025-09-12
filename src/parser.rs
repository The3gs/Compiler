use crate::ast::{Declaration, Expression, Statement, Type};

#[derive(Debug)]
pub enum Token {
    Comment,
    KwFn,
    KwLet,
    KwReturn,
    Number(u32),
    Identifier(String),
    StringLiteral(String),
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Comma,
    Colon,
    Semicolon,
    Equals,
    Add,
    Minus,
    Mod,
    Divide,
    Multiply,
    Bang,
    BangEquals,
}

fn get_tokens(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '(' => tokens.push(Token::OpenParen),
            ')' => tokens.push(Token::CloseParen),
            '[' => tokens.push(Token::OpenBracket),
            ']' => tokens.push(Token::CloseBracket),
            '{' => tokens.push(Token::OpenBrace),
            '}' => tokens.push(Token::CloseBrace),
            '=' => tokens.push(Token::Equals),
            ':' => tokens.push(Token::Colon),
            ';' => tokens.push(Token::Semicolon),
            ',' => tokens.push(Token::Comma),
            '+' => tokens.push(Token::Add),
            '-' => tokens.push(Token::Minus),
            '*' => tokens.push(Token::Multiply),
            '/' => {
                if chars.next_if(|c| *c == '/').is_some() {
                    while chars.next_if(|c| *c != '\n').is_some() {}
                    tokens.push(Token::Comment)
                } else {
                    tokens.push(Token::Divide)
                }
            }
            '%' => tokens.push(Token::Mod),
            '!' => {
                if chars.next_if(|c| *c == '=').is_some() {
                    tokens.push(Token::BangEquals)
                } else {
                    tokens.push(Token::Bang)
                }
            }
            '"' => {
                let mut string = String::new();
                loop {
                    match chars.next() {
                        Some('\\') => match chars.next() {
                            Some('\\') => string.push('\\'),
                            Some('n') => string.push('\n'),
                            Some('"') => string.push('"'),
                            Some(c) => panic!("Unknown escape char {c:?}"),
                            None => panic!("Reached eof while parsing string"),
                        },
                        Some('"') => break,
                        Some(c) => string.push(c),
                        None => panic!("Reached eof while parsing string"),
                    }
                }
                tokens.push(Token::StringLiteral(string))
            }
            '0'..='9' => {
                let mut n = c as u64 as u32 - 48;
                while let Some(d) = chars.next_if(|n| matches!(n, '0'..='9')) {
                    n *= 10;
                    n += d as u64 as u32 - 48;
                }
                tokens.push(Token::Number(n))
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                ident.push(c);
                while let Some(c) =
                    chars.next_if(|c| matches!(c, '0'..='9'|'a'..='z'|'A'..='Z'|'_'))
                {
                    ident.push(c);
                }
                tokens.push(match ident.as_str() {
                    "fn" => Token::KwFn,
                    "let" => Token::KwLet,
                    "return" => Token::KwReturn,
                    _ => Token::Identifier(ident),
                })
            }
            ' ' | '\t' | '\n' | '\r' => (),
            c => panic!("Unknown start of token {c:?}"),
        }
    }
    tokens
}

#[derive(Debug)]
pub enum Error {
    UnexpectedToken(Token),
    UnexpectedEof,
}

fn parse_type<T: Iterator<Item = Token>>(tokens: &mut Peekable2<T>) -> Result<Type, Error> {
    match tokens.next().ok_or(Error::UnexpectedEof)? {
        Token::Identifier(s) if s == "u32" => Ok(Type::U32),
        t => Err(Error::UnexpectedToken(t)),
    }
}

fn parse_statement<T: Iterator<Item = Token>>(
    tokens: &mut Peekable2<T>,
) -> Result<Statement, Error> {
    match tokens.first().as_ref().ok_or(Error::UnexpectedEof)? {
        Token::KwLet => {
            tokens.next();
            let name = match tokens.next() {
                Some(Token::Identifier(name)) => name,
                Some(t) => {
                    return Err(Error::UnexpectedToken(t));
                }
                None => return Err(Error::UnexpectedEof),
            };
            match tokens.next() {
                Some(Token::Colon) => {}
                Some(t) => {
                    return Err(Error::UnexpectedToken(t));
                }
                None => return Err(Error::UnexpectedEof),
            }
            let value_type = parse_type(tokens)?;

            match tokens.next() {
                Some(Token::Equals) => {}
                Some(t) => {
                    return Err(Error::UnexpectedToken(t));
                }
                None => return Err(Error::UnexpectedEof),
            }

            let value = parse_expression(tokens)?;

            Ok(Statement::Let(name, Some(value_type), value))
        }
        Token::KwReturn => {
            tokens.next();
            let expression = parse_expression(tokens)?;

            match tokens.next() {
                Some(Token::Semicolon) => {}
                Some(t) => {
                    return Err(Error::UnexpectedToken(t));
                }
                None => return Err(Error::UnexpectedEof),
            }

            Ok(Statement::Return(expression))
        }
        _ => {
            let expression = parse_expression(tokens)?;

            match tokens.next() {
                Some(Token::Semicolon) => {}
                Some(t) => {
                    return Err(Error::UnexpectedToken(t));
                }
                None => return Err(Error::UnexpectedEof),
            }

            Ok(Statement::Expr(expression))
        }
    }
}

fn parse_expression<T: Iterator<Item = Token>>(
    tokens: &mut Peekable2<T>,
) -> Result<Expression, Error> {
    parse_additive(tokens)
}

fn parse_additive<T: Iterator<Item = Token>>(
    tokens: &mut Peekable2<T>,
) -> Result<Expression, Error> {
    let mut expression = parse_multiplicative(tokens)?;
    while let Some(token) = tokens.next_if(|t| matches!(t, Token::Add | Token::Minus)) {
        let next_expression = parse_multiplicative(tokens)?;
        expression = match token {
            Token::Add => Expression::Add(Box::new(expression), Box::new(next_expression)),
            Token::Minus => Expression::Sub(Box::new(expression), Box::new(next_expression)),
            _ => unreachable!(),
        }
    }
    Ok(expression)
}

fn parse_multiplicative<T: Iterator<Item = Token>>(
    tokens: &mut Peekable2<T>,
) -> Result<Expression, Error> {
    let mut expression = parse_unary(tokens)?;
    while let Some(token) =
        tokens.next_if(|t| matches!(t, Token::Multiply | Token::Divide | Token::Mod))
    {
        let next_expression = parse_unary(tokens)?;
        expression = match token {
            Token::Mod => Expression::Mod(Box::new(expression), Box::new(next_expression)),
            Token::Divide => Expression::Div(Box::new(expression), Box::new(next_expression)),
            Token::Multiply => Expression::Mul(Box::new(expression), Box::new(next_expression)),
            _ => unreachable!(),
        }
    }
    Ok(expression)
}

fn parse_unary<T: Iterator<Item = Token>>(tokens: &mut Peekable2<T>) -> Result<Expression, Error> {
    parse_primary(tokens)
}

fn parse_primary<T: Iterator<Item = Token>>(
    tokens: &mut Peekable2<T>,
) -> Result<Expression, Error> {
    match tokens.next() {
        Some(Token::Number(n)) => return Ok(Expression::NumLiteral(n)),
        Some(Token::Identifier(name)) => {
            if tokens.next_if(|t| matches!(t, Token::OpenParen)).is_some() {
                let mut args = Vec::new();
                while tokens.next_if(|t| matches!(t, Token::CloseParen)).is_none() {
                    args.push(parse_expression(tokens)?);

                    if !(tokens.next_if(|t| matches!(t, Token::Comma)).is_some()
                        || matches!(tokens.first(), Some(Token::CloseParen)))
                    {
                        match tokens.next() {
                            Some(t) => return Err(Error::UnexpectedToken(t)),
                            None => return Err(Error::UnexpectedEof),
                        }
                    }
                }

                Ok(Expression::Call(name, args))
            } else {
                Ok(Expression::Variable(name))
            }
        }
        Some(Token::OpenParen) => {
            let expression = parse_expression(tokens)?;
            match tokens.next() {
                Some(Token::CloseParen) => Ok(expression),
                Some(t) => Err(Error::UnexpectedToken(t)),
                None => Err(Error::UnexpectedEof),
            }
        }
        Some(t) => return Err(Error::UnexpectedToken(t)),
        None => return Err(Error::UnexpectedEof),
    }
}

pub fn parse(input: &str) -> Result<Vec<Declaration>, Error> {
    let tokens = get_tokens(input);

    let mut iter = Peekable2::new(tokens.into_iter());

    let mut result = Vec::new();

    while let Some(token) = iter.next() {
        match token {
            Token::KwFn => {
                let name = match iter.next() {
                    Some(Token::Identifier(name)) => name,
                    Some(t) => {
                        return Err(Error::UnexpectedToken(t));
                    }
                    None => return Err(Error::UnexpectedEof),
                };

                match iter.next() {
                    Some(Token::OpenParen) => {}
                    Some(t) => {
                        return Err(Error::UnexpectedToken(t));
                    }
                    None => return Err(Error::UnexpectedEof),
                }

                let mut arguments = Vec::new();

                while let Some(token) = iter.next()
                    && !matches!(token, Token::CloseParen)
                {
                    match token {
                        Token::Identifier(arg_name) => {
                            match iter.next() {
                                Some(Token::Colon) => {}
                                Some(t) => {
                                    return Err(Error::UnexpectedToken(t));
                                }
                                None => return Err(Error::UnexpectedEof),
                            }

                            let arg_type = parse_type(&mut iter)?;

                            if !(iter
                                .next_if(|token| matches!(token, Token::Comma))
                                .is_some()
                                || iter
                                    .first()
                                    .as_ref()
                                    .is_some_and(|v| matches!(v, Token::CloseParen)))
                            {
                                match iter.next() {
                                    Some(t) => {
                                        return Err(Error::UnexpectedToken(t));
                                    }
                                    None => return Err(Error::UnexpectedEof),
                                }
                            }
                            arguments.push((arg_name, arg_type))
                        }
                        t => return Err(Error::UnexpectedToken(t)),
                    }
                }

                match iter.next() {
                    Some(Token::Colon) => {}
                    Some(t) => {
                        return Err(Error::UnexpectedToken(t));
                    }
                    None => return Err(Error::UnexpectedEof),
                }

                let return_type = parse_type(&mut iter)?;

                match iter.next() {
                    Some(Token::OpenBrace) => {}
                    Some(t) => {
                        return Err(Error::UnexpectedToken(t));
                    }
                    None => return Err(Error::UnexpectedEof),
                }

                let mut body = Vec::new();

                while iter
                    .next_if(|token| matches!(token, Token::CloseBrace))
                    .is_none()
                {
                    body.push(parse_statement(&mut iter)?);
                    match iter.next_if(|t| !matches!(t, Token::CloseBrace)) {
                        Some(Token::Semicolon) => {}
                        Some(t) => {
                            return Err(Error::UnexpectedToken(t));
                        }
                        None => {
                            if iter.first().is_none() {
                                return Err(Error::UnexpectedEof);
                            }
                        }
                    }
                }

                result.push(Declaration::Function {
                    name,
                    arguments,
                    return_type,
                    body,
                })
            }
            t => {
                return Err(Error::UnexpectedToken(t));
            }
        }
    }

    Ok(result)
}

enum Peeked2<T> {
    None,
    One(T),
    Two(T, T),
}

impl<T> Peeked2<T> {
    fn take(&mut self) -> Self {
        std::mem::replace(self, Peeked2::None)
    }
}

struct Peekable2<I>
where
    I: Iterator,
{
    iter: I,
    peeked: Peeked2<Option<I::Item>>,
}

impl<T: Iterator> Peekable2<T> {
    fn new(iter: T) -> Self {
        Self {
            iter,
            peeked: Peeked2::None,
        }
    }

    fn next(&mut self) -> Option<T::Item> {
        match self.peeked.take() {
            Peeked2::None => self.iter.next(),
            Peeked2::One(v0) => {
                self.peeked = Peeked2::None;
                v0
            }
            Peeked2::Two(v0, v1) => {
                self.peeked = Peeked2::One(v1);
                v0
            }
        }
    }

    fn first(&mut self) -> &Option<T::Item> {
        match &self.peeked {
            Peeked2::None => self.peeked = Peeked2::One(self.iter.next()),
            _ => (),
        }
        match &self.peeked {
            Peeked2::None => unreachable!(),
            Peeked2::One(t) => t,
            Peeked2::Two(t, _) => t,
        }
    }

    fn second(&mut self) -> &Option<T::Item> {
        match self.peeked.take() {
            Peeked2::None => self.peeked = Peeked2::Two(self.iter.next(), self.iter.next()),
            Peeked2::One(t) => self.peeked = Peeked2::Two(t, self.iter.next()),
            Peeked2::Two(t0, t1) => self.peeked = Peeked2::Two(t0, t1),
        }
        match &self.peeked {
            Peeked2::Two(_, t) => t,
            _ => unreachable!(),
        }
    }

    fn next_if(&mut self, predicate: impl Fn(&T::Item) -> bool) -> Option<T::Item> {
        let a = self.first().as_ref()?;

        if predicate(a) { self.next() } else { None }
    }
}
