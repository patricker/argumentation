//! Foundational types for bipolar argumentation. Full content in Task 2.

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
