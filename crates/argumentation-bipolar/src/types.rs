//! Foundational types for bipolar argumentation.

/// Which kind of directed edge is in the framework: an attack (A defeats B)
/// or a support (A is required for B under necessary-support semantics).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    /// `A` attacks `B` — the Dung-standard attack relation.
    Attack,
    /// `A` supports `B` — under necessary-support semantics, `A` must be
    /// in any extension that contains `B`.
    Support,
}

/// Which support semantics the framework uses. Only [`Self::Necessary`]
/// is implemented in v0.1.0; [`Self::Deductive`] and [`Self::Evidential`]
/// are reserved for v0.2.0 and return [`crate::Error::UnimplementedSemantics`]
/// if requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportSemantics {
    /// Necessary support (Nouioua & Risch 2011): `A` supports `B` means
    /// `A` must be in any extension containing `B`.
    Necessary,
    /// Deductive support (Boella et al. 2010). NOT IMPLEMENTED in v0.1.0.
    Deductive,
    /// Evidential support (Oren & Norman 2008). NOT IMPLEMENTED in v0.1.0.
    Evidential,
}

/// Identifier for a coalition detected via strongly-connected components
/// of the support graph. Coalition ids are assigned at detection time by
/// [`crate::coalition::detect_coalitions`] and are only stable within a
/// single call — they change if the framework is mutated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CoalitionId(pub u32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_kind_distinguishes_attack_from_support() {
        assert_ne!(EdgeKind::Attack, EdgeKind::Support);
    }

    #[test]
    fn support_semantics_necessary_is_default_implementation() {
        // v0.1.0 only supports Necessary. The other two exist for API
        // stability but route to UnimplementedSemantics.
        assert_eq!(SupportSemantics::Necessary, SupportSemantics::Necessary);
        assert_ne!(SupportSemantics::Necessary, SupportSemantics::Deductive);
        assert_ne!(SupportSemantics::Necessary, SupportSemantics::Evidential);
    }

    #[test]
    fn coalition_id_equality_is_value_based() {
        assert_eq!(CoalitionId(1), CoalitionId(1));
        assert_ne!(CoalitionId(1), CoalitionId(2));
    }
}
