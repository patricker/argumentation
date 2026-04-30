//! Hal & Carla integration test — the success criterion from the VAF
//! mini-RFC. Verifies grounded-extension flips by audience.
//!
//! Bench-Capon (2003): given the framework
//!     C1 ↔ H1   (mutual attack between property and life)
//!     C2 → H2   (Carla's "my only dose" defeats Hal's "too poor to compensate")
//!     H2 → C1   (Hal's poverty argument also attacks Carla's property claim)
//! and value assignment
//!     H1 → life
//!     C1 → property
//!     H2 → fairness
//!     C2 → life
//! the grounded extensions under three audiences should be:
//!     [[life], [property]]   → {H1, C2}   (Hal goes free)
//!     [[property], [life]]   → {C1, C2}   (Hal punished)
//!     [[life, property]]     → {C2}       (life and property incomparable;
//!                                          mutual attack stalemates; only C2
//!                                          (no in-edges) is grounded)

use argumentation::ArgumentationFramework;
use argumentation_values::{Audience, Value, ValueAssignment, ValueBasedFramework};

fn hal_carla_vaf() -> ValueBasedFramework<&'static str> {
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
fn life_over_property_grounds_hal() {
    let vaf = hal_carla_vaf();
    let audience = Audience::total([Value::new("life"), Value::new("property")]);
    let grounded = vaf.grounded_for(&audience).unwrap();
    assert!(grounded.contains("h1"), "h1 should be grounded under [life > property]");
    assert!(grounded.contains("c2"), "c2 should be grounded under [life > property]");
    assert!(!grounded.contains("c1"), "c1 should be defeated under [life > property]");
    assert!(!grounded.contains("h2"), "h2 should be defeated under [life > property]");
}

#[test]
fn property_over_life_grounds_carla() {
    let vaf = hal_carla_vaf();
    let audience = Audience::total([Value::new("property"), Value::new("life")]);
    let grounded = vaf.grounded_for(&audience).unwrap();
    assert!(grounded.contains("c1"), "c1 should be grounded under [property > life]");
    assert!(grounded.contains("c2"), "c2 should be grounded under [property > life]");
    assert!(!grounded.contains("h1"), "h1 should be defeated under [property > life]");
    assert!(!grounded.contains("h2"), "h2 should be defeated under [property > life]");
}

#[test]
fn incomparable_audience_yields_dung_result() {
    let vaf = hal_carla_vaf();
    // Both values in the same tier — neither is strictly preferred.
    let audience = Audience::from_tiers(vec![vec![
        Value::new("life"),
        Value::new("property"),
    ]]);
    // Without preferences, c1 ↔ h1 mutual attack stalemates: neither
    // is in the grounded extension. c2 still wins (no in-edges).
    let grounded = vaf.grounded_for(&audience).unwrap();
    assert!(grounded.contains("c2"), "c2 always grounded (no in-edges)");
    assert!(!grounded.contains("h1"), "h1 not grounded under symmetric attack");
    assert!(!grounded.contains("c1"), "c1 not grounded under symmetric attack");
    // h2 is defeated by c2 (which is in grounded), so h2 is out.
    assert!(!grounded.contains("h2"), "h2 defeated by grounded c2");
}

#[test]
fn accepted_for_matches_grounded_for_unique_extension() {
    let vaf = hal_carla_vaf();
    let audience = Audience::total([Value::new("life"), Value::new("property")]);
    // Under this audience the framework should have a unique preferred
    // extension that equals the grounded extension. Verify both APIs agree.
    assert!(vaf.accepted_for(&audience, &"h1").unwrap());
    assert!(vaf.accepted_for(&audience, &"c2").unwrap());
    assert!(!vaf.accepted_for(&audience, &"c1").unwrap());
    assert!(!vaf.accepted_for(&audience, &"h2").unwrap());
}
