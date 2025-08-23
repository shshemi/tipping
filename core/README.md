# Tipping - Token Interdependency Parsing

![Tests](https://github.com/shshemi/tipping/actions/workflows/CI.yml/badge.svg)

Tipping is a high-performance and flexible log parsing library. It leverages a rule based tokenizer to extract subtokens and interdependencies between them to cluster log messages and predict their templates and parameter masks. It is built with speed and efficiency in mind, capable of utilizing all available processor cores to accelerate the parsing process. At its core, Tipping is written in Rust to ensure maximum performance and stability while offering [Python bindings](https://github.com/shshemi/tipping) for ease of use and integration into log analysis researches and projects.

## Installation
```bash
cargo add tipping-rs
```
## Usage
Load your log messages into a list of strings (`Vec<String>`) and:
```rust
    let msgs: Vec<String>;
    let (event_ids, masks, templates) = tipping_rs::Parser::default()
        .with_threshold(threshold)
        .with_special_whites(special_whites)
        .with_special_blacks(special_blacks)
        .with_symbols(symbols)
        .with_filter_alphabetic(filter.alphabetic)
        .with_filter_numeric(filter.numeric)
        .with_filter_impure(filter.impure)
        .compute_templates()
        .compute_masks()
        .parse(msgs);
```

## Cite
```bibtex
will be filled upon publication
```