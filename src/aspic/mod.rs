//! ASPIC+ structured argumentation (Modgil & Prakken 2014).

pub mod kb;
pub mod language;
pub mod rules;

pub use kb::{KnowledgeBase, Premise};
pub use language::Literal;
pub use rules::{Rule, RuleId, RuleKind};
