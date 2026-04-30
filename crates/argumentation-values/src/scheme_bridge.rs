//! Bridge between `argumentation-schemes` and `ValueAssignment`.
//!
//! The `argument_from_values` Walton scheme (Walton 2008 p.321) carries
//! a `value` premise slot — see
//! `argumentation-schemes/src/catalog/practical.rs:120`. This module
//! extracts that slot from instantiated schemes and builds a
//! [`ValueAssignment`] keyed by the scheme's conclusion.
//!
//! For schemes other than `argument_from_values`, no value is extracted.
//!
//! ## Note on the scheme identifier
//!
//! `SchemeInstance` carries `scheme_name: String` (the human-readable name
//! from `SchemeSpec::name`), not the snake-case key. We compare against the
//! literal name `"Argument from Values"` since that is what
//! `argument_from_values()` registers in the default catalog. If consumers
//! register a custom values scheme under a different name, they should
//! call [`from_scheme_instances_with_name`] with the appropriate name.

use crate::types::{Value, ValueAssignment};
use argumentation_schemes::SchemeInstance;
use std::collections::HashMap;
use std::hash::Hash;

/// The default-catalog name of the values scheme — see
/// `argumentation-schemes/src/catalog/practical.rs:124`.
pub const DEFAULT_VALUES_SCHEME_NAME: &str = "Argument from Values";

/// Extract value promotions from an iterator of instantiated schemes
/// using the default catalog's values-scheme name.
///
/// For each scheme instance:
/// - If `instance.scheme_name == "Argument from Values"` and a `"value"`
///   binding is present, the scheme's conclusion is mapped to the bound
///   value.
/// - Otherwise, the scheme is skipped silently.
///
/// `to_arg` converts a `SchemeInstance` to the caller's argument label
/// type — typically by reading the conclusion literal. For encounter use
/// this is a closure producing an `ArgumentId` from `instance.conclusion`.
///
/// Bindings are passed in separately because `SchemeInstance` does not
/// retain its bindings post-instantiation — only the resolved literals.
/// Callers must keep the original bindings alongside each instance.
pub fn from_scheme_instances<'a, A, I, F>(
    instances: I,
    to_arg: F,
) -> ValueAssignment<A>
where
    A: Eq + Hash + Clone,
    I: IntoIterator<Item = (&'a SchemeInstance, &'a HashMap<String, String>)>,
    F: Fn(&SchemeInstance) -> A,
{
    from_scheme_instances_with_name(instances, to_arg, DEFAULT_VALUES_SCHEME_NAME)
}

/// Same as [`from_scheme_instances`] but lets the caller specify a custom
/// values-scheme name (for consumers who register their own variant).
pub fn from_scheme_instances_with_name<'a, A, I, F>(
    instances: I,
    to_arg: F,
    target_scheme_name: &str,
) -> ValueAssignment<A>
where
    A: Eq + Hash + Clone,
    I: IntoIterator<Item = (&'a SchemeInstance, &'a HashMap<String, String>)>,
    F: Fn(&SchemeInstance) -> A,
{
    let mut assignment = ValueAssignment::new();
    for (instance, bindings) in instances {
        if instance.scheme_name.as_str() != target_scheme_name {
            continue;
        }
        let Some(value_str) = bindings.get("value") else {
            continue;
        };
        let arg = to_arg(instance);
        assignment.promote(arg, Value::new(value_str.clone()));
    }
    assignment
}

#[cfg(test)]
mod tests {
    use super::*;
    use argumentation_schemes::catalog::default_catalog;

    #[test]
    fn extracts_value_from_argument_from_values_scheme() {
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_values").unwrap();

        let mut bindings = HashMap::new();
        bindings.insert("action".into(), "uphold_honor".into());
        bindings.insert("value".into(), "honor".into());
        bindings.insert("agent".into(), "alice".into());
        let instance = scheme.instantiate(&bindings).unwrap();

        let to_arg = |inst: &SchemeInstance| inst.conclusion.to_string();
        let assignment = from_scheme_instances(
            std::iter::once((&instance, &bindings)),
            to_arg,
        );
        let arg = instance.conclusion.to_string();
        let values = assignment.values(&arg);
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].as_str(), "honor");
    }

    #[test]
    fn skips_non_value_schemes() {
        let registry = default_catalog();
        let scheme = registry
            .by_key("argument_from_expert_opinion")
            .expect("argument_from_expert_opinion in default catalog");

        let mut bindings = HashMap::new();
        bindings.insert("expert".into(), "alice".into());
        bindings.insert("domain".into(), "military".into());
        bindings.insert("claim".into(), "fortify".into());
        let instance = scheme.instantiate(&bindings).unwrap();

        let to_arg = |inst: &SchemeInstance| inst.conclusion.to_string();
        let assignment = from_scheme_instances(
            std::iter::once((&instance, &bindings)),
            to_arg,
        );
        // Empty assignment because the scheme is not Argument from Values.
        assert!(assignment.values(&instance.conclusion.to_string()).is_empty());
    }

    #[test]
    fn custom_scheme_name_supported() {
        // If a consumer registers their own values scheme under a different
        // name (e.g., "Custom Values"), the with_name variant lets them
        // target it explicitly. Here we just verify the API shape compiles
        // and behaves correctly when the name doesn't match.
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_values").unwrap();
        let mut bindings = HashMap::new();
        bindings.insert("action".into(), "do_x".into());
        bindings.insert("value".into(), "honor".into());
        bindings.insert("agent".into(), "alice".into());
        let instance = scheme.instantiate(&bindings).unwrap();

        let to_arg = |inst: &SchemeInstance| inst.conclusion.to_string();
        let assignment = from_scheme_instances_with_name(
            std::iter::once((&instance, &bindings)),
            to_arg,
            "Some Other Scheme Name",
        );
        // No match — empty assignment.
        assert!(assignment.values(&instance.conclusion.to_string()).is_empty());
    }
}
