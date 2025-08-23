use crate::tokenizer::Token;
use crate::traits::TokenFilter;

pub struct StaticFilter {
    alphabetic: bool,
    numeric: bool,
    impure: bool,
}

impl StaticFilter {
    pub fn with(alphabetic: bool, numeric: bool, impure: bool) -> Self {
        Self {
            alphabetic,
            numeric,
            impure,
        }
    }
}

impl TokenFilter for StaticFilter {
    fn token_filter(&self, tok: &Token) -> bool {
        match tok {
            Token::Alphabetic(_) => self.alphabetic,
            Token::Numeric(_) => self.numeric,
            Token::Impure(_) => self.impure,
            Token::Symbolic(_) => false,
            Token::Whitespace(_) => false,
            Token::SpecialBlack(_) => false,
            Token::SpecialWhite(_) => true,
        }
    }
}
