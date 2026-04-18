//! Subset enumeration over (attacks ∪ supports) for weighted bipolar
//! frameworks under Amgoud 2008 + Dunne 2011 semantics.

use crate::error::Error;
use crate::framework::WeightedBipolarFramework;
use crate::types::Budget;
use argumentation_bipolar::BipolarFramework;
use std::fmt::Debug;
use std::hash::Hash;

/// Upper bound on the combined attack + support edge count for exact
/// subset enumeration. `2^24 ≈ 16.8M` subsets per residual build;
/// larger frameworks return [`Error::TooManyEdges`].
pub const EDGE_ENUMERATION_LIMIT: usize = 24;

/// Enumerate the residual [`BipolarFramework`]s obtained by dropping
/// every β-inconsistent subset `S` of `framework`'s edges. Returns one
/// residual per subset; residuals are yielded in bit-mask order where
/// bits `0..attacks.len()` index attacks and bits
/// `attacks.len()..attacks.len() + supports.len()` index supports.
pub fn wbipolar_residuals<A>(
    framework: &WeightedBipolarFramework<A>,
    budget: Budget,
) -> Result<Vec<BipolarFramework<A>>, Error>
where
    A: Clone + Eq + Hash + Debug,
{
    let attacks: Vec<_> = framework.attacks().collect();
    let supports: Vec<_> = framework.supports().collect();
    let m_a = attacks.len();
    let m_s = supports.len();
    let m = m_a + m_s;

    if m > EDGE_ENUMERATION_LIMIT {
        return Err(Error::TooManyEdges {
            edges: m,
            limit: EDGE_ENUMERATION_LIMIT,
        });
    }

    let args: Vec<A> = framework.arguments().cloned().collect();
    let total = 1u64 << m;
    let mut residuals = Vec::new();

    for bits in 0..total {
        let mut cost = 0.0_f64;
        for (i, atk) in attacks.iter().enumerate() {
            if bits & (1u64 << i) != 0 {
                cost += atk.weight.value();
            }
        }
        for (j, sup) in supports.iter().enumerate() {
            if bits & (1u64 << (m_a + j)) != 0 {
                cost += sup.weight.value();
            }
        }
        if cost > budget.value() {
            continue;
        }

        let mut bf: BipolarFramework<A> = BipolarFramework::new();
        for a in &args {
            bf.add_argument(a.clone());
        }
        for (i, atk) in attacks.iter().enumerate() {
            if bits & (1u64 << i) == 0 {
                bf.add_attack(atk.attacker.clone(), atk.target.clone());
            }
        }
        for (j, sup) in supports.iter().enumerate() {
            if bits & (1u64 << (m_a + j)) == 0 {
                bf.add_support(sup.supporter.clone(), sup.supported.clone())?;
            }
        }
        residuals.push(bf);
    }

    Ok(residuals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_budget_yields_single_residual_with_all_edges() {
        let mut wbf = WeightedBipolarFramework::new();
        wbf.add_weighted_attack("a", "b", 0.3).unwrap();
        wbf.add_weighted_support("c", "a", 0.2).unwrap();
        let residuals = wbipolar_residuals(&wbf, Budget::zero()).unwrap();
        assert_eq!(residuals.len(), 1);
        let r = &residuals[0];
        assert_eq!(r.attacks().count(), 1);
        assert_eq!(r.supports().count(), 1);
    }

    #[test]
    fn large_budget_yields_full_power_set_over_edges() {
        let mut wbf = WeightedBipolarFramework::new();
        wbf.add_weighted_attack("a", "b", 0.5).unwrap();
        wbf.add_weighted_support("c", "a", 0.5).unwrap();
        let residuals = wbipolar_residuals(&wbf, Budget::new(10.0).unwrap()).unwrap();
        assert_eq!(residuals.len(), 4);
    }

    #[test]
    fn budget_at_cheapest_edge_yields_two_residuals() {
        let mut wbf = WeightedBipolarFramework::new();
        wbf.add_weighted_attack("a", "b", 0.2).unwrap();
        wbf.add_weighted_support("c", "a", 0.9).unwrap();
        let residuals = wbipolar_residuals(&wbf, Budget::new(0.2).unwrap()).unwrap();
        assert_eq!(residuals.len(), 2);
    }

    #[test]
    fn oversized_framework_rejected() {
        let mut wbf: WeightedBipolarFramework<u32> = WeightedBipolarFramework::new();
        for i in 0..(EDGE_ENUMERATION_LIMIT as u32 + 1) {
            wbf.add_weighted_attack(2 * i, 2 * i + 1, 0.1).unwrap();
        }
        let err = wbipolar_residuals(&wbf, Budget::new(1.0).unwrap()).unwrap_err();
        assert!(matches!(err, Error::TooManyEdges { .. }));
    }

    #[test]
    fn every_residual_preserves_all_arguments() {
        let mut wbf = WeightedBipolarFramework::new();
        wbf.add_argument("isolated");
        wbf.add_weighted_attack("a", "b", 0.5).unwrap();
        let residuals = wbipolar_residuals(&wbf, Budget::new(1.0).unwrap()).unwrap();
        for r in &residuals {
            assert_eq!(r.arguments().count(), 3);
        }
    }
}
