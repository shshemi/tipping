use hashbrown::{HashMap, HashSet};

use rayon::prelude::*;

use crate::{
    tokenizer::{Token, Tokenizer},
    traits::Tokenize,
};

pub fn shared_slices<'a, Iter: Iterator<Item = &'a str> + Send>(
    iter: Iter,
    tokenizer: &Tokenizer,
    filter_alphabetic: bool,
    filter_numeric: bool,
    filter_impure: bool,
) -> HashSet<&'a str> {
    iter.par_bridge()
        .map(|msg| tokenizer.tokenize(msg))
        .map(|toks_vec| {
            toks_vec
                .into_iter()
                .filter_map(|tok| match tok {
                    Token::SpecialWhite(slice) => Some(slice),
                    Token::Whitespace(slice) => Some(slice),
                    Token::Symbolic(slice) => Some(slice),
                    Token::Alphabetic(slice) if filter_alphabetic => Some(slice),
                    Token::Numeric(slice) if filter_numeric => Some(slice),
                    Token::Impure(slice) if filter_impure => Some(slice),
                    _ => None,
                })
                .collect::<HashSet<_>>()
        })
        .map(Some)
        .reduce(
            || None,
            |s1, s2| match (s1, s2) {
                (None, None) => None,
                (None, Some(s)) => Some(s),
                (Some(s), None) => Some(s),
                (Some(s1), Some(s2)) => Some(s1.intersection(&s2).copied().collect()),
            },
        )
        .unwrap_or_default()
}

pub fn templates<'a, Iter: Iterator<Item = &'a str> + Send>(
    iter: Iter,
    tokenizer: &Tokenizer,
    common_slices: &HashSet<&'a str>,
) -> HashSet<String> {
    iter.par_bridge()
        .map(|msg| {
            tokenizer
                .tokenize(msg)
                .into_iter()
                .map(|tok| tok.as_str())
                .fold(Vec::new(), |mut temp, slice| {
                    temp.push(common_slices.contains(slice).then_some(slice));
                    temp
                })
        })
        .fold_with(HashSet::new(), |mut temp_set, temp| {
            temp_set.insert(temp);
            temp_set
        })
        .reduce(Default::default, |s1, s2| {
            let (mut larger, smaller) = if s1.len() > s2.len() {
                (s1, s2)
            } else {
                (s2, s1)
            };
            larger.extend(smaller);
            larger
        })
        .into_iter()
        .map(|temp| temp.into_iter().map(|tok| tok.unwrap_or("<*>")).collect())
        .collect()
}

pub fn parameter_masks<'a, Iter: Iterator<Item = &'a str> + Send>(
    iter: Iter,
    tokenizer: &Tokenizer,
    common_slices: &HashSet<&'a str>,
) -> HashMap<String, String> {
    iter.par_bridge()
        .fold_with(HashMap::new(), |mut map, msg| {
            let toks = tokenizer.tokenize(msg);
            let mut msk_vec = String::with_capacity(msg.len());
            let mut should_parameterize = false;
            toks.iter()
                .copied()
                .enumerate()
                .for_each(|(idx, tok)| match tok {
                    Token::Symbolic(slice) => {
                        if common_slices.contains(slice) {
                            if matches!(
                                toks.get(idx + 1),
                                Some(Token::Whitespace(_)) | Some(Token::Symbolic(_)) | None
                            ) || matches!(
                                toks.get(idx + 1),
                                Some(Token::Whitespace(_)) | Some(Token::Symbolic(_)) | None
                            ) {
                                msk_vec.push('0');
                            } else if should_parameterize {
                                msk_vec.push('1');
                            } else {
                                msk_vec.push('0');
                            }
                        } else {
                            msk_vec.push('1');
                        }
                    }
                    Token::Whitespace(_) => {
                        msk_vec.push('0');
                        should_parameterize = false;
                    }
                    Token::SpecialWhite(slice) => {
                        (0..slice.len()).for_each(|_| msk_vec.push('0'));
                    }
                    Token::SpecialBlack(slice) => {
                        (0..slice.len()).for_each(|_| msk_vec.push('1'));
                    }
                    _ => {
                        let slice = tok.as_str();
                        if !common_slices.contains(slice) || should_parameterize {
                            (0..slice.len()).for_each(|_| msk_vec.push('1'));
                            should_parameterize = true;
                        } else {
                            (0..slice.len()).for_each(|_| msk_vec.push('0'));
                        }
                    }
                });
            map.insert(msg, msk_vec);
            map
        })
        .reduce(HashMap::new, |mut m1, m2| {
            for (k, v) in m2 {
                if !m1.contains_key(k) {
                    m1.insert(k, v);
                }
            }
            m1
        })
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_words() {
        let msgs = [
            "The value is a",
            "The value is b",
            "The value is c",
            "The value is d",
        ];
        let tokenizer = Tokenizer::new(Vec::new(), Vec::new(), "".chars().collect());
        let expected = HashSet::from(["The", "value", "is", " "]);
        let computed = shared_slices(msgs.into_iter(), &tokenizer, true, false, false);
        assert_eq!(computed, expected);
    }

    #[test]
    fn test_parameter_mask() {
        let msgs = ["The value is (val_123) ->"];
        let tokenizer = Tokenizer::new(
            Default::default(),
            Default::default(),
            "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~".chars().collect(),
        );
        let common_slices = HashSet::from(["The", "value", "is", "val", "-", ">", "(", ")", "_"]);
        let pm = parameter_masks(msgs.into_iter(), &tokenizer, &common_slices);
        for (k, v) in pm {
            println!("{k}");
            println!("{v}");
        }
    }
}
