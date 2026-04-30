//! Multi-audience consensus queries.
//!
//! When a scene involves multiple characters, each potentially with their
//! own value ordering, the natural query becomes: "which arguments survive
//! under *every* character's audience?" This is DiArg's AgreementScenario
//! abstraction (Kampik 2020), simplified.

use crate::error::Error;
use crate::framework::ValueBasedFramework;
use crate::types::Audience;
use std::collections::HashSet;
use std::hash::Hash;

/// Query operations over a set of audiences.
pub struct MultiAudience<'a> {
    audiences: &'a [Audience],
}

impl<'a> MultiAudience<'a> {
    /// Construct from a slice of audiences. Empty slice means "no audiences"
    /// — every argument is trivially accepted under the empty universal
    /// quantifier (`common_extensions` returns the union of arguments).
    pub fn new(audiences: &'a [Audience]) -> Self {
        Self { audiences }
    }

    /// Borrow the underlying audiences.
    pub fn audiences(&self) -> &[Audience] {
        self.audiences
    }

    /// Returns the set of arguments that are credulously accepted (i.e.,
    /// in *some* preferred extension) under *every* audience in the set.
    ///
    /// This is the consensus answer: which proposals survive regardless
    /// of which character's value ordering you adopt? Useful for council
    /// / jury / cabinet narrative queries.
    pub fn common_credulous<A>(
        &self,
        vaf: &ValueBasedFramework<A>,
    ) -> Result<HashSet<A>, Error>
    where
        A: Clone + Eq + Hash + Ord + std::fmt::Debug,
    {
        if self.audiences.is_empty() {
            return Ok(vaf.base().arguments().cloned().collect());
        }

        // Compute per-audience credulous sets.
        let per_audience: Vec<HashSet<A>> = self
            .audiences
            .iter()
            .map(|aud| -> Result<HashSet<A>, Error> {
                let defeat = vaf.defeat_graph(aud)?;
                let extensions = defeat.preferred_extensions().map_err(Error::from)?;
                let mut credulous = HashSet::new();
                for ext in extensions {
                    for arg in ext {
                        credulous.insert(arg);
                    }
                }
                Ok(credulous)
            })
            .collect::<Result<_, _>>()?;

        // Intersect.
        let mut iter = per_audience.into_iter();
        let Some(mut acc) = iter.next() else {
            return Ok(HashSet::new());
        };
        for next in iter {
            acc.retain(|a| next.contains(a));
        }
        Ok(acc)
    }

    /// Returns the set of arguments grounded under *every* audience in
    /// the set. Strictly stronger than `common_credulous` — the consensus
    /// among the most cautious answers from each character.
    pub fn common_grounded<A>(
        &self,
        vaf: &ValueBasedFramework<A>,
    ) -> Result<HashSet<A>, Error>
    where
        A: Clone + Eq + Hash + Ord + std::fmt::Debug,
    {
        if self.audiences.is_empty() {
            return Ok(vaf.base().arguments().cloned().collect());
        }

        let per_audience: Vec<HashSet<A>> = self
            .audiences
            .iter()
            .map(|aud| vaf.grounded_for(aud))
            .collect::<Result<_, _>>()?;

        let mut iter = per_audience.into_iter();
        let Some(mut acc) = iter.next() else {
            return Ok(HashSet::new());
        };
        for next in iter {
            acc.retain(|a| next.contains(a));
        }
        Ok(acc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Value, ValueAssignment};
    use argumentation::ArgumentationFramework;

    fn hal_carla() -> ValueBasedFramework<&'static str> {
        let mut base = ArgumentationFramework::new();
        for arg in ["h1", "c1", "h2", "c2"] {
            base.add_argument(arg);
        }
        base.add_attack(&"h1", &"c1").unwrap();
        base.add_attack(&"c1", &"h1").unwrap();
        base.add_attack(&"c2", &"h2").unwrap();
        base.add_attack(&"h2", &"c1").unwrap();

        let mut values = ValueAssignment::new();
        values.promote("h1", Value::new("life"));
        values.promote("c1", Value::new("property"));
        values.promote("h2", Value::new("fairness"));
        values.promote("c2", Value::new("life"));

        ValueBasedFramework::new(base, values)
    }

    #[test]
    fn common_grounded_across_opposing_audiences_yields_only_unanimous_winners() {
        let vaf = hal_carla();
        let life_first = Audience::total([Value::new("life"), Value::new("property")]);
        let property_first = Audience::total([Value::new("property"), Value::new("life")]);
        let auds = [life_first, property_first];
        let multi = MultiAudience::new(&auds);
        let common = multi.common_grounded(&vaf).unwrap();
        // c2 always grounded; nothing else survives both audiences.
        assert!(common.contains("c2"));
        assert!(!common.contains("h1"));
        assert!(!common.contains("c1"));
    }

    #[test]
    fn empty_audience_set_returns_all_arguments() {
        let vaf = hal_carla();
        let multi = MultiAudience::new(&[]);
        let common = multi.common_grounded(&vaf).unwrap();
        assert_eq!(common.len(), 4);
    }

    #[test]
    fn common_credulous_is_superset_of_common_grounded() {
        let vaf = hal_carla();
        let life_first = Audience::total([Value::new("life"), Value::new("property")]);
        let property_first = Audience::total([Value::new("property"), Value::new("life")]);
        let auds = [life_first, property_first];
        let multi = MultiAudience::new(&auds);
        let credulous = multi.common_credulous(&vaf).unwrap();
        let grounded = multi.common_grounded(&vaf).unwrap();
        for arg in &grounded {
            assert!(credulous.contains(arg), "common_grounded must subset common_credulous");
        }
    }
}
