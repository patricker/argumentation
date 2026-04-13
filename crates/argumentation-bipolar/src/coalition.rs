//! Coalition detection on the support graph.
//!
//! A **coalition** is a strongly-connected component of the support
//! graph: a set of arguments where every pair mutually supports each
//! other, directly or transitively. Singleton SCCs (arguments with no
//! mutual support) are also returned as coalitions of size 1.
//!
//! Uses petgraph's Tarjan SCC implementation, which is O(V + E).

use crate::framework::BipolarFramework;
use crate::types::CoalitionId;
use petgraph::algo::tarjan_scc;
use petgraph::graph::DiGraph;
use std::collections::HashMap;
use std::hash::Hash;

/// A detected coalition with its member arguments.
#[derive(Debug, Clone)]
pub struct Coalition<A: Clone + Eq + Hash> {
    /// Assigned identifier — stable only within a single
    /// [`detect_coalitions`] call.
    pub id: CoalitionId,
    /// The arguments in this coalition. For a singleton coalition this
    /// has exactly one element.
    pub members: Vec<A>,
}

/// Detect all coalitions in a bipolar framework.
///
/// Builds a petgraph `DiGraph` from the support edges (ignoring
/// attacks), runs Tarjan's SCC algorithm, and returns one [`Coalition`]
/// per SCC with a freshly-assigned [`CoalitionId`].
///
/// Coalition ids are assigned in the order petgraph's SCC iterator
/// returns them, which is a reverse topological order over the
/// condensation. Consumers should treat ids as opaque and use
/// [`Coalition::members`] to identify coalitions semantically.
pub fn detect_coalitions<A>(framework: &BipolarFramework<A>) -> Vec<Coalition<A>>
where
    A: Clone + Eq + Hash,
{
    let mut graph: DiGraph<A, ()> = DiGraph::new();
    let mut index: HashMap<A, petgraph::graph::NodeIndex> = HashMap::new();

    for arg in framework.arguments() {
        let idx = graph.add_node(arg.clone());
        index.insert(arg.clone(), idx);
    }
    for (sup, supd) in framework.supports() {
        let (Some(&a), Some(&b)) = (index.get(sup), index.get(supd)) else {
            continue;
        };
        graph.add_edge(a, b, ());
    }

    tarjan_scc(&graph)
        .into_iter()
        .enumerate()
        .map(|(i, component)| Coalition {
            id: CoalitionId(i as u32),
            members: component.into_iter().map(|n| graph[n].clone()).collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolated_arguments_are_singleton_coalitions() {
        let mut bf = BipolarFramework::new();
        bf.add_argument("a");
        bf.add_argument("b");
        bf.add_argument("c");
        let coalitions = detect_coalitions(&bf);
        assert_eq!(coalitions.len(), 3);
        for c in &coalitions {
            assert_eq!(c.members.len(), 1);
        }
    }

    #[test]
    fn mutual_support_produces_one_coalition_of_two() {
        let mut bf = BipolarFramework::new();
        bf.add_support("alice", "bob").unwrap();
        bf.add_support("bob", "alice").unwrap();
        let coalitions = detect_coalitions(&bf);
        // Expect one coalition {alice, bob}, no other singletons.
        assert_eq!(coalitions.len(), 1);
        assert_eq!(coalitions[0].members.len(), 2);
        assert!(coalitions[0].members.contains(&"alice"));
        assert!(coalitions[0].members.contains(&"bob"));
    }

    #[test]
    fn one_way_support_is_two_singletons() {
        // a → b support (but no b → a) is NOT a coalition under SCC.
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        let coalitions = detect_coalitions(&bf);
        assert_eq!(coalitions.len(), 2);
        for c in &coalitions {
            assert_eq!(c.members.len(), 1);
        }
    }

    #[test]
    fn attack_edges_do_not_create_coalitions() {
        let mut bf = BipolarFramework::new();
        bf.add_attack("alice", "bob");
        bf.add_attack("bob", "alice"); // mutual attack, not mutual support
        let coalitions = detect_coalitions(&bf);
        assert_eq!(coalitions.len(), 2);
    }

    #[test]
    fn three_way_mutual_support_forms_one_coalition() {
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        bf.add_support("b", "c").unwrap();
        bf.add_support("c", "a").unwrap();
        let coalitions = detect_coalitions(&bf);
        assert_eq!(coalitions.len(), 1);
        assert_eq!(coalitions[0].members.len(), 3);
    }

    #[test]
    fn coalition_ids_are_distinct() {
        let mut bf = BipolarFramework::new();
        bf.add_argument("a");
        bf.add_argument("b");
        bf.add_argument("c");
        let coalitions = detect_coalitions(&bf);
        let ids: std::collections::HashSet<_> = coalitions.iter().map(|c| c.id).collect();
        assert_eq!(ids.len(), coalitions.len());
    }
}
