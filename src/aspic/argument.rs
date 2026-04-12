//! Argument tree construction from knowledge base and rule set.
//!
//! An ASPIC+ argument is a tree: leaves are premises, internal nodes are
//! rule applications. We construct arguments by forward chaining from the
//! knowledge base, applying rules whose premises are all derived.
//!
//! **Cyclic rule sets are rejected up front.** Forward chaining over rule
//! cycles (`p ⇒ p`, or `p ⇒ q, q ⇒ p`) produces infinite argument sequences
//! with fresh ids at each iteration: even though each argument tree has a
//! unique `(rule_id, sub_args)` tuple and the inner `already_exists` check
//! catches duplicates *at the same depth*, a genuine cycle keeps producing
//! deeper trees indefinitely (`A0: p → A1: p via rule on A0 → A2: p via
//! rule on A1 → ...`). The `already_exists` guard does not save us here,
//! so we detect rule-dependency cycles by DFS before chaining starts and
//! return [`crate::Error::Aspic`] if any are found.

use super::kb::{KnowledgeBase, Premise};
use super::language::Literal;
use super::rules::{Rule, RuleId};
use std::collections::{HashMap, HashSet};

/// A unique argument id within a `StructuredSystem`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ArgumentId(pub usize);

/// An ASPIC+ argument.
#[derive(Debug, Clone)]
pub struct Argument {
    /// This argument's id.
    pub id: ArgumentId,
    /// The conclusion of the argument (the literal it supports).
    pub conclusion: Literal,
    /// The origin of the argument: a premise (leaf) or a rule application.
    pub origin: Origin,
    /// The ids of sub-arguments this argument depends on.
    pub sub_arguments: Vec<ArgumentId>,
}

/// How an argument was constructed.
#[derive(Debug, Clone)]
pub enum Origin {
    /// A leaf argument: a premise taken directly from the knowledge base.
    Premise(Premise),
    /// An internal argument: a rule applied to sub-arguments.
    RuleApplication(RuleId),
}

impl Argument {
    /// Whether this argument is a premise leaf.
    pub fn is_premise(&self) -> bool {
        matches!(self.origin, Origin::Premise(_))
    }

    /// Whether this argument ends in a defeasible rule application.
    pub fn top_rule_is_defeasible(&self, rules: &[Rule]) -> bool {
        if let Origin::RuleApplication(rid) = &self.origin {
            rules
                .iter()
                .find(|r| r.id == *rid)
                .map(|r| r.is_defeasible())
                .unwrap_or(false)
        } else {
            false
        }
    }
}

/// Detect cyclic rule dependencies via DFS.
///
/// Builds a directed graph where `r -> s` iff `r`'s premises contain `s`'s
/// conclusion (i.e., applying `s` is a prerequisite for applying `r`).
/// Returns `true` if the graph contains a cycle.
fn rule_set_has_cycle(rules: &[Rule]) -> bool {
    let mut deps: HashMap<RuleId, Vec<RuleId>> = HashMap::new();
    for r in rules {
        let r_deps: Vec<RuleId> = rules
            .iter()
            .filter(|s| r.premises.contains(&s.conclusion))
            .map(|s| s.id)
            .collect();
        deps.insert(r.id, r_deps);
    }
    let mut visited: HashSet<RuleId> = HashSet::new();
    let mut on_stack: HashSet<RuleId> = HashSet::new();
    for r in rules {
        if !visited.contains(&r.id) && dfs_has_cycle(r.id, &deps, &mut visited, &mut on_stack) {
            return true;
        }
    }
    false
}

fn dfs_has_cycle(
    node: RuleId,
    deps: &HashMap<RuleId, Vec<RuleId>>,
    visited: &mut HashSet<RuleId>,
    on_stack: &mut HashSet<RuleId>,
) -> bool {
    visited.insert(node);
    on_stack.insert(node);
    if let Some(neighbors) = deps.get(&node) {
        for &next in neighbors {
            if !visited.contains(&next) {
                if dfs_has_cycle(next, deps, visited, on_stack) {
                    return true;
                }
            } else if on_stack.contains(&next) {
                return true;
            }
        }
    }
    on_stack.remove(&node);
    false
}

/// Build all possible arguments from a knowledge base and rule set by
/// forward chaining to a fixed point.
///
/// Returns [`crate::Error::Aspic`] if the rule set contains a dependency
/// cycle (which would prevent termination).
pub fn construct_arguments(
    kb: &KnowledgeBase,
    rules: &[Rule],
) -> Result<Vec<Argument>, crate::Error> {
    if rule_set_has_cycle(rules) {
        return Err(crate::Error::Aspic(
            "cyclic rule dependencies detected; forward chaining would not terminate".into(),
        ));
    }
    let mut arguments: Vec<Argument> = Vec::new();
    let mut next_id = 0usize;
    // Step 1: every premise is a leaf argument.
    for p in kb.premises() {
        arguments.push(Argument {
            id: ArgumentId(next_id),
            conclusion: p.literal().clone(),
            origin: Origin::Premise(p.clone()),
            sub_arguments: Vec::new(),
        });
        next_id += 1;
    }
    // Step 2: iterate forward chaining until no new arguments are added.
    loop {
        let before = arguments.len();
        // Index arguments by their conclusion for quick lookup.
        let mut by_concl: HashMap<Literal, Vec<ArgumentId>> = HashMap::new();
        for a in &arguments {
            by_concl.entry(a.conclusion.clone()).or_default().push(a.id);
        }
        // For each rule, try every combination of sub-arguments matching premises.
        for rule in rules {
            let sub_options: Vec<&[ArgumentId]> = rule
                .premises
                .iter()
                .map(|p| by_concl.get(p).map(|v| v.as_slice()).unwrap_or(&[]))
                .collect();
            if sub_options.iter().any(|v| v.is_empty()) {
                continue;
            }
            // Cartesian product across premise positions.
            let combos = cartesian(&sub_options);
            for combo in combos {
                let already_exists = arguments.iter().any(|existing| {
                    matches!(&existing.origin, Origin::RuleApplication(rid) if *rid == rule.id)
                        && existing.sub_arguments == combo
                });
                if already_exists {
                    continue;
                }
                arguments.push(Argument {
                    id: ArgumentId(next_id),
                    conclusion: rule.conclusion.clone(),
                    origin: Origin::RuleApplication(rule.id),
                    sub_arguments: combo,
                });
                next_id += 1;
            }
        }
        if arguments.len() == before {
            return Ok(arguments);
        }
    }
}

fn cartesian(options: &[&[ArgumentId]]) -> Vec<Vec<ArgumentId>> {
    let mut result = vec![Vec::new()];
    for opt in options {
        let mut next = Vec::new();
        for prefix in &result {
            for &id in *opt {
                let mut new_prefix = prefix.clone();
                new_prefix.push(id);
                next.push(new_prefix);
            }
        }
        result = next;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aspic::rules::{Rule, RuleId};

    #[test]
    fn single_premise_yields_leaf_argument() {
        let mut kb = KnowledgeBase::new();
        kb.add_ordinary(Literal::atom("p"));
        let args = construct_arguments(&kb, &[]).unwrap();
        assert_eq!(args.len(), 1);
        assert!(args[0].is_premise());
        assert_eq!(args[0].conclusion, Literal::atom("p"));
    }

    #[test]
    fn rule_application_creates_compound_argument() {
        let mut kb = KnowledgeBase::new();
        kb.add_ordinary(Literal::atom("p"));
        let rules = vec![Rule::defeasible(
            RuleId(0),
            vec![Literal::atom("p")],
            Literal::atom("q"),
        )];
        let args = construct_arguments(&kb, &rules).unwrap();
        assert_eq!(args.len(), 2);
        let q_arg = args.iter().find(|a| a.conclusion == Literal::atom("q")).unwrap();
        assert!(matches!(q_arg.origin, Origin::RuleApplication(RuleId(0))));
        assert_eq!(q_arg.sub_arguments.len(), 1);
    }

    #[test]
    fn missing_premise_blocks_rule() {
        let mut kb = KnowledgeBase::new();
        kb.add_ordinary(Literal::atom("p"));
        let rules = vec![Rule::defeasible(
            RuleId(0),
            vec![Literal::atom("r")],
            Literal::atom("q"),
        )];
        let args = construct_arguments(&kb, &rules).unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].conclusion, Literal::atom("p"));
    }

    #[test]
    fn cyclic_rules_are_rejected() {
        let mut kb = KnowledgeBase::new();
        kb.add_ordinary(Literal::atom("p"));
        let rules = vec![Rule::defeasible(
            RuleId(0),
            vec![Literal::atom("p")],
            Literal::atom("p"),
        )];
        let result = construct_arguments(&kb, &rules);
        assert!(matches!(result, Err(crate::Error::Aspic(_))));
    }

    #[test]
    fn indirect_cyclic_rules_are_rejected() {
        let mut kb = KnowledgeBase::new();
        kb.add_ordinary(Literal::atom("p"));
        let rules = vec![
            Rule::defeasible(RuleId(0), vec![Literal::atom("p")], Literal::atom("q")),
            Rule::defeasible(RuleId(1), vec![Literal::atom("q")], Literal::atom("p")),
        ];
        let result = construct_arguments(&kb, &rules);
        assert!(matches!(result, Err(crate::Error::Aspic(_))));
    }
}
