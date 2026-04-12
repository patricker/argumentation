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

/// Defeat resolution ordering for ASPIC+.
///
/// Per M&P 2014 §3.5, two orderings are defined:
///
/// - [`DefeatOrdering::LastLink`] (default): compares arguments at the
///   last defeasible rule or, when both rule frontiers are empty, at the
///   last-premise frontier. Appropriate for legal and normative reasoning
///   where rules carry more weight than the facts they act on.
/// - [`DefeatOrdering::WeakestLink`]: compares arguments on the full set
///   of defeasible rules and ordinary premises they use. Appropriate for
///   empirical reasoning where a chain is only as strong as its weakest
///   link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefeatOrdering {
    /// Last-link ordering. Default.
    #[default]
    LastLink,
    /// Weakest-link ordering.
    WeakestLink,
}

/// An ASPIC+ structured argumentation system.
#[derive(Debug, Default)]
pub struct StructuredSystem {
    kb: KnowledgeBase,
    rules: Vec<Rule>,
    /// `(preferred, less_preferred)` pairs over defeasible rules. Effective
    /// ordering is the transitive closure.
    preferences: Vec<(RuleId, RuleId)>,
    /// `(preferred, less_preferred)` pairs over ordinary premises. Effective
    /// ordering is the transitive closure. Compared via last-premise
    /// frontier when both arguments' last-defeasible-rule frontiers are
    /// empty (per M&P 2014 Definition 3.21).
    premise_preferences: Vec<(Literal, Literal)>,
    /// Defeat resolution ordering. Default: LastLink.
    ordering: DefeatOrdering,
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
    /// Copy of the rule set used to construct the arguments. Retained
    /// for post-hoc analyses like [`Self::check_postulates`].
    pub rules: Vec<Rule>,
}

impl BuildOutput {
    /// Return the conclusions of every argument in the given extension.
    ///
    /// Useful for mapping a `HashSet<ArgumentId>` back to user-visible
    /// literal content without manually walking `self.arguments`.
    pub fn conclusions_in(&self, extension: &HashSet<ArgumentId>) -> HashSet<&Literal> {
        self.arguments
            .iter()
            .filter(|a| extension.contains(&a.id))
            .map(|a| &a.conclusion)
            .collect()
    }

    /// Return the first argument whose conclusion equals `literal`, if any.
    ///
    /// When multiple arguments share a conclusion (e.g. two rules yielding
    /// the same literal), this returns the first one in construction order.
    /// Use [`Self::arguments_with_conclusion`] if you need all of them.
    pub fn argument_by_conclusion(&self, literal: &Literal) -> Option<&Argument> {
        self.arguments.iter().find(|a| &a.conclusion == literal)
    }

    /// Return every argument whose conclusion equals `literal`.
    pub fn arguments_with_conclusion(&self, literal: &Literal) -> Vec<&Argument> {
        self.arguments
            .iter()
            .filter(|a| &a.conclusion == literal)
            .collect()
    }

    /// Check the Caminada-Amgoud rationality postulates against a given
    /// extension. Returns a [`PostulateReport`] listing any violations.
    ///
    /// A clean report (empty `violations`) means the extension satisfies
    /// sub-argument closure, closure under strict rules, direct
    /// consistency, and indirect consistency.
    pub fn check_postulates(
        &self,
        extension: &HashSet<ArgumentId>,
    ) -> super::postulates::PostulateReport {
        super::postulates::check_postulates(&self.arguments, &self.rules, extension)
    }
}

impl StructuredSystem {
    /// Create a new empty system.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new system with a specific defeat ordering.
    pub fn with_ordering(ordering: DefeatOrdering) -> Self {
        Self {
            ordering,
            ..Self::default()
        }
    }

    /// Return the currently active defeat ordering.
    pub fn ordering(&self) -> DefeatOrdering {
        self.ordering
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
        let conclusion = Literal::undercut_marker(target.0);
        self.add_defeasible_rule(premises, conclusion)
    }

    /// Record that rule `preferred` is (directly) preferred to rule `less_preferred`.
    ///
    /// The effective preference ordering is the transitive closure of these
    /// pairs, computed on demand in `is_preferred`: a chain of direct
    /// preferences `r3 > r2 > r1` implies `r3 > r1`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Aspic`] if the preference would be reflexive
    /// (`a > a`) or would create a cycle in the transitive closure (e.g.
    /// adding `r2 > r1` when `r1 > r2` is already implied). Both cases
    /// violate strict-partial-order semantics and would silently produce
    /// contradictory `is_preferred` results.
    pub fn prefer_rule(
        &mut self,
        preferred: RuleId,
        less_preferred: RuleId,
    ) -> Result<(), crate::Error> {
        if preferred == less_preferred {
            return Err(crate::Error::Aspic(format!(
                "reflexive preference rejected: rule {:?} cannot be preferred to itself",
                preferred
            )));
        }
        if self.is_preferred(less_preferred, preferred) {
            return Err(crate::Error::Aspic(format!(
                "cyclic preference rejected: rule {:?} is already (transitively) preferred to {:?}",
                less_preferred, preferred
            )));
        }
        self.preferences.push((preferred, less_preferred));
        Ok(())
    }

    /// Record that ordinary premise `preferred` is (directly) preferred to
    /// ordinary premise `less_preferred`.
    ///
    /// Per M&P 2014 Definition 3.21, premise preferences are compared
    /// under the last-link ordering only when both arguments have empty
    /// last-defeasible-rule frontiers (i.e. they are pure-premise arguments
    /// or strict-rule chains grounded in premises).
    ///
    /// Rejects reflexive `(x, x)` preferences and cyclic preferences
    /// (where `less_preferred` is already transitively preferred to
    /// `preferred`).
    pub fn prefer_premise(
        &mut self,
        preferred: Literal,
        less_preferred: Literal,
    ) -> Result<(), crate::Error> {
        if preferred == less_preferred {
            return Err(crate::Error::Aspic(format!(
                "reflexive premise preference rejected: {:?} cannot be preferred to itself",
                preferred
            )));
        }
        if self.is_premise_preferred(&less_preferred, &preferred) {
            return Err(crate::Error::Aspic(format!(
                "cyclic premise preference rejected: {:?} is already (transitively) preferred to {:?}",
                less_preferred, preferred
            )));
        }
        self.premise_preferences.push((preferred, less_preferred));
        Ok(())
    }

    /// Whether ordinary premise `a` is strictly preferred to `b` under
    /// the transitive closure of recorded premise preferences.
    pub fn is_premise_preferred(&self, a: &Literal, b: &Literal) -> bool {
        if a == b {
            return false;
        }
        let mut visited: HashSet<&Literal> = HashSet::new();
        let mut frontier: Vec<&Literal> = vec![a];
        while let Some(current) = frontier.pop() {
            if !visited.insert(current) {
                continue;
            }
            for (p, lp) in &self.premise_preferences {
                if p == current {
                    if lp == b {
                        return true;
                    }
                    frontier.push(lp);
                }
            }
        }
        false
    }

    /// Read-side accessor for the direct premise preference pairs.
    pub fn premise_preferences(&self) -> &[(Literal, Literal)] {
        &self.premise_preferences
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

    /// Collect all defeasible rules used anywhere in an argument's
    /// derivation tree (not just the last ones). Used by weakest-link
    /// defeat ordering.
    fn all_defeasible_rules(&self, arg: &Argument, args: &[Argument]) -> Vec<RuleId> {
        let mut out = Vec::new();
        if let Origin::RuleApplication(rid) = &arg.origin
            && let Some(rule) = self.rules.iter().find(|r| r.id == *rid)
            && rule.is_defeasible()
        {
            out.push(*rid);
        }
        for sub_id in &arg.sub_arguments {
            if let Some(sub) = args.iter().find(|a| a.id == *sub_id) {
                out.extend(self.all_defeasible_rules(sub, args));
            }
        }
        out
    }

    /// Collect all ordinary (defeasible) premises used anywhere in an
    /// argument's derivation tree. Used by weakest-link defeat ordering.
    fn all_ordinary_premises(&self, arg: &Argument, args: &[Argument]) -> Vec<Literal> {
        let mut out = Vec::new();
        match &arg.origin {
            Origin::Premise(p) => {
                if p.is_defeasible() {
                    out.push(p.literal().clone());
                }
            }
            Origin::RuleApplication(_) => {
                for sub_id in &arg.sub_arguments {
                    if let Some(sub) = args.iter().find(|a| a.id == *sub_id) {
                        out.extend(self.all_ordinary_premises(sub, args));
                    }
                }
            }
        }
        out
    }

    /// Collect the "last-premise frontier": all ordinary premises that
    /// lie at the leaves of this argument's derivation tree.
    ///
    /// Per M&P 2014 Definition 3.21, this frontier is compared under
    /// last-link ordering when both arguments have empty last-defeasible-rule
    /// frontiers. Necessary (indefeasible) premises are excluded because
    /// they are not subject to preference comparison — they carry the
    /// full force of the knowledge base.
    fn last_premise_frontier(&self, arg: &Argument, args: &[Argument]) -> Vec<Literal> {
        match &arg.origin {
            Origin::Premise(p) => {
                if p.is_defeasible() {
                    vec![p.literal().clone()]
                } else {
                    Vec::new()
                }
            }
            Origin::RuleApplication(_) => {
                let mut out = Vec::new();
                for sub_id in &arg.sub_arguments {
                    if let Some(sub) = args.iter().find(|a| a.id == *sub_id) {
                        out.extend(self.last_premise_frontier(sub, args));
                    }
                }
                out
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
    /// - When both sides have empty defeasible-rule frontiers, fall through
    ///   to the last-premise frontier per M&P 2014 Def 3.21.
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

        // An attack succeeds as a defeat iff the attacker is NOT strictly
        // less preferred than the target (i.e. target does not strictly
        // dominate attacker). Per M&P 2014.
        let attacker_strictly_less = match self.ordering {
            DefeatOrdering::LastLink => self.last_link_prec(attacker, target, args),
            DefeatOrdering::WeakestLink => self.weakest_link_prec(attacker, target, args),
        };
        !attacker_strictly_less
    }

    /// Elitist strict set comparison `Γ ◁Eli Γ'` per M&P 2014 Def 3.19 + 3.21 note.
    ///
    /// Returns true iff Γ is strictly less than Γ' under Elitist ordering:
    /// - If Γ = ∅: false (rule 1).
    /// - If Γ' = ∅ and Γ ≠ ∅: true (rules 1+2).
    /// - Else: ∃X ∈ Γ. ∀Y ∈ Γ'. X < Y.
    fn rule_set_strict_lt(&self, gamma: &[RuleId], gamma_prime: &[RuleId]) -> bool {
        if gamma.is_empty() {
            return false;
        }
        if gamma_prime.is_empty() {
            return true;
        }
        gamma
            .iter()
            .any(|x| gamma_prime.iter().all(|y| self.is_preferred(*y, *x)))
    }

    /// Same as [`Self::rule_set_strict_lt`] but over ordinary-premise sets.
    fn premise_set_strict_lt(&self, gamma: &[Literal], gamma_prime: &[Literal]) -> bool {
        if gamma.is_empty() {
            return false;
        }
        if gamma_prime.is_empty() {
            return true;
        }
        gamma
            .iter()
            .any(|x| gamma_prime.iter().all(|y| self.is_premise_preferred(y, x)))
    }

    /// Last-link strict preference `A ≺ B` per M&P 2014 Def 3.21.
    ///
    /// A ≺ B iff:
    /// - LastDefRules(A) ◁s LastDefRules(B), OR
    /// - Both LastDefRules are empty AND Prem(A) ◁s Prem(B).
    fn last_link_prec(&self, a: &Argument, b: &Argument, args: &[Argument]) -> bool {
        let a_rules = self.last_defeasible_frontier(a, args);
        let b_rules = self.last_defeasible_frontier(b, args);
        if self.rule_set_strict_lt(&a_rules, &b_rules) {
            return true;
        }
        if a_rules.is_empty() && b_rules.is_empty() {
            let a_prems = self.last_premise_frontier(a, args);
            let b_prems = self.last_premise_frontier(b, args);
            return self.premise_set_strict_lt(&a_prems, &b_prems);
        }
        false
    }

    /// Weakest-link strict preference `A ≺ B` per M&P 2014 Def 3.23.
    ///
    /// A ≺ B iff (case 3, the general case used when at least one side
    /// is plausible and defeasible):
    /// Premp(A) ⊴s Premp(B) AND DefRules(A) ⊴s DefRules(B),
    /// with at least one of the two being strict (`◁s`).
    ///
    /// Special cases:
    /// - Both strict (no defeasible rules on either side): compare only
    ///   premises.
    /// - Both firm (no ordinary premises on either side): compare only
    ///   defeasible rules.
    fn weakest_link_prec(&self, a: &Argument, b: &Argument, args: &[Argument]) -> bool {
        let a_rules = self.all_defeasible_rules(a, args);
        let b_rules = self.all_defeasible_rules(b, args);
        let a_prems = self.all_ordinary_premises(a, args);
        let b_prems = self.all_ordinary_premises(b, args);

        let a_strict = a_rules.is_empty() && b_rules.is_empty();
        let a_firm = a_prems.is_empty() && b_prems.is_empty();

        if a_strict {
            return self.premise_set_strict_lt(&a_prems, &b_prems);
        }
        if a_firm {
            return self.rule_set_strict_lt(&a_rules, &b_rules);
        }

        // Case 3: both plausible and defeasible. Use the ≺ form: replace
        // ⊴ with ◁ (strict) in the conjunction per the paper's note.
        self.premise_set_strict_lt(&a_prems, &b_prems)
            && self.rule_set_strict_lt(&a_rules, &b_rules)
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
            rules: self.rules.clone(),
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
        system.prefer_rule(r2, r1).unwrap();

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
        system.prefer_rule(r3, r2).unwrap();
        system.prefer_rule(r2, r1).unwrap();
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
        assert_eq!(uc_rule.conclusion, Literal::undercut_marker(target.0));
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
        system.prefer_rule(r1, r2).unwrap();
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
        system.prefer_rule(r_strong, r_weak).unwrap();

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

    #[test]
    fn prefer_rule_rejects_cyclic_preferences() {
        let mut system = StructuredSystem::new();
        system.add_ordinary(Literal::atom("p"));
        system.add_ordinary(Literal::atom("q"));
        let r1 = system.add_defeasible_rule(vec![Literal::atom("p")], Literal::atom("x"));
        let r2 = system.add_defeasible_rule(vec![Literal::atom("q")], Literal::atom("y"));
        system.prefer_rule(r1, r2).unwrap();
        // Adding r2 > r1 now would create a cycle.
        let result = system.prefer_rule(r2, r1);
        assert!(matches!(result, Err(crate::Error::Aspic(_))));
    }

    #[test]
    fn prefer_rule_rejects_reflexive_preferences() {
        let mut system = StructuredSystem::new();
        system.add_ordinary(Literal::atom("p"));
        let r1 = system.add_defeasible_rule(vec![Literal::atom("p")], Literal::atom("x"));
        let result = system.prefer_rule(r1, r1);
        assert!(matches!(result, Err(crate::Error::Aspic(_))));
    }

    #[test]
    fn build_output_conclusions_in_extension_returns_literals() {
        let mut sys = StructuredSystem::new();
        sys.add_ordinary(Literal::atom("p"));
        let _r = sys.add_defeasible_rule(vec![Literal::atom("p")], Literal::atom("q"));
        let built = sys.build_framework().unwrap();
        let grounded = built.framework.grounded_extension();
        let concls = built.conclusions_in(&grounded);
        assert!(concls.contains(&Literal::atom("p")));
        assert!(concls.contains(&Literal::atom("q")));
        assert_eq!(concls.len(), 2);
    }

    #[test]
    fn build_output_argument_by_conclusion_finds_unique_matches() {
        let mut sys = StructuredSystem::new();
        sys.add_ordinary(Literal::atom("p"));
        let _r = sys.add_defeasible_rule(vec![Literal::atom("p")], Literal::atom("q"));
        let built = sys.build_framework().unwrap();
        let q_arg = built.argument_by_conclusion(&Literal::atom("q"));
        assert!(q_arg.is_some());
        assert_eq!(q_arg.unwrap().conclusion, Literal::atom("q"));
        let missing = built.argument_by_conclusion(&Literal::atom("never"));
        assert!(missing.is_none());
    }

    #[test]
    fn premise_preferences_support_transitive_closure() {
        let mut sys = StructuredSystem::new();
        sys.add_ordinary(Literal::atom("p"));
        sys.add_ordinary(Literal::atom("q"));
        sys.add_ordinary(Literal::atom("r"));
        sys.prefer_premise(Literal::atom("p"), Literal::atom("q"))
            .unwrap();
        sys.prefer_premise(Literal::atom("q"), Literal::atom("r"))
            .unwrap();
        assert!(sys.is_premise_preferred(&Literal::atom("p"), &Literal::atom("r")));
        assert!(!sys.is_premise_preferred(&Literal::atom("r"), &Literal::atom("p")));
    }

    #[test]
    fn prefer_premise_rejects_reflexive() {
        let mut sys = StructuredSystem::new();
        sys.add_ordinary(Literal::atom("p"));
        let result = sys.prefer_premise(Literal::atom("p"), Literal::atom("p"));
        assert!(matches!(result, Err(crate::Error::Aspic(_))));
    }

    #[test]
    fn prefer_premise_rejects_cyclic() {
        let mut sys = StructuredSystem::new();
        sys.add_ordinary(Literal::atom("p"));
        sys.add_ordinary(Literal::atom("q"));
        sys.prefer_premise(Literal::atom("p"), Literal::atom("q"))
            .unwrap();
        let result = sys.prefer_premise(Literal::atom("q"), Literal::atom("p"));
        assert!(matches!(result, Err(crate::Error::Aspic(_))));
    }

    #[test]
    fn premise_preference_blocks_undermine_when_target_premise_stronger() {
        // Scenario: KB has s and u (both ordinary). Strict rule u → ¬s.
        // Argument NS = u → ¬s undermines argument S = s.
        //
        // Without preferences: NS defeats S (target frontier {s} vs attacker
        // frontier {u}, neither has rule frontier).
        //
        // With prefer_premise(s, u): NS does NOT defeat S because target's
        // last-premise frontier {s} is preferred to attacker's {u}.
        let mut sys = StructuredSystem::new();
        sys.add_ordinary(Literal::atom("s"));
        sys.add_ordinary(Literal::atom("u"));
        sys.add_strict_rule(vec![Literal::atom("u")], Literal::neg("s"));
        sys.prefer_premise(Literal::atom("s"), Literal::atom("u"))
            .unwrap();

        let built = sys.build_framework().unwrap();
        let s_arg = built
            .argument_by_conclusion(&Literal::atom("s"))
            .expect("s premise argument");
        let grounded = built.framework.grounded_extension();
        assert!(
            grounded.contains(&s_arg.id),
            "expected s to survive under premise preference s > u, got {:?}",
            grounded
        );
    }

    #[test]
    fn build_output_arguments_with_conclusion_returns_all_matches() {
        let mut sys = StructuredSystem::new();
        sys.add_ordinary(Literal::atom("a"));
        sys.add_ordinary(Literal::atom("b"));
        let _r1 = sys.add_defeasible_rule(vec![Literal::atom("a")], Literal::atom("target"));
        let _r2 = sys.add_defeasible_rule(vec![Literal::atom("b")], Literal::atom("target"));
        let built = sys.build_framework().unwrap();
        let matches = built.arguments_with_conclusion(&Literal::atom("target"));
        assert_eq!(matches.len(), 2);
    }
}
