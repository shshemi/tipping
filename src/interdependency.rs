use std::collections::{BTreeSet, HashMap, HashSet};

use crate::tokenizer::{Token, Tokenizer};
use itertools::Itertools;
use petgraph::{algo::kosaraju_scc, matrix_graph::MatrixGraph};
use rayon::prelude::*;

pub type TokenCombination<'a> = BTreeSet<&'a str>;
pub type TokenOccurance<'a> = HashMap<TokenCombination<'a>, usize>;

pub struct Interdependency<'a> {
    token_occurance: TokenOccurance<'a>,
}

impl<'a> Interdependency<'a> {
    pub fn with<S, F>(msgs: &'a [S], tokenizer: &Tokenizer, token_filter: F) -> Self
    where
        S: AsRef<str> + Sync,
        F: Fn(&Token) -> bool + Sync + Copy,
    {
        Self {
            token_occurance: msgs
                .iter()
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
                .unwrap(),
        }
    }

    pub fn key_tokens(
        &self,
        tokens: Vec<Token<'a>>,
        threshold: f32,
    ) -> BTreeSet<Token<'_>> {
        let g = self.graph(&tokens, threshold);
        let scc = kosaraju_scc(&g);
        let mut key_nodes = scc
            .iter()
            .enumerate()
            .max_by_key(|(_, cc)| cc.len())
            .map(|(lcc_idx, _)| {
                let temp_toks = scc[..=lcc_idx]
                    .iter()
                    .flat_map(|v| v.iter())
                    .map(|n| g.node_weight(*n).clone())
                    .collect::<BTreeSet<_>>();
                temp_toks
            })
            .unwrap_or_default();
        for tok in tokens {
            match tok {
                Token::SpecialWhite(_) => {
                    key_nodes.insert(tok);
                }
                Token::SpecialBlack(_) => {
                    key_nodes.remove(&tok);
                }
                _ => (),
            }
        }
        key_nodes
    }

    pub fn graph(
        &self,
        tokens: &[Token<'a>],
        threshold: f32,
    ) -> MatrixGraph<Token<'a>, ()> {
        let mut graph = MatrixGraph::with_capacity(tokens.len());
        let nodes = tokens
            .iter()
            .map(|tok| graph.add_node(tok.clone()))
            .collect::<Vec<_>>();
        nodes.into_iter().combinations(2).for_each(|comb| {
            let (node1, node2) = (comb[0], comb[1]);
            if self.dependency(
                graph.node_weight(node1).as_str(),
                graph.node_weight(node2).as_str(),
            ) > threshold
            {
                graph.add_edge(node1, node2, ());
            }
            if self.dependency(
                graph.node_weight(node2).as_str(),
                graph.node_weight(node1).as_str(),
            ) > threshold
            {
                graph.add_edge(node2, node1, ());
            }
        });
        // let nodes = token.iter().map(|t| graph.add_node(**t)).collect::<Vec<_>>();
        graph
    }

    pub fn dependency(&self, word: &str, condition: &str) -> f32 {
        let double = *self
            .token_occurance
            .get(&[word, condition].into())
            .unwrap_or_else(|| panic!("Pair {:?} not found in occurances", [word, condition]));
        let single = *self
            .token_occurance
            .get(&[word].into())
            .unwrap_or_else(|| panic!("Word '{}' not found in occurances", word));
        (double as f32) / (single as f32)
    }
}
