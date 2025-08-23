mod graph;
mod token_record;
mod misc;
mod parser;
mod template;
mod token_filter;
mod tokenizer;
mod traits;
pub use misc::compile_into_regex;
pub use parser::Parser;
pub use tokenizer::Tokenizer;
pub use template::{shared_slices, parameter_masks};
pub use traits::Tokenize;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn trivial_test() {
        let msgs = [
            "a x1 x2 b",
            "a x2 b",
            "a x3 b",
            "a x4 b",
            "c x1 d",
            "c x2 d",
            "c x3 d",
            "c x4 d",
        ];
        let (clus, temps, masks) = Parser::default()
            .compute_templates()
            .compute_masks()
            .parse(&msgs);
        let s1 = clus[..4]
            .iter()
            .map(|item| item.unwrap())
            .collect::<HashSet<_>>();
        let s2 = clus[4..]
            .iter()
            .map(|item| item.unwrap())
            .collect::<HashSet<_>>();
        assert_eq!(s1.len(), 1);
        assert_eq!(s2.len(), 1);

        // let exp_temps = vec![

        // ]
        let exp_all_temps = ["a <*> b", "a <*> <*> b", "c <*> d"]
            .into_iter()
            .map(ToOwned::to_owned)
            .collect::<HashSet<String>>();
        let all_temps = temps
            .into_iter()
            .flat_map(|t| t.into_iter())
            .collect::<HashSet<String>>();
        assert_eq!(all_temps, exp_all_temps);

        let exp_masks = [
            ("a x1 x2 b", "001101100"),
            ("a x2 b", "001100"),
            ("a x3 b", "001100"),
            ("a x4 b", "001100"),
            ("c x1 d", "001100"),
            ("c x2 d", "001100"),
            ("c x3 d", "001100"),
            ("c x4 d", "001100"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect::<HashMap<String, String>>();
        assert_eq!(masks, exp_masks);
    }
}
