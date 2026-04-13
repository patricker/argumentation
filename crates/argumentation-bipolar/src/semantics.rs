//! Bipolar semantics: compute Dung extensions on the flattened framework
//! and filter them for support closure under necessary-support semantics.
//!
//! An extension `E` is **support-closed** iff for every `a ∈ E`, every
//! direct necessary supporter of `a` is also in `E`. Nouioua & Risch 2011
//! proves this captures necessary-support acceptability exactly when
//! applied on top of Dung extensions of the closed attack relation.

use crate::error::Error;
use crate::flatten::flatten;
use crate::framework::BipolarFramework;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

/// Check whether a candidate extension is support-closed in a bipolar
/// framework: every argument in the extension has all its direct
/// necessary supporters in the extension too.
#[must_use]
pub fn is_support_closed<A>(framework: &BipolarFramework<A>, extension: &HashSet<A>) -> bool
where
    A: Clone + Eq + Hash,
{
    for a in extension {
        for supporter in framework.direct_supporters(a) {
            if !extension.contains(supporter) {
                return false;
            }
        }
    }
    true
}

/// All bipolar preferred extensions under necessary-support semantics.
///
/// Pipeline: flatten → Dung preferred extensions → support-closure filter.
/// The filter may drop candidates that are Dung-preferred but not
/// support-closed. It does NOT promote smaller subsets in their place;
/// if every Dung-preferred extension is filtered out, the result is
/// empty.
pub fn bipolar_preferred_extensions<A>(
    framework: &BipolarFramework<A>,
) -> Result<Vec<HashSet<A>>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let af = flatten(framework)?;
    let dung_preferred = af.preferred_extensions()?;
    let filtered: Vec<HashSet<A>> = dung_preferred
        .into_iter()
        .filter(|ext| is_support_closed(framework, ext))
        .collect();
    Ok(filtered)
}

/// All bipolar complete extensions under necessary-support semantics.
pub fn bipolar_complete_extensions<A>(
    framework: &BipolarFramework<A>,
) -> Result<Vec<HashSet<A>>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let af = flatten(framework)?;
    let dung_complete = af.complete_extensions()?;
    let filtered: Vec<HashSet<A>> = dung_complete
        .into_iter()
        .filter(|ext| is_support_closed(framework, ext))
        .collect();
    Ok(filtered)
}

/// All bipolar stable extensions.
pub fn bipolar_stable_extensions<A>(
    framework: &BipolarFramework<A>,
) -> Result<Vec<HashSet<A>>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let af = flatten(framework)?;
    let dung_stable = af.stable_extensions()?;
    let filtered: Vec<HashSet<A>> = dung_stable
        .into_iter()
        .filter(|ext| is_support_closed(framework, ext))
        .collect();
    Ok(filtered)
}

/// The bipolar grounded extension.
///
/// The Dung grounded extension is unique and may or may not be
/// support-closed. If it is not, this function returns the largest
/// support-closed subset of it — specifically, the result of
/// iteratively removing any argument whose direct supporter is missing.
/// (This differs from the preferred/complete case where we drop the
/// whole candidate; grounded is unique so we must always return
/// something.)
pub fn bipolar_grounded_extension<A>(framework: &BipolarFramework<A>) -> Result<HashSet<A>, Error>
where
    A: Clone + Eq + Hash + Debug + Ord,
{
    let af = flatten(framework)?;
    let mut grounded = af.grounded_extension();
    // Support-closure repair: remove arguments whose direct supporters
    // are missing, until a fixed point.
    loop {
        let to_remove: Vec<A> = grounded
            .iter()
            .filter(|a| {
                framework
                    .direct_supporters(a)
                    .iter()
                    .any(|s| !grounded.contains(*s))
            })
            .cloned()
            .collect();
        if to_remove.is_empty() {
            break;
        }
        for a in to_remove {
            grounded.remove(&a);
        }
    }
    Ok(grounded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_extension_is_support_closed() {
        let bf: BipolarFramework<&str> = BipolarFramework::new();
        let ext: HashSet<&str> = HashSet::new();
        assert!(is_support_closed(&bf, &ext));
    }

    #[test]
    fn extension_missing_supporter_fails_closure() {
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        let ext: HashSet<&str> = ["b"].into_iter().collect();
        assert!(!is_support_closed(&bf, &ext));
    }

    #[test]
    fn extension_containing_supporter_passes_closure() {
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        let ext: HashSet<&str> = ["a", "b"].into_iter().collect();
        assert!(is_support_closed(&bf, &ext));
    }

    #[test]
    fn bipolar_preferred_filters_unsupported_candidates() {
        // b has a necessary supporter a, but nothing attacks either.
        // Dung would give {a, b} as the only preferred extension.
        // Bipolar should give the same (and no {b}-alone candidate).
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        let prefs = bipolar_preferred_extensions(&bf).unwrap();
        assert_eq!(prefs.len(), 1);
        assert_eq!(prefs[0].len(), 2);
        assert!(prefs[0].contains(&"a"));
        assert!(prefs[0].contains(&"b"));
    }

    #[test]
    fn grounded_repair_removes_unsupported_arguments() {
        // a is attacked by c, b is supported by a. Dung grounded would
        // include b (unattacked) but not a (attacked by unattacked c).
        // Under support closure, b must be removed because its
        // supporter a is missing.
        let mut bf = BipolarFramework::new();
        bf.add_support("a", "b").unwrap();
        bf.add_attack("c", "a");
        let grounded = bipolar_grounded_extension(&bf).unwrap();
        assert!(grounded.contains(&"c"));
        assert!(!grounded.contains(&"a"));
        assert!(
            !grounded.contains(&"b"),
            "b should be removed because its supporter a is missing"
        );
    }
}
