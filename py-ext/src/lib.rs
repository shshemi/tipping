use std::collections::{HashMap, HashSet};

use fancy_regex::Regex;
use pyo3::prelude::*;
use tipping_rs::Tokenize;

#[pyclass]
pub struct Tokenizer {
    internal: tipping_rs::Tokenizer,
}

#[pymethods]
impl Tokenizer {
    #[new]
    pub fn new(special_whites: Vec<String>, special_blacks: Vec<String>, symbols: String) -> Self {
        Self {
            internal: tipping_rs::Tokenizer::new(
                special_whites
                    .into_iter()
                    .map(|pattern| {
                        Regex::new(&pattern)
                            .unwrap_or_else(|_| panic!("Unable to compile {pattern}"))
                    })
                    .collect::<Vec<_>>(),
                special_blacks
                    .into_iter()
                    .map(|pattern| {
                        Regex::new(&pattern)
                            .unwrap_or_else(|_| panic!("Unable to compile {pattern}"))
                    })
                    .collect(),
                symbols.chars().collect(),
            ),
        }
    }

    pub fn tokenize(&self, msg: String) -> Vec<String> {
        self.internal
            .tokenize(&msg)
            .into_iter()
            .map(|tok| tok.as_str().to_owned())
            .collect()
    }
}

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
    let special_blacks = special_blacks.into_iter().map(compile_regex).collect();
    let special_whites = special_whites.into_iter().map(compile_regex).collect();
    let symbols = symbols.chars().collect();

    let parser = tipping_rs::Parser::default()
        .with_threshold(threshold)
        .with_special_whites(special_whites)
        .with_special_blacks(special_blacks)
        .with_symbols(symbols)
        .with_filter_alphabetic(filter.alphabetic)
        .with_filter_numeric(filter.numeric)
        .with_filter_impure(filter.impure);
    Ok(match comps {
        Computations {
            template: false,
            mask: false,
        } => {
            let clusters = parser.parse(&messages);
            (clusters, Default::default(), Default::default())
        }
        Computations {
            template: false,
            mask: true,
        } => {
            let (clusters, masks) = parser.compute_masks().parse(&messages);
            (
                clusters,
                one_to_one_masks(&messages, masks),
                Default::default(),
            )
        }
        Computations {
            template: true,
            mask: false,
        } => {
            let (clusters, templates) = parser.compute_templates().parse(&messages);
            (clusters, Default::default(), templates)
        }

        Computations {
            template: true,
            mask: true,
        } => {
            let (clusters, templates, masks) =
                parser.compute_masks().compute_templates().parse(&messages);
            (clusters, one_to_one_masks(&messages, masks), templates)
        }
    })
}

/// A Python module implemented in Rust.
#[pymodule]
fn _lib_tipping(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(token_independency_clusters, m)?)?;
    m.add_class::<TokenFilter>()?;
    m.add_class::<Computations>()?;
    m.add_class::<Tokenizer>()?;
    Ok(())
}

fn one_to_one_masks(messages: &[String], masks: HashMap<String, String>) -> Vec<String> {
    messages
        .iter()
        .map(|msg| {
            masks
                .get(msg)
                .map(ToOwned::to_owned)
                .unwrap_or("0".repeat(msg.len()))
        })
        .collect::<Vec<_>>()
}

fn compile_regex(re: impl AsRef<str>) -> Regex {
    match Regex::new(re.as_ref()) {
        Ok(regex) => regex,
        Err(err) => panic!("Error: {}, Regex: {}", err, re.as_ref()),
    }
}
