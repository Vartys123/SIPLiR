#[derive(Debug)]
enum Token {
    Num(i32),
    AddOp,
    SubOp,
    Add(Box<Token>, Box<Token>),
    Sub(Box<Token>, Box<Token>),
    Identifier(String),
}

fn lex(code: &str) -> Vec<Token> {
    code.split_whitespace()
        .filter_map(|word| match word {
            "+" => Some(Token::AddOp),
            "-" => Some(Token::SubOp),
            n => n.parse::<i32>().ok().map(|n| Token::Num(n)),
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

fn main() {
    let tokens = lex(include_str!("../something.lg"));
    println!("Lexed: {:?}", tokens);
    println!("Parsed: {:?}", parse(tokens));
}
