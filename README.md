# Tipping - Token Interdependency Parsing

![Tests](https://github.com/shshemi/tipping/actions/workflows/CI.yml/badge.svg)
[![PyPI version](https://badge.fury.io/py/tipping.svg)](https://badge.fury.io/py/tipping)

Tipping is a high-performance and flexible log parsing library. It leverages the interdependencies between tokens to cluster them and computes their templates and parameter masks. It is built with speed and efficiency in mind, capable of utilizing all available processor cores to accelerate the parsing process. At its core, Tipping is written in Rust to ensure maximum performance and stability while offering Python bindings for ease of use and integration into Log Analysis research and projects.

## Features

* 🚀 **High Performance**: Optimized to be fast and memory efficient.
* 🐍 **Python API**: Easy to use, compatible with python 3.8 and newer.
* 🔧 **Customizable**: Many parameters to manipulate and optimize the process.
* 🦀 **Rust-powered Core**: Safe, concurrent, and stable.

## Installation
Tipping could be installed from PyPI:
```bash
pip install tipping
```
## Usage
Load your log messages into a list of strings (`List[str]`) and:
```python
import tipping

# Example usage
messages = ["message1", "message2", ...]
clusters, masks, templates = tipping.parse(messages)

print(result)
```

## Details
Tipping offers the following parameters to manipulate and optimize the process:
```python
tipping.parse(
    messages: List[str],
    threshold: float = 0.5,
    special_whites: List[str] = None,
    special_blacks: List[str] = None,
    symbols: str = "()[]{}=,*",
    keep_alphabetic: bool = True,
    keep_numeric: bool = False,
    keep_impure: bool = False,
    return_templates: bool = True,
    return_masks: bool = True,
)
```

## Cite
```bibtex
will be filled upon publication
```