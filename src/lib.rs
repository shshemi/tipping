use std::collections::BTreeSet;
use std::collections::{HashMap, HashSet};

use interdependency::build_token_occurance;
use pyo3::prelude::*;
use rayon::prelude::*;
use regex::Regex;
use tokenizer::Token;
use tokenizer::Tokenizer;
use traits::IntoKeyNodes;

use crate::interdependency::build_interdependency_graph;
use crate::interdependency::key_node_values;

mod interdependency;
mod tokenizer;
mod traits;

#[pyclass]
#[derive(Debug, Clone)]
struct TokenFilter {
    alphabetic: bool,
    numeric: bool,
    impure: bool,
}

#[pymethods]
impl TokenFilter {
    #[new]
    fn new(alphabetic: bool, numeric: bool, impure: bool) -> Self {
        Self {
            alphabetic,
            numeric,
            impure,
        }
    }
}
/// Formats the sum of two numbers as string.
#[pyfunction]
fn token_independency_clusters(
    messages: Vec<String>,
    threshold: f32,
    special_whites: Vec<String>,
    special_blacks: Vec<String>,
    symbols: String,
    filter: TokenFilter,
) -> PyResult<Vec<(Vec<String>, HashSet<usize>)>> {
    let special_blacks = special_blacks
        .into_iter()
        .map(|re| Regex::new(re.as_str()).unwrap())
        .collect();
    let special_whites = special_whites
        .into_iter()
        .map(|re| Regex::new(re.as_str()).unwrap())
        .collect();
    let symbols = symbols.chars().collect();
    let tokenizer = Tokenizer::new(special_whites, special_blacks, symbols);
    let occurance = build_token_occurance(&messages, &tokenizer, |tok| match tok {
        Token::Alphabetic(_) => filter.alphabetic,
        Token::Numeric(_) => filter.numeric,
        Token::Symbolic(_) => false,
        Token::Whitespace(_) => false,
        Token::Impure(_) => filter.impure,
        Token::SpecialWhite(_) => true,
        Token::SpecialBlack(_) => false,
    });
    println!("occurance length: {}", occurance.len());
    let c = messages
        .iter()
        .enumerate()
        .par_bridge()
        .map(|(idx, msg)| {
            let toks = tokenizer.tokenize(msg);
            let igraph = build_interdependency_graph(&toks, &occurance, threshold);
            let mut key_nodes = igraph.into_key_nodes().unwrap_or_default();
            for tok in &toks {
                match tok {
                    Token::SpecialWhite(slice) => {
                        key_nodes.insert(slice);
                    }
                    Token::SpecialBlack(slice) => {
                        key_nodes.remove(slice);
                    }
                    _ => (),
                }
            }

            (idx, key_nodes)
        })
        .fold_with(
            HashMap::<BTreeSet<&str>, HashSet<usize>>::new(),
            |mut map, (idx, key_tokens)| {
                map.entry(key_tokens)
                    .and_modify(|indices| {
                        indices.insert(idx);
                    })
                    .or_insert([idx].into());
                map
            },
        )
        .reduce_with(|mut m1, m2| {
            m2.into_iter().for_each(|(k, v2)| {
                if let Some(v1) = m1.get_mut(&k) {
                    v1.extend(v2);
                } else {
                    m1.insert(k, v2);
                }
            });
            m1
        })
        .unwrap()
        .into_iter()
        .map(|(k, v)| (k.into_iter().map(ToOwned::to_owned).collect::<Vec<_>>(), v))
        .collect();
    // todo!()
    Ok(c)
}

/// A Python module implemented in Rust.
#[pymodule]
#[pyo3(name = "_lib_tipping")]
fn tipping(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(token_independency_clusters, m)?)?;
    m.add_class::<TokenFilter>()?;
    Ok(())
}
