use std::fs::read_to_string;
use std::iter::Peekable;

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
    EOF,
}

#[derive(Debug)]
enum Expr<'a> {
    Num(i32),
    Negate(Box<Expr<'a>>),
    Add(Box<Expr<'a>>, Box<Expr<'a>>),
    Sub(Box<Expr<'a>>, Box<Expr<'a>>),
    Mul(Box<Expr<'a>>, Box<Expr<'a>>),
    Div(Box<Expr<'a>>, Box<Expr<'a>>),
    Pow(Box<Expr<'a>>, Box<Expr<'a>>),
    Parentheses(Box<Expr<'a>>),
    Identifier(&'a str),
}

fn get_bp(token: &Token) -> u8 {
    match token {
        Token::AddOp | Token::SubOp => 10,
        Token::MulOp | Token::DivOp => 20,
        Token::PowOp => 30,
        _ => 0,
    }
}

fn lex(input: &str) -> Vec<Token> {
    let bytes = input.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
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
            b'0'..=b'9' => {
                let start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1
                }
                let n = input[start..i].parse().expect("Error!!!!!!");
                tokens.push(Token::Num(n));
            }
            b'a'..=b'z' | b'A'..=b'Z' => {
                let start = i;
                while i < bytes.len() && bytes[i].is_ascii_alphanumeric() {
                    i += 1
                }
                tokens.push(Token::Identifier(&input[start..i]));
            }
            _ => i += 1,
        }
    }
    tokens.push(Token::EOF);
    tokens
}

fn parse_expr<'a>(iter: &mut Peekable<impl Iterator<Item = Token<'a>>>, bp_start: u8) -> Expr<'a> {
    let first_token = iter.next().expect("Error!!!!!!");
    let mut left = match first_token {
        Token::Num(n) => Expr::Num(n),
        Token::Identifier(s) => Expr::Identifier(s),
        Token::LParen => {
            let inner = parse_expr(iter, 0);
            iter.next().expect("Error!!!!!!");
            inner
        }
        Token::SubOp => {
            let inner = parse_expr(iter, 25);
            Expr::Negate(Box::new(inner))
        }
        _ => unimplemented!(),
    };

    while let Some(next_token) = iter.peek() {
        let bp = get_bp(&next_token);
        if bp <= bp_start {
            break;
        }
        let op = iter.next().unwrap();
        let right = parse_expr(iter, bp);
        left = match op {
            Token::AddOp => Expr::Add(Box::new(left), Box::new(right)),
            Token::SubOp => Expr::Sub(Box::new(left), Box::new(right)),
            Token::MulOp => Expr::Mul(Box::new(left), Box::new(right)),
            Token::DivOp => Expr::Div(Box::new(left), Box::new(right)),
            Token::PowOp => Expr::Pow(Box::new(left), Box::new(right)),
            _ => unimplemented!(),
        }
    }
    left
}

fn parse(lexed: Vec<Token>) -> Expr {
    let mut iter = lexed.into_iter().peekable();
    parse_expr(&mut iter, 0)
}

fn eval(parsed: &Expr) -> i32 {
    match parsed {
        Expr::Num(n) => *n,
        Expr::Negate(inner) => -eval(inner),
        Expr::Add(left, right) => eval(left) + eval(right),
        Expr::Sub(left, right) => eval(left) - eval(right),
        Expr::Mul(left, right) => eval(left) * eval(right),
        Expr::Div(left, right) => eval(left) / eval(right),
        Expr::Pow(left, right) => eval(left).pow(eval(right).try_into().expect("Error!!!!!!")),
        _ => unimplemented!(),
    }
}

fn main() {
    let file_content = read_to_string("something.lg").expect("Failed to read the file");
    let tokens = lex(&file_content);
    println!("Lexed: {:?}", tokens);
    let tokens = parse(tokens);
    println!("Parsed: {:?}", tokens);
    println!("Result: {:?}", eval(&tokens));
}
