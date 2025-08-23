use std::{collections::BTreeSet, hash::Hash};

use itertools::Itertools;
use petgraph::algo::kosaraju_scc;
use petgraph::matrix_graph::MatrixGraph;

pub fn build_graph<T: Clone + Eq + Hash, Iter: Iterator<Item = T>>(
    token_iter: Iter,
    is_connected: impl Fn(&T, &T) -> bool,
) -> MatrixGraph<T, ()> {
    let tokens = token_iter.collect::<Vec<_>>();
    let mut graph = MatrixGraph::with_capacity(tokens.len());
    let nodes = tokens
        .iter()
        .unique()
        .cloned()
        .map(|tok| graph.add_node(tok))
        .collect::<Vec<_>>();
    nodes.iter().tuple_combinations().for_each(|(n1, n2)| {
        if is_connected(graph.node_weight(*n1), graph.node_weight(*n2)) {
            graph.add_edge(*n1, *n2, ());
        }

        if is_connected(graph.node_weight(*n2), graph.node_weight(*n1)) {
            graph.add_edge(*n2, *n1, ());
        }
    });
    graph
}
pub fn anchor_nodes<T: Clone + Eq + Hash + Ord>(g: MatrixGraph<T, ()>) -> BTreeSet<T> {
    let scc = kosaraju_scc(&g);
    let nodes = scc
        .iter()
        .max_by_key(|cc| cc.len())
        .map(|lcc| lcc.iter().map(|v| g.node_weight(*v)).cloned().collect())
        .unwrap_or_default();
    nodes
}
