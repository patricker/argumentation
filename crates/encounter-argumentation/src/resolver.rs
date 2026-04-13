//! Core argument resolution via ASPIC+ extension semantics.

use argumentation::aspic::{Literal, RuleId, StructuredSystem};
use argumentation_schemes::CatalogRegistry;
use argumentation_schemes::aspic::add_scheme_to_system;
use argumentation_schemes::instance::SchemeInstance;
use argumentation_schemes::types::SchemeStrength;

/// Outcome of resolving an argument between proposer and responder.
#[derive(Debug, Clone, PartialEq)]
pub enum ArgumentOutcome {
    /// The proposer's argument(s) survived in the preferred extension.
    ProposerWins {
        /// Fraction of proposer conclusions that survived (0.0-1.0).
        survival_rate: f64,
    },
    /// The responder's counter-argument(s) defeated the proposer.
    ResponderWins {
        /// Fraction of proposer conclusions that were defeated (0.0-1.0).
        defeat_rate: f64,
    },
    /// Neither side decisively won.
    Undecided,
}

/// Look up a scheme instance's strength from the catalog registry.
///
/// Converts the instance's `scheme_name` to a snake_case key and looks it up.
/// Falls back to `Moderate` when the name is not found in the registry.
fn lookup_strength(instance: &SchemeInstance, registry: &CatalogRegistry) -> SchemeStrength {
    let key = instance.scheme_name.to_lowercase().replace(' ', "_");
    registry
        .by_key(&key)
        .map(|s| s.metadata.strength)
        .unwrap_or(SchemeStrength::Moderate)
}

/// Numeric weight for a scheme strength (higher = stronger).
fn strength_rank(strength: SchemeStrength) -> u8 {
    crate::strength_rank(strength)
}

/// Resolve an argument between proposer and responder scheme instances.
///
/// Builds an ASPIC+ `StructuredSystem`, adds all scheme instances, sets
/// preference ordering based on scheme strength, computes preferred
/// extensions, and determines which side's conclusions survive.
pub fn resolve_argument(
    proposer_instances: &[SchemeInstance],
    responder_instances: &[SchemeInstance],
    registry: &CatalogRegistry,
) -> ArgumentOutcome {
    if proposer_instances.is_empty() {
        return ArgumentOutcome::Undecided;
    }

    if responder_instances.is_empty() {
        return ArgumentOutcome::ProposerWins { survival_rate: 1.0 };
    }

    let mut system = StructuredSystem::new();

    // Add all proposer instances and record their rule IDs with strengths.
    let proposer_rules: Vec<(RuleId, SchemeStrength)> = proposer_instances
        .iter()
        .map(|inst| {
            let rule_id = add_scheme_to_system(inst, &mut system);
            let strength = lookup_strength(inst, registry);
            (rule_id, strength)
        })
        .collect();

    // Add all responder instances and record their rule IDs with strengths.
    let responder_rules: Vec<(RuleId, SchemeStrength)> = responder_instances
        .iter()
        .map(|inst| {
            let rule_id = add_scheme_to_system(inst, &mut system);
            let strength = lookup_strength(inst, registry);
            (rule_id, strength)
        })
        .collect();

    // Set preference ordering: stronger rules defeat weaker ones.
    // For each proposer rule vs each responder rule, prefer the one with
    // higher strength. Equal strength means no preference is set.
    for &(p_rule, p_strength) in &proposer_rules {
        for &(r_rule, r_strength) in &responder_rules {
            let p_rank = strength_rank(p_strength);
            let r_rank = strength_rank(r_strength);
            if p_rank > r_rank {
                // Proposer's rule is stronger; prefer it over responder's.
                let _ = system.prefer_rule(p_rule, r_rule);
            } else if r_rank > p_rank {
                // Responder's rule is stronger; prefer it over proposer's.
                let _ = system.prefer_rule(r_rule, p_rule);
            }
            // Equal strength: no preference — symmetric attack, both can survive.
        }
    }

    // Build the framework and compute preferred extensions.
    let built = match system.build_framework() {
        Ok(b) => b,
        Err(_) => return ArgumentOutcome::Undecided,
    };

    let extensions = match built.framework.preferred_extensions() {
        Ok(exts) => exts,
        Err(_) => return ArgumentOutcome::Undecided,
    };

    if extensions.is_empty() {
        return ArgumentOutcome::Undecided;
    }

    // Collect the proposer's conclusions.
    let proposer_conclusions: Vec<Literal> = proposer_instances
        .iter()
        .map(|inst| inst.conclusion.clone())
        .collect();

    // Count how many proposer conclusions survive across ALL preferred extensions.
    // A conclusion "survives" when it appears in every preferred extension.
    let total = proposer_conclusions.len();
    let survived_count = proposer_conclusions
        .iter()
        .filter(|conclusion| {
            // The conclusion survives if at least one argument supporting it
            // is in every preferred extension.
            extensions.iter().all(|ext| {
                let ext_conclusions = built.conclusions_in(ext);
                ext_conclusions.contains(conclusion)
            })
        })
        .count();

    let survival_rate = survived_count as f64 / total as f64;

    if survival_rate > 0.5 {
        ArgumentOutcome::ProposerWins { survival_rate }
    } else if survival_rate == 0.0 {
        // Check whether any responder conclusion survived.
        let responder_conclusions: Vec<Literal> = responder_instances
            .iter()
            .map(|inst| inst.conclusion.clone())
            .collect();
        let responder_survived = responder_conclusions.iter().any(|conclusion| {
            extensions.iter().all(|ext| {
                let ext_conclusions = built.conclusions_in(ext);
                ext_conclusions.contains(conclusion)
            })
        });
        if responder_survived {
            let defeat_rate = 1.0 - survival_rate;
            ArgumentOutcome::ResponderWins { defeat_rate }
        } else {
            ArgumentOutcome::Undecided
        }
    } else {
        ArgumentOutcome::Undecided
    }
}
