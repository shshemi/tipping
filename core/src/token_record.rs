use hashbrown::{HashMap, HashSet};

use crate::traits::{TokenFilter, Tokenize};
use itertools::Itertools;

use rayon::prelude::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct TokenPair<'a>(&'a str, &'a str);

impl<'a> TokenPair<'a> {
    pub fn with(t1: &'a str, t2: &'a str) -> Self {
        if t1 > t2 {
            TokenPair(t1, t2)
        } else {
            TokenPair(t2, t1)
        }
    }
}

#[derive(Debug)]
pub struct TokenRecord<'a> {
    soc: HashMap<&'a str, u32>,
    poc: HashMap<TokenPair<'a>, u32>,
}

impl<'a> TokenRecord<'a> {
    pub fn new<Message, Tokenizer, Filter>(
        msgs: &'a [Message],
        tokenizer: &Tokenizer,
        tf: &Filter,
    ) -> Self
    where
        Message: AsRef<str> + Sync,
        Tokenizer: Tokenize + Sync,
        Filter: TokenFilter + Sync,
    {
        let (soc, poc) = msgs
            .iter()
            .par_bridge()
            .fold_with(
                (HashMap::new(), HashMap::new()),
                |(mut soc, mut poc), msg| {
                    let toks = tokenizer
                        .tokenize(msg.as_ref())
                        .into_iter()
                        .unique()
                        .filter(|tok| tf.token_filter(tok))
                        .map(|tok| tok.as_str())
                        .collect::<HashSet<_>>();

                    // Insert single occurances
                    for tok in &toks {
                        soc.entry(*tok).and_modify(|count| *count += 1).or_insert(1);
                    }

                    // Insert double occurances
                    for (tok1, tok2) in toks.iter().tuple_combinations() {
                        poc.entry(TokenPair::with(tok1, tok2))
                            .and_modify(|count| *count += 1)
                            .or_insert(1);
                    }
                    (soc, poc)
                },
            )
            .reduce(
                || (Default::default(), Default::default()),
                |(mut soc1, mut poc1), (mut soc2, mut poc2)| {
                    // merge soc
                    let soc = if soc1.len() > soc2.len() {
                        soc1.reserve(soc2.len());
                        for (tok, count) in soc2 {
                            soc1.entry(tok)
                                .and_modify(|count1| *count1 += count)
                                .or_insert(count);
                        }
                        soc1
                    } else {
                        soc2.reserve(soc1.len());
                        for (tok, count) in soc1 {
                            soc2.entry(tok)
                                .and_modify(|count1| *count1 += count)
                                .or_insert(count);
                        }
                        soc2
                    };

                    // merge poc
                    let poc = if poc1.len() > poc2.len() {
                        poc1.reserve(poc2.len());
                        for (pair, count) in poc2 {
                            poc1.entry(pair)
                                .and_modify(|count1| *count1 += count)
                                .or_insert(count);
                        }
                        poc1
                    } else {
                        poc2.reserve(poc1.len());
                        for (pair, count) in poc1 {
                            poc2.entry(pair)
                                .and_modify(|count1| *count1 += count)
                                .or_insert(count);
                        }
                        poc2
                    };
                    (soc, poc)
                },
            );

        Self { soc, poc }
    }

    pub fn occurence(&self, tok: impl AsRef<str>) -> Option<u32> {
        self.soc.get(tok.as_ref()).copied()
    }

    #[allow(dead_code)]
    pub fn coocurence(&self, tok1: impl AsRef<str>, tok2: impl AsRef<str>) -> Option<u32> {
        self.poc
            .get(&TokenPair::with(tok1.as_ref(), tok2.as_ref()))
            .copied()
    }

    pub fn dependency(&self, eve: &'a str, con: &'a str) -> Option<f32> {
        let double = *self.poc.get(&TokenPair::with(eve, con))?;
        let single = *self.soc.get(eve)?;
        Some((double as f32) / (single as f32))
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::Token;

    use super::*;

    #[test]
    fn test_all() {
        let msg = ["a x1 b", "a x2 b", "a x3 c", "a x4 c"];
        let tokenizer = MockTokenizer;
        let filter = MockFilter;
        let idep = TokenRecord::new(&msg, &tokenizer, &filter);
        let expected_soc = HashMap::from([
            ("a", 4),
            ("b", 2),
            ("c", 2),
            ("x1", 1),
            ("x2", 1),
            ("x3", 1),
            ("x4", 1),
        ]);
        let expected_poc = HashMap::from([
            (TokenPair::with("a", "x1"), 1),
            (TokenPair::with("a", "x2"), 1),
            (TokenPair::with("a", "x3"), 1),
            (TokenPair::with("a", "x4"), 1),
            (TokenPair::with("a", "b"), 2),
            (TokenPair::with("a", "c"), 2),
            (TokenPair::with("b", "x1"), 1),
            (TokenPair::with("b", "x2"), 1),
            (TokenPair::with("c", "x3"), 1),
            (TokenPair::with("c", "x4"), 1),
        ]);
        assert_eq!(expected_soc, idep.soc);
        assert_eq!(expected_poc, idep.poc);

        assert_eq!(idep.dependency("a", "x1"), Some(0.25));
        assert_eq!(idep.dependency("a", "x2"), Some(0.25));
        assert_eq!(idep.dependency("a", "x3"), Some(0.25));
        assert_eq!(idep.dependency("a", "x4"), Some(0.25));

        assert_eq!(idep.dependency("x1", "a"), Some(1.0));
        assert_eq!(idep.dependency("x2", "a"), Some(1.0));
        assert_eq!(idep.dependency("x3", "a"), Some(1.0));
        assert_eq!(idep.dependency("x4", "a"), Some(1.0));

        assert_eq!(idep.dependency("b", "x1"), Some(0.5));
        assert_eq!(idep.dependency("b", "x2"), Some(0.5));

        assert_eq!(idep.dependency("x1", "b"), Some(1.0));
        assert_eq!(idep.dependency("x2", "b"), Some(1.0));

        assert_eq!(idep.dependency("c", "x3"), Some(0.5));
        assert_eq!(idep.dependency("c", "x4"), Some(0.5));

        assert_eq!(idep.dependency("x3", "c"), Some(1.0));
        assert_eq!(idep.dependency("x4", "c"), Some(1.0));

        assert_eq!(idep.dependency("a", "b"), Some(0.5));
        assert_eq!(idep.dependency("a", "c"), Some(0.5));

        assert_eq!(idep.dependency("b", "a"), Some(1.0));
        assert_eq!(idep.dependency("c", "a"), Some(1.0));

        assert!(idep.occurence("a").is_some());
        assert!(idep.occurence("b").is_some());
        assert!(idep.occurence("c").is_some());
        assert!(idep.occurence("x1").is_some());
        assert!(idep.occurence("x2").is_some());
        assert!(idep.occurence("x3").is_some());
        assert!(idep.occurence("x4").is_some());

        assert!(idep.occurence("z").is_none());
        assert!(idep.occurence("x5").is_none());
    }

    struct MockTokenizer;
    impl Tokenize for MockTokenizer {
        fn tokenize<'a>(&self, msg: &'a str) -> Vec<Token<'a>> {
            msg.split(' ')
                .map(|slice| Token::with(slice, &HashSet::default()))
                .collect_vec()
        }
    }

    struct MockFilter;
    impl TokenFilter for MockFilter {
        fn token_filter(&self, _tok: &Token) -> bool {
            true
        }
    }
}
