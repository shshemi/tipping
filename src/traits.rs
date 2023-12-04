use std::collections::BTreeSet;

use petgraph::{matrix_graph::MatrixGraph, algo::kosaraju_scc};

pub trait IntoKeyNodes<'a, N: ?Sized> {
    fn into_key_nodes(self) -> Option<BTreeSet<&'a N>>;
}

impl<'a, N: ?Sized, E> IntoKeyNodes<'a, N> for MatrixGraph<&'a N, E>
where
    N: std::cmp::Ord
{
    fn into_key_nodes(self) -> Option<BTreeSet<&'a N>> {
        let scc = kosaraju_scc(&self);
        scc.iter()
            .enumerate()
            .max_by_key(|(_, cc)| cc.len())
            .map(|(lcc_idx, _)| {
                let temp_toks = scc[..=lcc_idx]
                    .iter()
                    .flat_map(|v| v.iter())
                    .map(|n| *self.node_weight(*n))
                    .collect::<BTreeSet<_>>();
                temp_toks
            })
    }
}
