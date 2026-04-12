//! ASPIC+ structured argumentation (Modgil & Prakken 2014).

pub mod argument;
pub mod attacks;
pub mod defeat;
pub mod kb;
pub mod language;
pub mod postulates;
pub mod rules;

pub use argument::{Argument, ArgumentId, Origin, construct_arguments};
pub use attacks::{Attack, AttackKind, compute_attacks};
pub use defeat::{BuildOutput, DefeatOrdering, StructuredSystem};
pub use kb::{KnowledgeBase, Premise};
pub use language::Literal;
pub use postulates::{PostulateReport, PostulateViolation};
pub use rules::{Rule, RuleId, RuleKind};
