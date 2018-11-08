use std::cmp::min;
use std::collections::{HashMap, HashSet};

use crate::dfs::{Dfs, Status, Visit};
use crate::Graph;

struct NodeState {
    on_stack: bool,
    index: usize,
    lowlink: usize,
}

impl NodeState {
    fn new(index: usize) -> NodeState {
        NodeState {
            on_stack: true,
            index: index,
            lowlink: index,
        }
    }
}

struct Tarjan<'a, G: Graph<'a> + ?Sized> {
    dfs: Dfs<'a, G>,
    stack: Vec<G::Node>,
    node_states: HashMap<G::Node, NodeState>,
    next_index: usize,
}

impl<'a, G: Graph<'a> + ?Sized> Tarjan<'a, G> {
    fn run(mut self) -> Decomposition<'a, G> {
        let mut ret = Vec::new();

        for visit in self.dfs {
            match visit {
                Visit::Retreat { u, parent } => {
                    let lowlink = self.node_states[&u].lowlink;
                    let index = self.node_states[&u].index;

                    if let Some(p) = parent {
                        self.node_states
                            .entry(p)
                            .and_modify(|s| s.lowlink = min(s.lowlink, lowlink));
                    }

                    if lowlink == index {
                        // u is the root of a strongly connected component, which consists of all
                        // the nodes that are above u in the stack.
                        let mut scc = HashSet::new();
                        loop {
                            // The unwrap is ok here: when we start the loop, u is guaranteed to be
                            // in the stack. Since we stop the loop whenever we find u, we're
                            // guaranteed never to run out of stack.
                            let v = self.stack.pop().unwrap();
                            scc.insert(v.clone());
                            if v == u {
                                break;
                            }
                        }
                        ret.push(scc);
                    }
                }
                Visit::Root(u) => {
                    self.stack.push(u.clone());
                    self.node_states.insert(u, NodeState::new(self.next_index));
                    self.next_index += 1;
                }
                Visit::Edge { src, dst, status } => {
                    if status == Status::New {
                        // The DFS is about to recurse on the destination node, so we'll update our
                        // state to reflect that.
                        self.stack.push(dst.clone());
                        self.node_states
                            .insert(dst, NodeState::new(self.next_index));
                        self.next_index += 1;
                    } else if self.node_states[&dst].on_stack {
                        // The fact that dst is on the stack implies that there is a path from dst
                        // to src.
                        let index = self.node_states[&dst].index;
                        self.node_states
                            .entry(src)
                            .and_modify(|s| s.lowlink = min(s.lowlink, index));
                    }
                }
            }
        }

        ret.reverse();
        Decomposition { sccs: ret }
    }
}

/// The output of Tarjan's algorithm.
///
/// Tarjan's algorithm decomposes a directed graph into strongly connected components.  Moreover,
/// those components are ordered topologically.
pub struct Decomposition<'a, G: Graph<'a> + ?Sized> {
    sccs: Vec<HashSet<G::Node>>,
}

impl<'a, G: Graph<'a> + ?Sized> Decomposition<'a, G> {
    pub(crate) fn from_graph(g: &'a G) -> Decomposition<G> {
        let tj = Tarjan {
            dfs: g.dfs(),
            stack: Vec::new(),
            node_states: HashMap::new(),
            next_index: 0,
        };
        tj.run()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::graph;
    use crate::Graph;

    macro_rules! tarjan_test {
        ($name:ident, $graph:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let g = graph($graph);
                let d = g.tarjan();
                let expected: Vec<_> = $expected
                    .into_iter()
                    .map(|scc| scc.into_iter().cloned().collect::<HashSet<u32>>())
                    .collect();
                assert_eq!(d.sccs, expected);
            }
        };
    }

    tarjan_test!(triangle, "0-1, 1-2, 2-0", [[0, 1, 2]]);
    tarjan_test!(
        two_triangles,
        "0-1, 1-2, 2-0, 2-3, 3-4, 4-5, 5-3",
        [[0, 1, 2], [3, 4, 5]]
    );
}
