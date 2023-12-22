use std::collections::BTreeSet;
use std::collections::{HashMap, HashSet};

use pyo3::prelude::*;
use rayon::prelude::*;
use regex::Regex;
use template::{common_words, parameter_masks, templates};
use tokenizer::MessageToken;
use tokenizer::Tokenizer;


use interdependency::Interdependency;

mod interdependency;
mod template;
mod tokenizer;

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

#[pyclass]
#[derive(Debug, Clone)]
pub struct Computations {
    template: bool,
    mask: bool,
}

#[pymethods]
impl Computations {
    #[new]
    fn new(template: bool, mask: bool) -> Self {
        Self { mask, template }
    }
}

type MessageClusters = Vec<Option<usize>>;
type ParameterMasks = Vec<String>;
type ClusterTemplates = Vec<HashSet<String>>;

#[pyfunction]
fn token_independency_clusters(
    messages: Vec<String>,
    threshold: f32,
    special_whites: Vec<String>,
    special_blacks: Vec<String>,
    symbols: String,
    filter: TokenFilter,
    comps: Computations,
) -> PyResult<(MessageClusters, ParameterMasks, ClusterTemplates)> {
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
    let idep = Interdependency::with(&messages, &tokenizer, |tok| match tok {
        MessageToken::Alphabetic(_) => filter.alphabetic,
        MessageToken::Numeric(_) => filter.numeric,
        MessageToken::Symbolic(_) => false,
        MessageToken::Whitespace(_) => false,
        MessageToken::Impure(_) => filter.impure,
        MessageToken::SpecialWhite(_) => true,
        MessageToken::SpecialBlack(_) => false,
    });

    let cluster_map = cluster_map(&messages, &tokenizer, &idep, threshold);

    let mut cluster_vec = vec![None; messages.len()];
    let mut mask_vec = if comps.mask {
        let mut v = vec![String::default(); messages.len()];
        if let Some(indices) = cluster_map.get(&BTreeSet::default()) {
            for idx in indices {
                let idx = *idx;
                v[idx] = "0".repeat(messages[idx].len());
            }
        }
        v
    } else {
        Default::default()
    };
    let mut template_vec = if comps.template {
        vec![HashSet::new(); cluster_map.len()]
    } else {
        Default::default()
    };
    cluster_map
        .iter()
        .filter(|(set, _)| !set.is_empty())
        .enumerate()
        .for_each(|(cid, (_, indices))| {
            if comps.mask | comps.template {
                let cw = common_words(
                    indices.iter().map(|idx| messages[*idx].as_str()),
                    &tokenizer,
                    filter.alphabetic,
                    filter.numeric,
                    filter.impure,
                );
                if comps.template {
                    template_vec[cid] = templates(
                        indices.iter().map(|idx| messages[*idx].as_str()),
                        &tokenizer,
                        &cw,
                    );
                }
                if comps.mask {
                    let msg_msk_map = parameter_masks(
                        indices.iter().map(|idx| messages[*idx].as_str()),
                        &tokenizer,
                        &cw,
                    );
                    for idx in indices {
                        cluster_vec[*idx] = Some(cid);
                        mask_vec[*idx] = msg_msk_map.get(&messages[*idx]).unwrap().to_owned();
                    }
                }
            }
            if !comps.mask {
                for idx in indices {
                    cluster_vec[*idx] = Some(cid);
                }
            }
        });

    Ok((cluster_vec, mask_vec, template_vec))
}

fn cluster_map<'a, T: AsRef<str> + Sync>(
    messages: &'a [T],
    tokenizer: &Tokenizer,
    idep: &'a Interdependency<'a>,
    threshold: f32,
) -> HashMap<BTreeSet<MessageToken<'a>>, HashSet<usize>> {
    messages
        .iter()
        .enumerate()
        .par_bridge()
        .map(|(idx, msg)| {
            // let toks = tokenizer.tokenize(msg.as_ref());
            // let igraph = idep.graph(&toks, threshold);
            // let mut key_nodes = key_tokens(igraph);

            (idx, idep.key_tokens(tokenizer.tokenize(msg.as_ref()), threshold))
        })
        .fold_with(
            HashMap::<BTreeSet<MessageToken<'a>>, HashSet<usize>>::new(),
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
        .unwrap_or_default()
}

/// A Python module implemented in Rust.
#[pymodule]
#[pyo3(name = "_lib_tipping")]
fn tipping(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(token_independency_clusters, m)?)?;
    // m.add_function(wrap_pyfunction!(mine_template, m)?)?;
    m.add_class::<TokenFilter>()?;
    m.add_class::<Computations>()?;
    Ok(())
}
