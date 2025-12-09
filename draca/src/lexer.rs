use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    OpenParen,
    CloseParen,
    Number(f64),
    Symbol(String),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::Number(n) => write!(f, "{n}"),
            Token::Symbol(s) => write!(f, "{s}"),
        }
    }
}

impl Token {
    pub fn tokenize(expr: &str) -> Result<Vec<Self>, String> {
        let expr = expr.replace('(', " ( ").replace(')', " ) ");

        let words = expr.split_whitespace();

        let mut tokens = vec![];

        for word in words {
            match word {
                "(" => tokens.push(Token::OpenParen),
                ")" => tokens.push(Token::CloseParen),
                _ => {
                    let i = word.parse::<f64>();
                    if let Ok(num) = i {
                        tokens.push(Token::Number(num));
                    } else {
                        tokens.push(Token::Symbol(word.to_string()));
                    }
                }
            }
        }

        Ok(tokens)
    }
}
