use std::collections::HashSet;

use regex::Regex;

pub struct Tokenizer {
    special_whites: Vec<Regex>,
    special_blacks: Vec<Regex>,
    symbols: HashSet<char>,
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

    pub fn tokenize<'a>(&self, msg: &'a str) -> Vec<MessageToken<'a>> {
        let mut tokens = Vec::new();
        for pre_token in self.pre_tokenize(msg) {
            match pre_token {
                // PreToken::Special(slice) => tokens.push(Token::Special(slice)),
                PreToken::SpecialWhite(slice) => {
                    tokens.push(MessageToken::SpecialWhite(slice));
                }
                PreToken::SpecialBlack(slice) => {
                    tokens.push(MessageToken::SpecialBlack(slice));
                }
                PreToken::Unrefined(slice) => {
                    tokens.append(&mut split_token(slice, &self.symbols));
                }
            }
        }
        tokens
    }

    fn pre_tokenize<'a>(&self, msg: &'a str) -> Vec<PreToken<'a>> {
        let mut pre_toks = vec![PreToken::Unrefined(msg)];
        for regex in &self.special_whites {
            let mut new_pre_toks = Vec::new();
            for pre_tok in pre_toks {
                match pre_tok {
                    // PreToken::Special(slice) => {
                    //     new_pre_toks.push(PreToken::Special(slice));
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
                    // PreToken::Special(slice) => {
                    //     new_pre_toks.push(PreToken::Special(slice));
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

fn split_special<'a, Special: Fn(&'a str) -> PreToken>(
    msg: &'a str,
    regex: &Regex,
    special_type: Special,
) -> Vec<PreToken<'a>> {
    let mut last_idx = 0;
    let mut pre_tokens = Vec::new();
    for m in regex.find_iter(msg) {
        if m.is_empty() {
            continue;
        }
        if m.start() != last_idx {
            pre_tokens.push(PreToken::Unrefined(&msg[last_idx..m.start()]));
        }
        pre_tokens.push(special_type(m.as_str()));
        last_idx = m.end();
    }
    if last_idx != msg.len() {
        pre_tokens.push(PreToken::Unrefined(&msg[last_idx..]));
    }
    pre_tokens
}

fn split_token<'a>(msg: &'a str, symbols: &HashSet<char>) -> Vec<MessageToken<'a>> {
    let mut start_idx = 0;
    let mut toks = Vec::new();
    while let Some(end_idx) = msg[start_idx..]
        .find(|c: char| c.is_whitespace() || symbols.contains(&c))
        .map(|idx| idx + start_idx)
    {
        if start_idx < end_idx {
            toks.push(MessageToken::with(&msg[start_idx..end_idx], symbols));
        }
        toks.push(MessageToken::with(&msg[end_idx..end_idx + 1], symbols));
        start_idx = end_idx + 1;
    }
    if start_idx < msg.len() {
        toks.push(MessageToken::with(&msg[start_idx..], symbols));
    }
    toks
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum MessageToken<'a> {
    Alphabetic(&'a str),
    Numeric(&'a str),
    Symbolic(&'a str),
    Whitespace(&'a str),
    Impure(&'a str),
    // Special(&'a str),
    SpecialWhite(&'a str),
    SpecialBlack(&'a str),
}

impl<'a> MessageToken<'a> {
    pub fn with(slice: &'a str, symbols: &HashSet<char>) -> MessageToken<'a> {
        if slice.chars().all(char::is_alphabetic) {
            MessageToken::Alphabetic(slice)
        } else if slice.chars().all(char::is_numeric) {
            MessageToken::Numeric(slice)
        } else if slice.len() == 1 {
            if slice.chars().all(char::is_whitespace) {
                MessageToken::Whitespace(slice)
            } else if slice.chars().all(|c| symbols.contains(&c)) {
                MessageToken::Symbolic(slice)
            } else {
                // panic!("Invalid token '{}'", slice);
                MessageToken::Impure(slice)
            }
        } else {
            MessageToken::Impure(slice)
        }
    }

    pub fn as_str(&self) -> &'a str {
        match self {
            MessageToken::Alphabetic(slice) => slice,
            MessageToken::Numeric(slice) => slice,
            MessageToken::Symbolic(slice) => slice,
            MessageToken::Whitespace(slice) => slice,
            MessageToken::Impure(slice) => slice,
            MessageToken::SpecialWhite(slice) => slice,
            MessageToken::SpecialBlack(slice) => slice,
            // Token::Special(slice) => slice,
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
            MessageToken::Alphabetic("Fan"),
            MessageToken::Whitespace(" "),
            MessageToken::SpecialWhite("fan_2"),
            MessageToken::Whitespace(" "),
            MessageToken::Alphabetic("speed"),
            MessageToken::Whitespace(" "),
            MessageToken::Alphabetic("is"),
            MessageToken::Whitespace(" "),
            MessageToken::Alphabetic("set"),
            MessageToken::Whitespace(" "),
            MessageToken::Alphabetic("to"),
            MessageToken::Whitespace(" "),
            MessageToken::SpecialBlack("12.3114"),
            MessageToken::Whitespace(" "),
            MessageToken::Alphabetic("on"),
            MessageToken::Whitespace(" "),
            MessageToken::Alphabetic("machine"),
            MessageToken::Whitespace(" "),
            MessageToken::Alphabetic("sys"),
            MessageToken::Symbolic("."),
            MessageToken::Alphabetic("node"),
            MessageToken::Symbolic("."),
            MessageToken::SpecialWhite("fan_3"),
            MessageToken::Whitespace(" "),
            MessageToken::Alphabetic("on"),
            MessageToken::Whitespace(" "),
            MessageToken::Alphabetic("node"),
            MessageToken::Whitespace(" "),
            MessageToken::Numeric("12"),
        ];
        assert_eq!(expected, computed);
    }
}
