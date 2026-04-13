//! Maps critical questions to encounter beat candidates.
//!
//! Each critical question on a [`SchemeInstance`] can be posed as a challenge
//! in an encounter. This module converts them into [`Beat`] values that the
//! encounter engine can process.

use argumentation_schemes::instance::{CriticalQuestionInstance, SchemeInstance};
use encounter::types::{Beat, Effect};

/// Convert a [`SchemeInstance`]'s critical questions into encounter beat
/// candidates, one beat per critical question.
///
/// All beats are produced with `accepted = false` (challenges are adversarial
/// by nature) and an empty `effects` list. Use [`cq_to_beat`] directly if you
/// need to attach effects to individual beats.
///
/// # Arguments
///
/// * `instance` - The instantiated scheme whose critical questions are to be
///   mapped.
/// * `challenger` - The name of the character posing the challenges.
pub fn critical_question_beats(instance: &SchemeInstance, challenger: &str) -> Vec<Beat> {
    instance
        .critical_questions
        .iter()
        .map(|cq| cq_to_beat(cq, challenger, Vec::new()))
        .collect()
}

/// Convert a single [`CriticalQuestionInstance`] into an encounter [`Beat`].
///
/// The beat's `action` is formatted as `"cq<number>:<text>"` so that both
/// the question index and its resolved text are visible in logs and tests.
///
/// # Arguments
///
/// * `cq` - The critical question instance to convert.
/// * `challenger` - The name of the character posing the challenge.
/// * `effects` - Effects to attach to the beat (caller-supplied).
pub fn cq_to_beat(cq: &CriticalQuestionInstance, challenger: &str, effects: Vec<Effect>) -> Beat {
    Beat {
        actor: challenger.to_string(),
        action: format!("cq{}:{}", cq.number, cq.text),
        accepted: false,
        effects,
    }
}
