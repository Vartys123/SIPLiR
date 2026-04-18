use std::fs::read_to_string;
use std::iter::Peekable;

#[derive(Debug)]
enum Token {
    Num(i32),
    AddOp,
    SubOp,
    MulOp,
    Identifier(String),
}

#[derive(Debug)]
enum Expr {
    Num(i32),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Identifier(String),
}

fn get_bp(token: &Token) -> u8 {
    match token {
        Token::AddOp | Token::SubOp => 10,
        Token::MulOp => 20,
        _ => 0,
    }
}

fn lex(input: &str) -> Vec<Token> {
    input
        .split_whitespace()
        .filter_map(|word| match word {
            "+" => Some(Token::AddOp),
            "-" => Some(Token::SubOp),
            "*" => Some(Token::MulOp),
            n => n.parse::<i32>().ok().map(Token::Num),
        })
        .collect()
}

fn parse_expr(iter: &mut Peekable<impl Iterator<Item = Token>>, bp_start: u8) -> Expr {
    let first_token = iter.next().expect("Error!!!!!!");
    let mut left = match first_token {
        Token::Num(n) => Expr::Num(n),
        Token::Identifier(s) => Expr::Identifier(s),
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
        Expr::Add(left, right) => eval(left) + eval(right),
        Expr::Sub(left, right) => eval(left) - eval(right),
        Expr::Mul(left, right) => eval(left) * eval(right),
        _ => unimplemented!(),
    }
}

fn main() {
    let tokens = lex(&read_to_string("something.lg").expect("Failed to read the file"));
    println!("Lexed: {:?}", tokens);
    let tokens = parse(tokens);
    println!("Parsed: {:?}", tokens);
    println!("Result: {:?}", eval(&tokens));
}
