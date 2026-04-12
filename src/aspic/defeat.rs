//! `StructuredSystem`: the top-level ASPIC+ entry point.
//!
//! Holds a knowledge base, a rule set, and a rule preference ordering,
//! then constructs arguments, computes attacks, resolves defeats via
//! the last-link principle, and emits a Dung AF.

use super::argument::{Argument, ArgumentId, Origin, construct_arguments};
use super::attacks::{Attack, AttackKind, compute_attacks};
use super::kb::KnowledgeBase;
use super::language::Literal;
use super::rules::{Rule, RuleId};
use crate::framework::ArgumentationFramework;
use std::collections::HashSet;

/// An ASPIC+ structured argumentation system.
#[derive(Debug, Default)]
pub struct StructuredSystem {
    kb: KnowledgeBase,
    rules: Vec<Rule>,
    /// `(preferred, less_preferred)` pairs. The public preference ordering is
    /// the transitive closure of this list, computed on demand in `is_preferred`.
    preferences: Vec<(RuleId, RuleId)>,
    next_rule_id: usize,
}

/// The combined output of building a `StructuredSystem`: constructed arguments,
/// computed attacks, and the resulting abstract framework. Returned by
/// [`StructuredSystem::build_framework`] so consumers can get all three
/// from a single forward-chaining pass.
#[derive(Debug)]
pub struct BuildOutput {
    /// All arguments constructed from the knowledge base and rule set.
    pub arguments: Vec<Argument>,
    /// All attacks between those arguments (before defeat resolution).
    pub attacks: Vec<Attack>,
    /// The abstract framework with edges for defeats (post-preference).
    pub framework: ArgumentationFramework<ArgumentId>,
}

impl StructuredSystem {
    /// Create a new empty system.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mutable access to the knowledge base.
    pub fn kb_mut(&mut self) -> &mut KnowledgeBase {
        &mut self.kb
    }

    /// Convenience forwarder for [`KnowledgeBase::add_necessary`].
    pub fn add_necessary(&mut self, l: Literal) {
        self.kb.add_necessary(l);
    }

    /// Convenience forwarder for [`KnowledgeBase::add_ordinary`].
    pub fn add_ordinary(&mut self, l: Literal) {
        self.kb.add_ordinary(l);
    }

    /// Add a strict rule, returning its id.
    pub fn add_strict_rule(&mut self, premises: Vec<Literal>, conclusion: Literal) -> RuleId {
        let id = RuleId(self.next_rule_id);
        self.next_rule_id += 1;
        self.rules.push(Rule::strict(id, premises, conclusion));
        id
    }

    /// Add a defeasible rule, returning its id.
    pub fn add_defeasible_rule(&mut self, premises: Vec<Literal>, conclusion: Literal) -> RuleId {
        let id = RuleId(self.next_rule_id);
        self.next_rule_id += 1;
        self.rules.push(Rule::defeasible(id, premises, conclusion));
        id
    }

    /// Add an *undercut* rule targeting the defeasible rule `target`.
    ///
    /// This is the safe way to construct an undercut: it encodes the
    /// conclusion as the reserved literal `¬__applicable_<target>`, which
    /// [`super::attacks::compute_attacks`] recognises.
    /// Consumers should never build this literal by hand — the `__applicable_`
    /// prefix is reserved and must not be used in user atom names.
    pub fn add_undercut_rule(&mut self, target: RuleId, premises: Vec<Literal>) -> RuleId {
        let conclusion = Literal::neg(format!("__applicable_{}", target.0));
        self.add_defeasible_rule(premises, conclusion)
    }

    /// Record that rule `preferred` is (directly) preferred to rule `less_preferred`.
    ///
    /// The effective preference ordering is the transitive closure of these
    /// pairs, computed on demand in `is_preferred`: a chain of direct
    /// preferences `r3 > r2 > r1` implies `r3 > r1`.
    pub fn prefer_rule(&mut self, preferred: RuleId, less_preferred: RuleId) {
        self.preferences.push((preferred, less_preferred));
    }

    /// Read-side accessor for the knowledge base (for debugging/visualization).
    pub fn kb(&self) -> &KnowledgeBase {
        &self.kb
    }

    /// Read-side accessor for the rule set.
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// Read-side accessor for the direct preference pairs (not transitively closed).
    pub fn preferences(&self) -> &[(RuleId, RuleId)] {
        &self.preferences
    }

    /// Whether rule `a` is strictly preferred to rule `b` under the transitive
    /// closure of recorded preferences.
    fn is_preferred(&self, a: RuleId, b: RuleId) -> bool {
        // Self-preference is not a valid strict preference.
        if a == b {
            return false;
        }
        // BFS from `a` following preferred-to edges; return true if we reach `b`.
        let mut visited: HashSet<RuleId> = HashSet::new();
        let mut frontier: Vec<RuleId> = vec![a];
        while let Some(current) = frontier.pop() {
            if !visited.insert(current) {
                continue;
            }
            for (p, lp) in &self.preferences {
                if *p == current {
                    if *lp == b {
                        return true;
                    }
                    frontier.push(*lp);
                }
            }
        }
        false
    }

    /// Collect the "last-link frontier": all defeasible rules that are the
    /// final defeasible step on some path from a premise to this argument's
    /// conclusion. Per Modgil & Prakken 2014 §3.4.
    ///
    /// - If the top rule is defeasible, the frontier is `{top_rule}`.
    /// - If the top rule is strict, the frontier is the union of the frontiers
    ///   of the sub-arguments (skipping strict rules in search of the deepest
    ///   defeasible step).
    /// - If the argument is a premise leaf, the frontier is empty.
    fn last_defeasible_frontier(&self, arg: &Argument, args: &[Argument]) -> Vec<RuleId> {
        match &arg.origin {
            Origin::Premise(_) => Vec::new(),
            Origin::RuleApplication(rid) => {
                if let Some(rule) = self.rules.iter().find(|r| r.id == *rid)
                    && rule.is_defeasible()
                {
                    return vec![*rid];
                }
                // Strict top: recurse into sub-arguments.
                let mut result = Vec::new();
                for sub_id in &arg.sub_arguments {
                    if let Some(sub) = args.iter().find(|a| a.id == *sub_id) {
                        result.extend(self.last_defeasible_frontier(sub, args));
                    }
                }
                result
            }
        }
    }

    /// Whether an attack succeeds as a defeat under the last-link principle
    /// with Elitist preference comparison (Modgil & Prakken 2014 §5.2).
    ///
    /// - Undercut always succeeds (regardless of preferences).
    /// - Rebut and undermine succeed unless the target's last-link frontier
    ///   contains a rule that is strictly preferred to *every* rule in the
    ///   attacker's last-link frontier (Elitist ordering).
    ///
    /// For single-rule frontiers (the common case) this collapses to a direct
    /// pairwise comparison `target_rule > attacker_rule`.
    fn is_defeat(&self, attack: &Attack, args: &[Argument]) -> bool {
        if attack.kind == AttackKind::Undercut {
            return true;
        }
        let attacker = args
            .iter()
            .find(|a| a.id == attack.attacker)
            .expect("attack references nonexistent attacker argument");
        let target = args
            .iter()
            .find(|a| a.id == attack.target)
            .expect("attack references nonexistent target argument");
        let attacker_rules = self.last_defeasible_frontier(attacker, args);
        let target_rules = self.last_defeasible_frontier(target, args);
        if attacker_rules.is_empty() || target_rules.is_empty() {
            // If either side has no defeasible rules, preferences can't fire;
            // the attack succeeds as a defeat.
            return true;
        }
        // Elitist: target beats attacker iff SOME target rule is strictly
        // preferred to EVERY attacker rule.
        let target_beats_attacker = target_rules
            .iter()
            .any(|tr| attacker_rules.iter().all(|ar| self.is_preferred(*tr, *ar)));
        !target_beats_attacker
    }

    /// Single-pass construction: build all arguments, compute all attacks,
    /// resolve defeats, and emit a framework. Prefer this over calling
    /// [`Self::to_framework`] and [`Self::arguments`] separately — each of
    /// those does its own forward-chaining pass.
    pub fn build_framework(&self) -> Result<BuildOutput, crate::Error> {
        let arguments = construct_arguments(&self.kb, &self.rules)?;
        let attacks = compute_attacks(&arguments, &self.rules);
        let mut framework = ArgumentationFramework::new();
        for arg in &arguments {
            framework.add_argument(arg.id);
        }
        for attack in &attacks {
            if self.is_defeat(attack, &arguments) {
                framework.add_attack(&attack.attacker, &attack.target)?;
            }
        }
        Ok(BuildOutput {
            arguments,
            attacks,
            framework,
        })
    }

    /// Construct arguments, compute attacks, resolve defeats, and emit an AF.
    ///
    /// Convenience wrapper over [`Self::build_framework`] that discards the
    /// constructed arguments and attacks. Consumers that need those should
    /// call `build_framework` directly.
    pub fn to_framework(&self) -> Result<ArgumentationFramework<ArgumentId>, crate::Error> {
        Ok(self.build_framework()?.framework)
    }

    /// Expose constructed arguments. Convenience wrapper over
    /// [`Self::build_framework`]; see its docs for performance notes.
    pub fn arguments(&self) -> Result<Vec<Argument>, crate::Error> {
        Ok(self.build_framework()?.arguments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aspic::language::Literal;

    #[test]
    fn penguin_example_resolves_correctly() {
        let mut system = StructuredSystem::new();
        system.kb_mut().add_ordinary(Literal::atom("penguin"));
        system.add_strict_rule(vec![Literal::atom("penguin")], Literal::atom("bird"));
        let r1 = system.add_defeasible_rule(vec![Literal::atom("bird")], Literal::atom("flies"));
        let r2 = system.add_defeasible_rule(vec![Literal::atom("penguin")], Literal::neg("flies"));
        system.prefer_rule(r2, r1);

        let built = system.build_framework().unwrap();
        let preferred = built.framework.preferred_extensions().unwrap();
        assert_eq!(preferred.len(), 1);
        let ext = &preferred[0];
        let flies_arg = built
            .arguments
            .iter()
            .find(|a| a.conclusion == Literal::atom("flies"));
        let not_flies_arg = built
            .arguments
            .iter()
            .find(|a| a.conclusion == Literal::neg("flies"));
        if let (Some(f), Some(nf)) = (flies_arg, not_flies_arg) {
            assert!(!ext.contains(&f.id));
            assert!(ext.contains(&nf.id));
        }
    }

    #[test]
    fn transitive_preferences_are_respected() {
        // Recorded: r3 > r2, r2 > r1. Expected: r3 > r1 via transitive closure.
        let mut system = StructuredSystem::new();
        system.kb_mut().add_ordinary(Literal::atom("p"));
        system.kb_mut().add_ordinary(Literal::atom("q"));
        system.kb_mut().add_ordinary(Literal::atom("r"));
        let r1 = system.add_defeasible_rule(vec![Literal::atom("p")], Literal::atom("x"));
        let r2 = system.add_defeasible_rule(vec![Literal::atom("q")], Literal::atom("y"));
        let r3 = system.add_defeasible_rule(vec![Literal::atom("r")], Literal::atom("z"));
        system.prefer_rule(r3, r2);
        system.prefer_rule(r2, r1);
        assert!(
            system.is_preferred(r3, r1),
            "transitive r3 > r1 should hold"
        );
    }

    #[test]
    fn undercut_helper_constructs_reserved_literal() {
        let mut system = StructuredSystem::new();
        system.kb_mut().add_ordinary(Literal::atom("p"));
        let target = system.add_defeasible_rule(vec![Literal::atom("p")], Literal::atom("q"));
        system.kb_mut().add_ordinary(Literal::atom("trigger"));
        let uc = system.add_undercut_rule(target, vec![Literal::atom("trigger")]);
        let uc_rule = system.rules().iter().find(|r| r.id == uc).unwrap();
        assert_eq!(
            uc_rule.conclusion,
            Literal::neg(format!("__applicable_{}", target.0))
        );
    }

    #[test]
    fn read_side_getters_expose_state() {
        let mut system = StructuredSystem::new();
        system.kb_mut().add_ordinary(Literal::atom("p"));
        let r1 = system.add_defeasible_rule(vec![Literal::atom("p")], Literal::atom("q"));
        let r2 = system.add_defeasible_rule(vec![Literal::atom("p")], Literal::atom("r"));
        assert_eq!(system.kb().premises().len(), 1);
        assert_eq!(system.rules().len(), 2);
        assert!(system.preferences().is_empty());
        system.prefer_rule(r1, r2);
        assert_eq!(system.preferences().len(), 1);
    }

    #[test]
    fn empty_system_produces_empty_framework() {
        let system = StructuredSystem::new();
        let af = system.to_framework().unwrap();
        assert_eq!(af.arguments().count(), 0);
    }

    #[test]
    fn elitist_single_rule_frontier_still_works() {
        // Regression test for the quantifier change from ∃/∃ to Elitist ∃/∀.
        // With single-rule frontiers on both sides, Elitist collapses to the
        // same result as the old ∃/∃ code: strong rule's argument beats weak
        // rule's argument. This test pins that behavior so future quantifier
        // experiments don't silently break the penguin case.
        let mut system = StructuredSystem::new();
        system.kb_mut().add_ordinary(Literal::atom("p"));
        system.kb_mut().add_ordinary(Literal::atom("q"));
        let r_strong = system.add_defeasible_rule(vec![Literal::atom("p")], Literal::atom("x"));
        let r_weak = system.add_defeasible_rule(vec![Literal::atom("q")], Literal::neg("x"));
        system.prefer_rule(r_strong, r_weak);

        let built = system.build_framework().unwrap();
        let preferred = built.framework.preferred_extensions().unwrap();
        assert_eq!(preferred.len(), 1);
        let ext = &preferred[0];
        let x_arg = built
            .arguments
            .iter()
            .find(|a| a.conclusion == Literal::atom("x"))
            .unwrap();
        let nx_arg = built
            .arguments
            .iter()
            .find(|a| a.conclusion == Literal::neg("x"))
            .unwrap();
        assert!(ext.contains(&x_arg.id));
        assert!(!ext.contains(&nx_arg.id));
    }
}
