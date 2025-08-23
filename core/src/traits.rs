use crate::tokenizer::Token;

pub trait Tokenize {
    fn tokenize<'a>(&self, msg: &'a str) -> Vec<Token<'a>>;
}

pub trait TokenFilter {
    fn token_filter(&self, tok: &Token) -> bool;
}