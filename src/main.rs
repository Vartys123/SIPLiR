use std::fs::read_to_string;

#[derive(Debug)]
enum Token {
    Num(i32),
    AddOp,
    SubOp,
    Add(Box<Token>, Box<Token>),
    Sub(Box<Token>, Box<Token>),
    Identifier(String),
}

impl Token {
    fn deref(self) -> i32 {
        match self {
            Token::Num(n) => n,
            Token::Add(n1, n2) => n1.deref() + n2.deref(),
            Token::Sub(n1, n2) => n1.deref() - n2.deref(),
            _ => unimplemented!(),
        }
    }
}

fn lex(code: &str) -> Vec<Token> {
    code.split_whitespace()
        .filter_map(|word| match word {
            "+" => Some(Token::AddOp),
            "-" => Some(Token::SubOp),
            n => n.parse::<i32>().ok().map(Token::Num),
        })
        .collect()
}

fn parse(lexed: Vec<Token>) -> Vec<Token> {
    let mut iter = lexed.into_iter().peekable();
    let mut tokens = Vec::new();

    while let Some(token) = iter.next() {
        match token {
            Token::Num(n) => tokens.push(Token::Num(n)),
            Token::AddOp => {
                let left = tokens.pop().expect("Error!!!!!!");
                let right = iter.next().expect("Error!!!!!!");
                tokens.push(Token::Add(Box::new(left), Box::new(right)));
            }
            Token::SubOp => {
                let left = tokens.pop().expect("Error!!!!!!");
                let right = iter.next().expect("Error!!!!!!");
                tokens.push(Token::Sub(Box::new(left), Box::new(right)));
            }
            _ => unimplemented!(),
        }
    }
    tokens
}

fn run(parsed: Vec<Token>) {
    for token in parsed {
        println!("{:?}", token.deref());
    }
}

fn main() {
    let tokens = lex(&read_to_string("something.lg").expect("Failed to read the file"));
    println!("Lexed: {:?}", tokens);
    let tokens = parse(tokens);
    println!("Parsed: {:?}", tokens);
    run(tokens);
}
