use hashbrown::HashSet;

use fancy_regex::Regex;

use crate::traits::Tokenize;

pub struct Tokenizer {
    special_whites: Vec<Regex>,
    special_blacks: Vec<Regex>,
    symbols: HashSet<char>,
}

impl Tokenize for Tokenizer {
    fn tokenize<'a>(&self, msg: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        for pre_token in self.pre_tokenize(msg) {
            match pre_token {
                PreToken::SpecialWhite(slice) => {
                    tokens.push(Token::SpecialWhite(slice));
                }
                PreToken::SpecialBlack(slice) => {
                    tokens.push(Token::SpecialBlack(slice));
                }
                PreToken::Unrefined(slice) => {
                    tokens.append(&mut split_token(slice, &self.symbols));
                }
            }
        }
        tokens
    }
}

impl Tokenizer {
    pub fn new(
        special_whites: Vec<Regex>,
        special_blacks: Vec<Regex>,
        symbols: HashSet<char>,
    ) -> Self {
        Tokenizer {
            special_whites,
            special_blacks,
            symbols,
        }
    }

    pub fn new_with_symbols(&self, symbols: HashSet<char>) -> Self {
        Tokenizer {
            special_whites: self.special_whites.clone(),
            special_blacks: self.special_blacks.clone(),
            symbols,
        }
    }

    fn pre_tokenize<'a>(&self, msg: &'a str) -> Vec<PreToken<'a>> {
        let mut pre_toks = vec![PreToken::Unrefined(msg)];
        for regex in &self.special_whites {
            let mut new_pre_toks = Vec::new();
            for pre_tok in pre_toks {
                match pre_tok {
                    PreToken::SpecialWhite(slice) => {
                        new_pre_toks.push(PreToken::SpecialWhite(slice))
                    }
                    PreToken::SpecialBlack(slide) => {
                        new_pre_toks.push(PreToken::SpecialBlack(slide))
                    }
                    PreToken::Unrefined(slice) => {
                        new_pre_toks.append(&mut split_special(
                            slice,
                            regex,
                            PreToken::SpecialWhite,
                        ));
                    }
                }
            }
            pre_toks = new_pre_toks;
        }

        for regex in &self.special_blacks {
            let mut new_pre_toks = Vec::new();
            for pre_tok in pre_toks {
                match pre_tok {
                    // }
                    PreToken::SpecialWhite(slice) => {
                        new_pre_toks.push(PreToken::SpecialWhite(slice))
                    }
                    PreToken::SpecialBlack(slide) => {
                        new_pre_toks.push(PreToken::SpecialBlack(slide))
                    }
                    PreToken::Unrefined(slice) => {
                        new_pre_toks.append(&mut split_special(
                            slice,
                            regex,
                            PreToken::SpecialBlack,
                        ));
                    }
                }
            }
            pre_toks = new_pre_toks;
        }
        pre_toks
    }
}

fn split_special<'a, Special>(
    msg: &'a str,
    regex: &Regex,
    special_type: Special,
) -> Vec<PreToken<'a>>
where
    Special: Fn(&'a str) -> PreToken<'a>,
{
    let mut last_idx = 0;
    let mut pre_tokens = Vec::new();
    for m in regex.find_iter(msg).filter_map(Result::ok) {
        let start = m.start();
        let end = m.end();
        if end - start > 0 {
            if start != last_idx {
                pre_tokens.push(PreToken::Unrefined(&msg[last_idx..m.start()]));
            }
            pre_tokens.push(special_type(m.as_str()));
            last_idx = m.end();
        }
    }
    if last_idx != msg.len() {
        pre_tokens.push(PreToken::Unrefined(&msg[last_idx..]));
    }
    pre_tokens
}

fn split_token<'a>(msg: &'a str, symbols: &HashSet<char>) -> Vec<Token<'a>> {
    let mut start_idx = 0;
    let mut toks = Vec::new();
    while let Some(end_idx) = msg[start_idx..]
        .find(|c: char| c.is_whitespace() || symbols.contains(&c))
        .map(|idx| idx + start_idx)
    {
        if start_idx < end_idx {
            toks.push(Token::with(&msg[start_idx..end_idx], symbols));
        }
        toks.push(Token::with(&msg[end_idx..end_idx + 1], symbols));
        start_idx = end_idx + 1;
    }
    if start_idx < msg.len() {
        toks.push(Token::with(&msg[start_idx..], symbols));
    }
    toks
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub enum Token<'a> {
    Alphabetic(&'a str),
    Numeric(&'a str),
    Symbolic(&'a str),
    Whitespace(&'a str),
    Impure(&'a str),
    SpecialWhite(&'a str),
    SpecialBlack(&'a str),
}

impl<'a> Token<'a> {
    pub fn with(slice: &'a str, symbols: &HashSet<char>) -> Token<'a> {
        if slice.chars().all(char::is_alphabetic) {
            Token::Alphabetic(slice)
        } else if slice.chars().all(char::is_numeric) {
            Token::Numeric(slice)
        } else if slice.len() == 1 {
            if slice.chars().all(char::is_whitespace) {
                Token::Whitespace(slice)
            } else if slice.chars().all(|c| symbols.contains(&c)) {
                Token::Symbolic(slice)
            } else {
                Token::Impure(slice)
            }
        } else {
            Token::Impure(slice)
        }
    }

    pub fn as_str(&self) -> &'a str {
        match self {
            Token::Alphabetic(slice) => slice,
            Token::Numeric(slice) => slice,
            Token::Symbolic(slice) => slice,
            Token::Whitespace(slice) => slice,
            Token::Impure(slice) => slice,
            Token::SpecialWhite(slice) => slice,
            Token::SpecialBlack(slice) => slice,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum PreToken<'a> {
    // Special(&'a str),
    SpecialWhite(&'a str),
    SpecialBlack(&'a str),
    Unrefined(&'a str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizer_pre_tokenize() {
        let tokenizer = Tokenizer::new(
            vec![Regex::new(r"\ba\b").unwrap()],
            vec![Regex::new(r"\d+\.\d+").unwrap()],
            "".chars().collect(),
        );
        let expected = vec![
            PreToken::Unrefined("This "),
            PreToken::SpecialBlack("10001.2"),
            PreToken::Unrefined(" is "),
            PreToken::SpecialBlack("1.323"),
            PreToken::Unrefined(" "),
            PreToken::SpecialWhite("a"),
            PreToken::Unrefined(" "),
            PreToken::SpecialBlack("1.4411"),
            PreToken::Unrefined(" message"),
        ];
        let computed = tokenizer.pre_tokenize("This 10001.2 is 1.323 a 1.4411 message");
        assert_eq!(expected, computed);
    }

    #[test]
    fn tokenizer_tokenize() {
        let tokenizer = Tokenizer::new(
            vec![Regex::new(r"fan_\d+").unwrap()],
            vec![Regex::new(r"\d+\.\d+").unwrap()],
            ".".chars().collect(),
        );
        let computed = tokenizer
            .tokenize("Fan fan_2 speed is set to 12.3114 on machine sys.node.fan_3 on node 12");
        let expected = vec![
            Token::Alphabetic("Fan"),
            Token::Whitespace(" "),
            Token::SpecialWhite("fan_2"),
            Token::Whitespace(" "),
            Token::Alphabetic("speed"),
            Token::Whitespace(" "),
            Token::Alphabetic("is"),
            Token::Whitespace(" "),
            Token::Alphabetic("set"),
            Token::Whitespace(" "),
            Token::Alphabetic("to"),
            Token::Whitespace(" "),
            Token::SpecialBlack("12.3114"),
            Token::Whitespace(" "),
            Token::Alphabetic("on"),
            Token::Whitespace(" "),
            Token::Alphabetic("machine"),
            Token::Whitespace(" "),
            Token::Alphabetic("sys"),
            Token::Symbolic("."),
            Token::Alphabetic("node"),
            Token::Symbolic("."),
            Token::SpecialWhite("fan_3"),
            Token::Whitespace(" "),
            Token::Alphabetic("on"),
            Token::Whitespace(" "),
            Token::Alphabetic("node"),
            Token::Whitespace(" "),
            Token::Numeric("12"),
        ];
        assert_eq!(expected, computed);
    }
}
