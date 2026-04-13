//! Scheme-backed value argument resolution.
//!
//! Resolves disputes over Schwartz values using the "Argument from Values"
//! Walton argumentation scheme evaluated through ASPIC+ and Dung preferred
//! extension semantics.  Falls back to the conviction-gap formula when the
//! scheme is unavailable or produces an undecided outcome.

use crate::resolver::ArgumentOutcome;
use argumentation_schemes::CatalogRegistry;
use encounter::value_argument::ValueArgumentResult;

/// Resolve a value argument using the "Argument from Values" Walton scheme.
///
/// Builds an ASPIC+ structured system where:
/// - The attacker instantiates the scheme with `action = "uphold_<value>"`,
///   `value = value_at_stake`, and `agent = attacker`.
/// - The defender's counter-argument is added as a contrary to the attacker's
///   conclusion.
/// - Preference ordering is determined by conviction levels (higher conviction
///   wins the preference ordering).
///
/// If the scheme is absent from `registry` or if extension semantics yield an
/// undecided result, the function falls back to the simple conviction-gap
/// formula: the character with the higher conviction wins; ties favour the
/// attacker.
///
/// # Returns
///
/// A [`ValueArgumentResult`] containing the winner, loser, the value at stake,
/// how much the loser's value shifts, and the winner's small self-reinforcement.
pub fn scheme_value_argument(
    attacker: &str,
    defender: &str,
    value_at_stake: &str,
    attacker_conviction: f64,
    defender_conviction: f64,
    defender_openness: f64,
    registry: &CatalogRegistry,
) -> ValueArgumentResult {
    let attacker_wins_by_conviction = attacker_conviction >= defender_conviction;

    // Try scheme-based resolution.
    let scheme_winner = registry.by_key("argument_from_values").and_then(|scheme| {
        use argumentation::aspic::StructuredSystem;
        use argumentation_schemes::aspic::add_scheme_to_system;

        let attacker_bindings = [
            ("action".into(), format!("uphold_{}", value_at_stake)),
            ("value".into(), value_at_stake.into()),
            ("agent".into(), attacker.into()),
        ]
        .into_iter()
        .collect();

        let attacker_instance = scheme.instantiate(&attacker_bindings).ok()?;

        let mut system = StructuredSystem::new();
        let attacker_rule = add_scheme_to_system(&attacker_instance, &mut system);

        // Defender's counter: assert the contrary of attacker's conclusion.
        let counter_literal = attacker_instance.conclusion.contrary();
        system.add_ordinary(counter_literal.clone());
        let defender_rule = system.add_defeasible_rule(
            vec![counter_literal],
            attacker_instance.conclusion.contrary(),
        );

        // Set preference by conviction level.
        if attacker_conviction > defender_conviction {
            let _ = system.prefer_rule(attacker_rule, defender_rule);
        } else if defender_conviction > attacker_conviction {
            let _ = system.prefer_rule(defender_rule, attacker_rule);
        }

        let built = system.build_framework().ok()?;
        let extensions = built.framework.preferred_extensions().ok()?;
        if extensions.is_empty() {
            return None;
        }

        let ext = &extensions[0];
        let conclusions = built.conclusions_in(ext);
        let attacker_survives = conclusions.contains(&attacker_instance.conclusion);

        if attacker_survives {
            Some(ArgumentOutcome::ProposerWins { survival_rate: 1.0 })
        } else {
            Some(ArgumentOutcome::ResponderWins { defeat_rate: 1.0 })
        }
    });

    // Determine winner using scheme result, falling back to conviction ordering.
    let attacker_wins = match scheme_winner {
        Some(ArgumentOutcome::ProposerWins { .. }) => true,
        Some(ArgumentOutcome::ResponderWins { .. }) => false,
        _ => attacker_wins_by_conviction,
    };

    let (winner, loser) = if attacker_wins {
        (attacker.to_string(), defender.to_string())
    } else {
        (defender.to_string(), attacker.to_string())
    };

    let conviction_gap = (attacker_conviction - defender_conviction).abs();
    let loser_value_shift = (conviction_gap * defender_openness).clamp(0.0, 1.0);
    let winner_value_shift = (conviction_gap * 0.1).clamp(0.0, 0.1);

    ValueArgumentResult {
        winner,
        loser,
        value_at_stake: value_at_stake.into(),
        loser_value_shift,
        winner_value_shift,
    }
}
