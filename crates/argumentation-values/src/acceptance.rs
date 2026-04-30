//! Subjective and objective acceptance over the space of audiences.
//!
//! Per Bench-Capon (2003):
//! - **Subjective acceptance**: arg is accepted by *some* audience.
//! - **Objective acceptance**: arg is accepted by *every* audience.
//!
//! Both queries enumerate the linear extensions of the partial order
//! defined by the value set. The number of linear extensions is bounded
//! above by `n!` for `n` distinct values; we hard-cap at 6 (= 720) to
//! keep these queries tractable for narrative-scale frameworks.
//!
//! Past the cap, methods return [`Error::AudienceTooLarge`].

use crate::error::Error;
use crate::framework::ValueBasedFramework;
use crate::types::{Audience, Value};
use std::hash::Hash;

/// Hard cap on distinct values for subjective/objective acceptance.
/// At 6 values, worst-case linear-extension count is 720.
pub const ENUMERATION_LIMIT: usize = 6;

/// Returns `Ok(true)` iff `arg` is accepted by *some* total ordering of
/// the values mentioned in the framework.
///
/// Strategy: enumerate every permutation of the value set, build the
/// corresponding total-order [`Audience`], and check `accepted_for`.
/// Returns true on the first acceptance; otherwise false.
pub fn subjectively_accepted<A>(
    vaf: &ValueBasedFramework<A>,
    arg: &A,
) -> Result<bool, Error>
where
    A: Clone + Eq + Hash + Ord + std::fmt::Debug,
{
    let values: Vec<Value> = vaf
        .value_assignment()
        .distinct_values()
        .into_iter()
        .cloned()
        .collect();

    if values.len() > ENUMERATION_LIMIT {
        return Err(Error::AudienceTooLarge {
            values: values.len(),
            limit: ENUMERATION_LIMIT,
        });
    }

    for perm in permutations(&values) {
        let audience = Audience::total(perm);
        if vaf.accepted_for(&audience, arg)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Returns `Ok(true)` iff `arg` is accepted by *every* total ordering of
/// the values mentioned in the framework.
pub fn objectively_accepted<A>(
    vaf: &ValueBasedFramework<A>,
    arg: &A,
) -> Result<bool, Error>
where
    A: Clone + Eq + Hash + Ord + std::fmt::Debug,
{
    let values: Vec<Value> = vaf
        .value_assignment()
        .distinct_values()
        .into_iter()
        .cloned()
        .collect();

    if values.len() > ENUMERATION_LIMIT {
        return Err(Error::AudienceTooLarge {
            values: values.len(),
            limit: ENUMERATION_LIMIT,
        });
    }

    for perm in permutations(&values) {
        let audience = Audience::total(perm);
        if !vaf.accepted_for(&audience, arg)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// All permutations of a small slice. Heap's algorithm; allocates a
/// fresh `Vec<Value>` per permutation. Acceptable up to ENUMERATION_LIMIT.
fn permutations<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
    let mut result = Vec::new();
    let mut working: Vec<T> = items.to_vec();
    permute_recursive(&mut working, 0, &mut result);
    result
}

fn permute_recursive<T: Clone>(items: &mut [T], start: usize, out: &mut Vec<Vec<T>>) {
    if start == items.len() {
        out.push(items.to_vec());
        return;
    }
    for i in start..items.len() {
        items.swap(start, i);
        permute_recursive(items, start + 1, out);
        items.swap(start, i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ValueAssignment;
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
    fn c2_objectively_accepted() {
        let vaf = hal_carla();
        // c2 has no in-edges in any audience → always grounded → always accepted.
        assert!(objectively_accepted(&vaf, &"c2").unwrap());
    }

    #[test]
    fn h1_subjectively_but_not_objectively_accepted() {
        let vaf = hal_carla();
        // h1 is accepted under [life > property, fairness, ...] but not
        // under [property > life, ...].
        assert!(subjectively_accepted(&vaf, &"h1").unwrap());
        assert!(!objectively_accepted(&vaf, &"h1").unwrap());
    }

    #[test]
    fn c1_subjectively_but_not_objectively_accepted() {
        let vaf = hal_carla();
        // Symmetric to h1.
        assert!(subjectively_accepted(&vaf, &"c1").unwrap());
        assert!(!objectively_accepted(&vaf, &"c1").unwrap());
    }

    #[test]
    fn audience_too_large_returns_error() {
        // Build a 7-value framework and verify the cap fires.
        let mut base = ArgumentationFramework::new();
        for arg in ["a", "b", "c", "d", "e", "f", "g"] {
            base.add_argument(arg);
        }
        let mut values = ValueAssignment::new();
        for (i, name) in ["v1", "v2", "v3", "v4", "v5", "v6", "v7"].iter().enumerate() {
            let arg = ["a", "b", "c", "d", "e", "f", "g"][i];
            values.promote(arg, Value::new(*name));
        }
        let vaf = ValueBasedFramework::new(base, values);
        let result = subjectively_accepted(&vaf, &"a");
        assert!(matches!(
            result,
            Err(Error::AudienceTooLarge { values: 7, limit: 6 })
        ));
    }

    #[test]
    fn permutations_of_three_yields_six() {
        let perms = permutations(&[1, 2, 3]);
        assert_eq!(perms.len(), 6);
    }

    #[test]
    fn permutations_of_zero_yields_one_empty() {
        let perms: Vec<Vec<i32>> = permutations(&[]);
        assert_eq!(perms.len(), 1);
        assert!(perms[0].is_empty());
    }
}
