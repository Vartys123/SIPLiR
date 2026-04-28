use std::collections::HashMap;
use std::fs::read_to_string;
use std::iter::Peekable;

#[derive(Debug)]
enum ErrorKind {
    LexerError(&'static str),
    ParserError(&'static str),
    EvalError(&'static str),
    IoError(std::io::Error),
}

impl From<std::io::Error> for ErrorKind {
    fn from(err: std::io::Error) -> Self {
        ErrorKind::IoError(err)
    }
}

#[derive(Debug)]
enum Token<'a> {
    Num(i32),
    AddOp,
    SubOp,
    MulOp,
    DivOp,
    PowOp,
    LParen,
    RParen,
    Identifier(&'a str),
    Let,
    Equal,
    EOF,
}

#[derive(Debug)]
enum UnaryOp {
    Negative,
    Increment,
    Decrement,
}

#[derive(Debug)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Debug)]
enum Expr<'a> {
    Num(i32),
    Unary(UnaryOp, Box<Expr<'a>>),
    Binary(BinaryOp, Box<Expr<'a>>, Box<Expr<'a>>),
    Identifier(&'a str),
}

#[derive(Debug)]
struct Scope<'a> {
    variables: HashMap<&'a str, i32>,
}

impl<'a> Scope<'a> {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

#[derive(Debug)]
enum Stmt<'a> {
    Let(&'a str, Expr<'a>),
    Expr(Expr<'a>),
}

fn get_bp(token: &Token) -> u8 {
    match token {
        Token::AddOp | Token::SubOp => 10,
        Token::MulOp | Token::DivOp => 20,
        Token::PowOp => 30,
        _ => 0,
    }
}

fn lex(input: &str) -> (Vec<Token<'_>>, Vec<ErrorKind>) {
    let bytes = input.as_bytes();
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b' ' | b'\n' | b'\t' | b'\r' => i += 1,
            b'+' => {
                tokens.push(Token::AddOp);
                i += 1
            }
            b'-' => {
                tokens.push(Token::SubOp);
                i += 1
            }
            b'*' => {
                tokens.push(Token::MulOp);
                i += 1
            }
            b'/' => {
                tokens.push(Token::DivOp);
                i += 1
            }
            b'^' => {
                tokens.push(Token::PowOp);
                i += 1
            }
            b'(' => {
                tokens.push(Token::LParen);
                i += 1
            }
            b')' => {
                tokens.push(Token::RParen);
                i += 1
            }
            b'=' => {
                tokens.push(Token::Equal);
                i += 1
            }
            b'0'..=b'9' => {
                let start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1
                }
                match input[start..i].parse() {
                    Ok(n) => tokens.push(Token::Num(n)),
                    Err(..) => errors.push(ErrorKind::LexerError("Failed to parse integer!")),
                }
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1
                }
                let word = &input[start..i];
                let token = match word {
                    "let" => Token::Let,
                    _ => Token::Identifier(word),
                };
                tokens.push(token);
            }
            _ => {
                errors.push(ErrorKind::LexerError("Unrecognized character!"));
                i += 1;
            }
        }
    }
    tokens.push(Token::EOF);
    (tokens, errors)
}

fn parse_expr<'a>(
    iter: &mut Peekable<impl Iterator<Item = Token<'a>>>,
    bp_start: u8,
) -> Result<Expr<'a>, ErrorKind> {
    let first_token = iter
        .next()
        .ok_or(ErrorKind::ParserError("Unexpected End Of Sequence!"))?;
    let mut left = match first_token {
        Token::Num(n) => Expr::Num(n),
        Token::Identifier(s) => Expr::Identifier(s),
        Token::LParen => {
            let inner = parse_expr(iter, 0)?;
            match iter.next() {
                Some(Token::RParen) => inner,
                _ => return Err(ErrorKind::ParserError("Unclosed Parenthesis!")),
            }
        }
        Token::SubOp => {
            let inner = parse_expr(iter, 25)?;
            Expr::Unary(UnaryOp::Negative, Box::new(inner))
        }
        _ => return Err(ErrorKind::ParserError("Unexpected first token!")),
    };

    while let Some(next_token) = iter.peek() {
        let bp = get_bp(&next_token);
        if bp <= bp_start {
            break;
        }
        let op = iter.next().unwrap();
        let right = match op {
            Token::PowOp => parse_expr(iter, bp - 1)?,
            _ => parse_expr(iter, bp)?,
        };
        let opkind = match op {
            Token::AddOp => BinaryOp::Add,
            Token::SubOp => BinaryOp::Sub,
            Token::MulOp => BinaryOp::Mul,
            Token::DivOp => BinaryOp::Div,
            Token::PowOp => BinaryOp::Pow,
            _ => return Err(ErrorKind::ParserError("Unexpected operation!")),
        };
        left = Expr::Binary(opkind, Box::new(left), Box::new(right));
    }
    Ok(left)
}

fn parse_stmt<'a>(
    iter: &mut Peekable<impl Iterator<Item = Token<'a>>>,
) -> Result<Stmt<'a>, ErrorKind> {
    match iter.peek() {
        Some(Token::Let) => {
            iter.next();
            let name = match iter.next() {
                Some(Token::Identifier(n)) => n,
                _ => {
                    return Err(ErrorKind::ParserError(
                        "Expected variable name after 'let'!",
                    ))
                }
            };
            match iter.next() {
                Some(Token::Equal) => {}
                _ => return Err(ErrorKind::ParserError("Expected '=' after variable name!")),
            }
            let expr = parse_expr(iter, 0)?;
            Ok(Stmt::Let(name, expr))
        }
        _ => Ok(Stmt::Expr(parse_expr(iter, 0)?)),
    }
}

fn parse(lexed: Vec<Token>) -> Result<Vec<Stmt>, ErrorKind> {
    let mut iter = lexed.into_iter().peekable();
    let mut stmts = Vec::new();

    while let Some(token) = iter.peek() {
        if let Token::EOF = token {
            break;
        };
        stmts.push(parse_stmt(&mut iter)?);
    }
    Ok(stmts)
}

fn eval_expr(expr: &Expr, scope: &Scope) -> Result<i32, ErrorKind> {
    Ok(match expr {
        Expr::Num(n) => *n,
        Expr::Unary(kind, inner) => match kind {
            UnaryOp::Negative => -eval_expr(inner, scope)?,
            _ => return Err(ErrorKind::EvalError("Unexpected unary operation!")),
        },
        Expr::Binary(kind, left, right) => {
            let l = eval_expr(left, scope)?;
            let r = eval_expr(right, scope)?;
            match kind {
                BinaryOp::Add => l + r,
                BinaryOp::Sub => l - r,
                BinaryOp::Mul => l * r,
                BinaryOp::Div => {
                    if r == 0 {
                        return Err(ErrorKind::EvalError("Division by zero!"));
                    }
                    l / r
                }
                BinaryOp::Pow => l.pow(
                    r.try_into()
                        .map_err(|_| ErrorKind::EvalError("Failed to convert the type!"))?,
                ),
            }
        }
        Expr::Identifier(name) => scope
            .variables
            .get(name)
            .copied()
            .ok_or(ErrorKind::EvalError("Unidentified variable!"))?,
    })
}

fn eval_stmt<'a>(stmt: &Stmt<'a>, scope: &mut Scope<'a>) -> Result<(), ErrorKind> {
    match stmt {
        Stmt::Let(name, expr) => {
            let expr = eval_expr(&expr, &scope)?;
            scope.variables.insert(name, expr);
            Ok(())
        }
        Stmt::Expr(expr) => {
            eval_expr(&expr, &scope)?;
            Ok(())
        }
    }
}

fn main() -> Result<(), ErrorKind> {
    let file_content = read_to_string("something.lg")?;

    let (tokens, errors) = lex(&file_content);
    if !errors.is_empty() {
        for error in errors {
            println!("{:?}", error);
        }
        return Ok(());
    }
    println!("Lexed: {:?}", tokens);

    let stmts = parse(tokens)?;
    println!("Parsed: {:?}", stmts);

    let mut scope = Scope::new();
    for stmt in stmts {
        eval_stmt(&stmt, &mut scope)?;
    }

    println!("Result: {:?}", scope);
    Ok(())
}
