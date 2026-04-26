//! Shared trace types serialised by every scene fixture.

use serde::Serialize;

#[derive(Serialize)]
pub struct Trace {
    pub scene_name: String,
    pub beta: f64,
    pub participants: Vec<String>,
    pub seeded_arguments: Vec<SeededArg>,
    pub attacks: Vec<AttackEdge>,
    pub beats: Vec<BeatRecord>,
    pub errors: Vec<String>,
}

#[derive(Serialize)]
pub struct SeededArg {
    pub actor: String,
    pub affordance_name: String,
    pub conclusion: String,
}

#[derive(Serialize)]
pub struct AttackEdge {
    pub attacker: String,
    pub target: String,
    pub weight: f64,
}

#[derive(Serialize)]
pub struct BeatRecord {
    pub actor: String,
    pub action: String,
    pub accepted: bool,
}
