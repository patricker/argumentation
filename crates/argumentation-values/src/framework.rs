//! `ValueBasedFramework` — Dung framework + value assignment, with
//! audience-conditioned defeat semantics.

use crate::error::Error;
use crate::types::{Audience, ValueAssignment};
use argumentation::ArgumentationFramework;
use std::collections::HashSet;
use std::hash::Hash;

/// A value-based argumentation framework: an underlying Dung framework
/// plus a [`ValueAssignment`] mapping arguments to the values they promote.
///
/// Acceptance is computed *per audience* — there is no audience-independent
/// notion of acceptance in a VAF. See [`Self::accepted_for`].
///
/// # Type parameter
///
/// `A` is the argument label type, matching the underlying
/// [`ArgumentationFramework<A>`]. For encounter-bridge use, this is
/// typically `argumentation::ArgumentId`; for standalone use `&'static str`
/// or `String` work fine.
#[derive(Debug, Clone)]
pub struct ValueBasedFramework<A: Clone + Eq + Hash> {
    base: ArgumentationFramework<A>,
    values: ValueAssignment<A>,
}

impl<A: Clone + Eq + Hash + Ord + std::fmt::Debug> ValueBasedFramework<A> {
    /// Construct from a Dung framework and value assignment.
    pub fn new(base: ArgumentationFramework<A>, values: ValueAssignment<A>) -> Self {
        Self { base, values }
    }

    /// Borrow the underlying Dung framework (unconditioned attacks).
    pub fn base(&self) -> &ArgumentationFramework<A> {
        &self.base
    }

    /// Borrow the value assignment.
    pub fn value_assignment(&self) -> &ValueAssignment<A> {
        &self.values
    }

    /// Build the audience-conditioned defeat graph as a fresh
    /// [`ArgumentationFramework`].
    ///
    /// An attack `(attacker, target)` in [`Self::base`] becomes a defeat
    /// in the result iff `defeats(attacker, target)` returns true under
    /// this audience.
    ///
    /// # Defeat rule (Kaci & van der Torre 2008, Pareto-defeating)
    ///
    /// Given multi-value assignments, A defeats B iff for **every** value
    /// `v_b` promoted by B, there is **some** value `v_a` promoted by A
    /// such that `v_b` is *not strictly preferred* over `v_a` under the
    /// audience. This degenerates to Bench-Capon (2003) when each
    /// argument promotes exactly one value.
    ///
    /// # Special cases
    ///
    /// - Attacker promotes no value → A defeats B (unconditional).
    /// - Target promotes no value → A defeats B (no preference can save B).
    /// - Either value is unranked in the audience → considered incomparable
    ///   (no strict preference); the attacker side wins ties.
    pub fn defeat_graph(&self, audience: &Audience) -> Result<ArgumentationFramework<A>, Error> {
        let mut result = ArgumentationFramework::new();
        let args: Vec<A> = self.base.arguments().cloned().collect();
        for arg in &args {
            result.add_argument(arg.clone());
        }
        // Iterate attacks via the per-target attackers() accessor — the
        // base framework doesn't expose a flat (attacker, target) iterator,
        // so we walk the graph one target at a time.
        for target in &args {
            let attackers: Vec<A> = self.base.attackers(target).into_iter().cloned().collect();
            for attacker in &attackers {
                if self.defeats(attacker, target, audience) {
                    result.add_attack(attacker, target)?;
                }
            }
        }
        Ok(result)
    }

    /// Returns true iff `attacker` defeats `target` under the audience.
    /// Both arguments must already be in the underlying framework's attack
    /// graph (i.e., `attacker` attacks `target` in the base); this method
    /// only filters by value preference. Calling this with non-attacking
    /// pairs is meaningless but not an error.
    pub fn defeats(&self, attacker: &A, target: &A, audience: &Audience) -> bool {
        let attacker_values = self.values.values(attacker);
        let target_values = self.values.values(target);

        // Null-promotion rule: if either side promotes no value, no value
        // preference can intervene, so the attack stands as a defeat.
        if attacker_values.is_empty() || target_values.is_empty() {
            return true;
        }

        // Pareto-defeating: for every target value, attacker has at least
        // one value that the target's value does not strictly outrank.
        target_values.iter().all(|tv| {
            attacker_values
                .iter()
                .any(|av| !audience.prefers(tv, av))
        })
    }

    /// Audience-conditioned credulous acceptance under preferred semantics.
    ///
    /// Returns `Ok(true)` iff `arg` is in *some* preferred extension of the
    /// audience-conditioned defeat graph.
    pub fn accepted_for(&self, audience: &Audience, arg: &A) -> Result<bool, Error> {
        let defeat = self.defeat_graph(audience)?;
        let extensions = defeat.preferred_extensions().map_err(Error::from)?;
        Ok(extensions.iter().any(|ext| ext.contains(arg)))
    }

    /// Audience-conditioned grounded extension (Dung 1995).
    ///
    /// Returns the unique skeptically-accepted set under the audience-conditioned
    /// defeat graph. The grounded extension is computed by the upstream
    /// `argumentation` crate via least-fixed-point of the characteristic
    /// function (not via intersection of preferred extensions, which is the
    /// ideal extension and may be a strict superset on non-well-founded
    /// frameworks).
    pub fn grounded_for(&self, audience: &Audience) -> Result<HashSet<A>, Error> {
        let defeat = self.defeat_graph(audience)?;
        Ok(defeat.grounded_extension())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    fn framework_two_args_mutual_attack() -> ValueBasedFramework<&'static str> {
        let mut base = ArgumentationFramework::new();
        base.add_argument("h1");
        base.add_argument("c1");
        base.add_attack(&"h1", &"c1").unwrap();
        base.add_attack(&"c1", &"h1").unwrap();

        let mut values = ValueAssignment::new();
        values.promote("h1", Value::new("life"));
        values.promote("c1", Value::new("property"));

        ValueBasedFramework::new(base, values)
    }

    #[test]
    fn life_audience_defeats_property_attack() {
        let vaf = framework_two_args_mutual_attack();
        let audience = Audience::total([Value::new("life"), Value::new("property")]);
        // h1 attacks c1: target value (property) NOT preferred over attacker
        // value (life), so h1 defeats c1.
        assert!(vaf.defeats(&"h1", &"c1", &audience));
        // c1 attacks h1: target value (life) IS strictly preferred over
        // attacker value (property), so c1 does NOT defeat h1.
        assert!(!vaf.defeats(&"c1", &"h1", &audience));
    }

    #[test]
    fn property_audience_inverts_defeats() {
        let vaf = framework_two_args_mutual_attack();
        let audience = Audience::total([Value::new("property"), Value::new("life")]);
        assert!(!vaf.defeats(&"h1", &"c1", &audience));
        assert!(vaf.defeats(&"c1", &"h1", &audience));
    }

    #[test]
    fn null_promotion_attacker_always_defeats() {
        let mut base = ArgumentationFramework::new();
        base.add_argument("a");
        base.add_argument("b");
        base.add_attack(&"a", &"b").unwrap();
        let mut values = ValueAssignment::new();
        values.promote("b", Value::new("life"));
        // a promotes nothing.
        let vaf = ValueBasedFramework::new(base, values);
        let audience = Audience::total([Value::new("life")]);
        assert!(vaf.defeats(&"a", &"b", &audience));
    }

    #[test]
    fn empty_audience_preserves_all_attacks() {
        let vaf = framework_two_args_mutual_attack();
        let audience = Audience::new();
        // No preferences → everything is incomparable → all attacks defeat.
        assert!(vaf.defeats(&"h1", &"c1", &audience));
        assert!(vaf.defeats(&"c1", &"h1", &audience));
    }

    #[test]
    fn defeat_graph_filters_attacks() {
        let vaf = framework_two_args_mutual_attack();
        let audience = Audience::total([Value::new("life"), Value::new("property")]);
        let defeat = vaf.defeat_graph(&audience).unwrap();
        assert_eq!(defeat.attackers(&"h1").len(), 0);
        assert_eq!(defeat.attackers(&"c1").len(), 1);
    }
}
