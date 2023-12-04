use std::collections::{BTreeSet, HashMap, HashSet};

use crate::tokenizer::{Token, Tokenizer};
use itertools::Itertools;
use petgraph::matrix_graph::MatrixGraph;
use rayon::prelude::*;

pub type TokenCombination<'a> = BTreeSet<&'a str>;
pub type TokenOccurance<'a> = HashMap<TokenCombination<'a>, usize>;

pub fn build_token_occurance<'a, S, F>(
    msgs: &'a [S],
    tokenizer: &Tokenizer,
    token_filter: F,
) -> TokenOccurance<'a>
where
    S: AsRef<str> + Sync,
    F: Fn(&Token) -> bool + Sync + Copy,
{
    msgs.iter()
        .par_bridge()
        .fold_with(HashMap::new(), |mut map, msg| {
            let toks = tokenizer
                .tokenize(msg.as_ref())
                .into_iter()
                .filter(token_filter)
                .map(|tok| tok.as_str())
                .unique()
                .collect::<HashSet<_>>();

            // Insert single occurances
            for tok in &toks {
                map.entry([*tok].into())
                    .and_modify(|count| *count += 1)
                    .or_insert(1_usize);
            }

            // Insert double occurances
            for (tok1, tok2) in toks.iter().tuple_combinations() {
                map.entry([*tok1, *tok2].into())
                    .and_modify(|count| *count += 1)
                    .or_insert(1_usize);
            }
            map
        })
        .reduce_with(|mut map1, map2| {
            for (occ, count2) in map2 {
                map1.entry(occ)
                    .and_modify(|count1| *count1 += count2)
                    .or_insert(count2);
            }
            map1
        })
        .unwrap()
}

pub fn build_interdependency_graph<'a>(
    tokens: &[Token<'a>],
    token_occurance: &TokenOccurance,
    threshold: f32,
) -> MatrixGraph<&'a str, ()> {
    let token = tokens
        .iter()
        .map(Token::as_str)
        .filter(|slice| token_occurance.contains_key(&BTreeSet::from([*slice])))
        .collect::<Vec<_>>();
    let mut graph = MatrixGraph::with_capacity(token.len());
    let nodes = token.iter().map(|t| graph.add_node(*t)).collect::<Vec<_>>();
    nodes.iter().enumerate().combinations(2).for_each(|comb| {
        let ((i1, w1), (i2, w2)) = (comb[0], comb[1]);
        if dependency(token_occurance, token[i1], token[i2]) > threshold {
            graph.add_edge(*w1, *w2, ());
        }
        if dependency(token_occurance, token[i2], token[i1]) > threshold {
            graph.add_edge(*w2, *w1, ());
        }
    });
    graph
}

pub fn dependency(tok_occ: &TokenOccurance, word: &str, condition: &str) -> f32 {
    let double = *tok_occ.get(&[word, condition].into()).unwrap_or_else(||{panic!("Pair {:?} not found in occurances", [word, condition])});
    let single = *tok_occ.get(&[word].into()).unwrap_or_else(||panic!("Word '{}' not found in occurances", word));
    (double as f32) / (single as f32)
}