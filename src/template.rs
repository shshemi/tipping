use std::collections::{HashMap, HashSet};

use rayon::prelude::*;

use crate::tokenizer::{MessageToken, Tokenizer};

pub fn common_words<'a, Iter: Iterator<Item = &'a str> + Send>(
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
                    MessageToken::SpecialWhite(slice) => Some(slice),
                    MessageToken::Whitespace(slice) => Some(slice),
                    MessageToken::Symbolic(slice) => Some(slice),
                    MessageToken::Alphabetic(slice) if filter_alphabetic => Some(slice),
                    MessageToken::Numeric(slice) if filter_numeric => Some(slice),
                    MessageToken::Impure(slice) if filter_impure => Some(slice),
                    _ => None,
                })
                .collect::<HashSet<_>>()
        })
        .reduce_with(|s1, s2| s1.intersection(&s2).copied().collect())
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
                    if common_slices.contains(slice) {
                        temp.push(Some(slice));
                    } else if !matches!(temp.last(), Some(None)) {
                        temp.push(None);
                    }
                    temp
                })
        })
        .fold_with(HashSet::new(), |mut temp_set, temp| {
            temp_set.insert(temp);
            temp_set
        })
        .reduce_with(|mut s1, s2| {
            s1.extend(s2);
            s1
        })
        .unwrap_or_default()
        .into_iter()
        .map(|temp| {
            temp.into_iter()
                .map(|tok| if let Some(slice) = tok { slice } else { "<*>" })
                .collect()
        })
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
            let mut msk = String::with_capacity(msg.len());
            toks.into_iter().for_each(|tok| {
                let slice = tok.as_str();
                if common_slices.contains(slice) {
                    msk.push_str(&"0".repeat(slice.len()));
                } else {
                    msk.push_str(&"1".repeat(slice.len()));
                }
            });
            map.insert(msg, msk);
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
        let computed = common_words(msgs.into_iter(), &tokenizer, true, false, false);
        assert_eq!(computed, expected);
    }
}
