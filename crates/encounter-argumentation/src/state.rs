//! `EncounterArgumentationState`: the encounter-level state object
//! composing schemes + bipolar + weighted + weighted-bipolar.
//!
//! Consumers build a state via `new(registry)`, optionally configure
//! a weight source and scene intensity via builders, add scheme
//! instances and raw edges, then query acceptance and coalitions.

use crate::arg_id::ArgumentId;
use crate::error::Error;
use argumentation_schemes::instance::SchemeInstance;
use argumentation_schemes::registry::CatalogRegistry;
use argumentation_weighted::types::Budget;
use argumentation_weighted_bipolar::WeightedBipolarFramework;
use std::collections::HashMap;

/// Encounter-level argumentation state composing schemes (premises +
/// conclusion), bipolar graph structure (attacks + supports), weighted
/// edge strengths, and a configurable scene-intensity budget.
pub struct EncounterArgumentationState {
    /// Scheme catalog used for instantiation + CQ lookup.
    #[allow(dead_code)]
    registry: CatalogRegistry,
    /// The underlying weighted bipolar framework.
    framework: WeightedBipolarFramework<ArgumentId>,
    /// Which actor asserted each argument. Multiple actors may share
    /// an `ArgumentId` (the same conclusion), so stored as a vec.
    actors_by_argument: HashMap<ArgumentId, Vec<String>>,
    /// Scheme instances backing each argument.
    instances_by_argument: HashMap<ArgumentId, Vec<SchemeInstance>>,
    /// Current scene intensity. Defaults to zero.
    intensity: Budget,
}

impl EncounterArgumentationState {
    /// Create a new state with the given scheme registry and zero
    /// scene intensity. Consumers that want relationship-modulated
    /// attack weights should construct a `RelationshipWeightSource`
    /// separately and pass its computed weights into
    /// [`add_weighted_attack`](Self::add_weighted_attack); Phase A
    /// does not auto-wire the source into the state.
    #[must_use]
    pub fn new(registry: CatalogRegistry) -> Self {
        Self {
            registry,
            framework: WeightedBipolarFramework::new(),
            actors_by_argument: HashMap::new(),
            instances_by_argument: HashMap::new(),
            intensity: Budget::zero(),
        }
    }

    /// Read-only access to the current scene intensity.
    #[must_use]
    pub fn intensity(&self) -> Budget {
        self.intensity
    }

    /// Number of argument nodes in the framework.
    #[must_use]
    pub fn argument_count(&self) -> usize {
        self.framework.argument_count()
    }

    /// Number of edges (attacks + supports) in the framework.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.framework.edge_count()
    }

    /// Add a scheme instance asserted by `actor`. The instance's
    /// conclusion literal becomes an argument node in the framework
    /// (if not already present). The actor and instance are recorded
    /// against that node for later lookup via `actors_for` /
    /// `instances_for`. Returns the argument's identifier.
    pub fn add_scheme_instance(
        &mut self,
        actor: &str,
        instance: SchemeInstance,
    ) -> ArgumentId {
        let id: ArgumentId = (&instance.conclusion).into();
        self.framework.add_argument(id.clone());
        self.actors_by_argument
            .entry(id.clone())
            .or_default()
            .push(actor.to_string());
        self.instances_by_argument
            .entry(id.clone())
            .or_default()
            .push(instance);
        id
    }

    /// Return the list of actors who have asserted the given argument.
    /// Empty slice if the argument is not associated with any actor.
    #[must_use]
    pub fn actors_for(&self, id: &ArgumentId) -> &[String] {
        self.actors_by_argument
            .get(id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Return the list of scheme instances backing the given argument.
    /// Empty slice if the argument is not scheme-backed.
    #[must_use]
    pub fn instances_for(&self, id: &ArgumentId) -> &[SchemeInstance] {
        self.instances_by_argument
            .get(id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Add a weighted attack edge. Both endpoints are implicitly added
    /// to the framework if not already present. Returns
    /// `Error::WeightedBipolar` for invalid weights.
    pub fn add_weighted_attack(
        &mut self,
        attacker: &ArgumentId,
        target: &ArgumentId,
        weight: f64,
    ) -> Result<(), Error> {
        self.framework
            .add_weighted_attack(attacker.clone(), target.clone(), weight)?;
        Ok(())
    }

    /// Add a weighted support edge. Both endpoints are implicitly
    /// added. Returns `Error::WeightedBipolar` for invalid weights or
    /// self-support.
    pub fn add_weighted_support(
        &mut self,
        supporter: &ArgumentId,
        supported: &ArgumentId,
        weight: f64,
    ) -> Result<(), Error> {
        self.framework
            .add_weighted_support(supporter.clone(), supported.clone(), weight)?;
        Ok(())
    }

    /// Builder method setting the scene-intensity budget. Returns
    /// `self` by value to allow chaining.
    #[must_use]
    pub fn at_intensity(mut self, intensity: Budget) -> Self {
        self.intensity = intensity;
        self
    }

    /// Whether the argument is credulously accepted under the current
    /// scene intensity (at least one preferred extension of at least
    /// one β-inconsistent residual contains it).
    pub fn is_credulously_accepted(&self, arg: &ArgumentId) -> Result<bool, Error> {
        Ok(argumentation_weighted_bipolar::is_credulously_accepted_at(
            &self.framework,
            arg,
            self.intensity,
        )?)
    }

    /// Whether the argument is skeptically accepted under the current
    /// scene intensity (every preferred extension of every
    /// β-inconsistent residual contains it).
    pub fn is_skeptically_accepted(&self, arg: &ArgumentId) -> Result<bool, Error> {
        Ok(argumentation_weighted_bipolar::is_skeptically_accepted_at(
            &self.framework,
            arg,
            self.intensity,
        )?)
    }

    /// Detect coalitions (strongly-connected components of the support
    /// graph) at the current framework state. Independent of scene
    /// intensity — coalitions are a structural property of supports,
    /// not a semantic query.
    ///
    /// Returns `Err(Error::WeightedBipolar)` if the framework exceeds
    /// the underlying edge-enumeration limit (currently 24 attacks +
    /// supports combined).
    pub fn coalitions(&self) -> Result<Vec<argumentation_bipolar::Coalition<ArgumentId>>, Error> {
        // Materialise the full-edge bipolar residual (β=0 → one residual
        // containing every edge) and run SCC on the support graph.
        let residuals = argumentation_weighted_bipolar::wbipolar_residuals(
            &self.framework,
            Budget::zero(),
        )?;
        let bipolar = residuals
            .into_iter()
            .next()
            .expect("zero-budget residual always includes the empty subset");
        Ok(argumentation_bipolar::detect_coalitions(&bipolar))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use argumentation_schemes::catalog::default_catalog;

    #[test]
    fn new_state_is_empty() {
        let state = EncounterArgumentationState::new(default_catalog());
        assert_eq!(state.argument_count(), 0);
        assert_eq!(state.edge_count(), 0);
    }

    #[test]
    fn new_state_has_zero_intensity() {
        let state = EncounterArgumentationState::new(default_catalog());
        assert_eq!(state.intensity().value(), 0.0);
    }

    #[test]
    fn add_scheme_instance_creates_argument_node() {
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();
        let instance = scheme
            .instantiate(
                &[
                    ("expert".to_string(), "alice".to_string()),
                    ("domain".to_string(), "military".to_string()),
                    ("claim".to_string(), "fortify_east".to_string()),
                ]
                .into_iter()
                .collect(),
            )
            .unwrap();

        let mut state = EncounterArgumentationState::new(registry);
        let id = state.add_scheme_instance("alice", instance);

        assert_eq!(id.as_str(), "fortify_east");
        assert_eq!(state.argument_count(), 1);
    }

    #[test]
    fn add_scheme_instance_associates_actor_and_instance() {
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();
        let instance = scheme
            .instantiate(
                &[
                    ("expert".to_string(), "alice".to_string()),
                    ("domain".to_string(), "military".to_string()),
                    ("claim".to_string(), "fortify_east".to_string()),
                ]
                .into_iter()
                .collect(),
            )
            .unwrap();
        let mut state = EncounterArgumentationState::new(registry);
        let id = state.add_scheme_instance("alice", instance);
        assert_eq!(state.actors_for(&id), &["alice".to_string()]);
        assert_eq!(state.instances_for(&id).len(), 1);
    }

    #[test]
    fn add_two_instances_with_same_conclusion_share_node() {
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();

        let inst1 = scheme
            .instantiate(
                &[
                    ("expert".to_string(), "alice".to_string()),
                    ("domain".to_string(), "military".to_string()),
                    ("claim".to_string(), "fortify_east".to_string()),
                ]
                .into_iter()
                .collect(),
            )
            .unwrap();
        let inst2 = scheme
            .instantiate(
                &[
                    ("expert".to_string(), "bob".to_string()),
                    ("domain".to_string(), "logistics".to_string()),
                    ("claim".to_string(), "fortify_east".to_string()),
                ]
                .into_iter()
                .collect(),
            )
            .unwrap();

        let mut state = EncounterArgumentationState::new(registry);
        let id1 = state.add_scheme_instance("alice", inst1);
        let id2 = state.add_scheme_instance("bob", inst2);
        assert_eq!(id1, id2);
        assert_eq!(state.argument_count(), 1);
        assert_eq!(
            state.actors_for(&id1),
            &["alice".to_string(), "bob".to_string()]
        );
        assert_eq!(state.instances_for(&id1).len(), 2);
    }

    #[test]
    fn add_weighted_attack_propagates_to_framework() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        let a = ArgumentId::new("a");
        let b = ArgumentId::new("b");
        state.add_weighted_attack(&a, &b, 0.5).unwrap();
        assert_eq!(state.edge_count(), 1);
    }

    #[test]
    fn add_weighted_support_propagates_to_framework() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        let a = ArgumentId::new("a");
        let b = ArgumentId::new("b");
        state.add_weighted_support(&a, &b, 0.5).unwrap();
        assert_eq!(state.edge_count(), 1);
    }

    #[test]
    fn add_weighted_support_rejects_self_support() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        let a = ArgumentId::new("a");
        let err = state.add_weighted_support(&a, &a, 0.5).unwrap_err();
        assert!(matches!(err, Error::WeightedBipolar(_)));
    }

    #[test]
    fn add_weighted_attack_rejects_invalid_weight() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        let a = ArgumentId::new("a");
        let b = ArgumentId::new("b");
        let err = state.add_weighted_attack(&a, &b, -0.1).unwrap_err();
        assert!(matches!(err, Error::WeightedBipolar(_)));
    }

    #[test]
    fn at_intensity_sets_budget() {
        let state = EncounterArgumentationState::new(default_catalog())
            .at_intensity(Budget::new(0.5).unwrap());
        assert_eq!(state.intensity().value(), 0.5);
    }

    #[test]
    fn at_intensity_is_chainable_with_add() {
        let mut state = EncounterArgumentationState::new(default_catalog())
            .at_intensity(Budget::new(0.25).unwrap());
        state
            .add_weighted_attack(&ArgumentId::new("a"), &ArgumentId::new("b"), 0.3)
            .unwrap();
        assert_eq!(state.intensity().value(), 0.25);
        assert_eq!(state.edge_count(), 1);
    }

    #[test]
    fn unattacked_argument_is_credulously_accepted() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        let a = ArgumentId::new("a");
        state.add_weighted_attack(&a, &ArgumentId::new("unused"), 0.0).unwrap();
        // `a` is unattacked: it appears only as attacker.
        assert!(state.is_credulously_accepted(&a).unwrap());
    }

    #[test]
    fn attacked_argument_is_not_credulously_accepted_at_zero_intensity() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        let a = ArgumentId::new("a");
        let b = ArgumentId::new("b");
        state.add_weighted_attack(&a, &b, 0.5).unwrap();
        // `b` is attacked by `a` (unattacked); at β=0 the attack binds.
        assert!(!state.is_credulously_accepted(&b).unwrap());
    }

    #[test]
    fn raising_intensity_flips_acceptance_when_budget_covers_attack() {
        let mut state = EncounterArgumentationState::new(default_catalog())
            .at_intensity(Budget::new(0.5).unwrap());
        let a = ArgumentId::new("a");
        let b = ArgumentId::new("b");
        state.add_weighted_attack(&a, &b, 0.4).unwrap();
        // At β=0.5 >= 0.4, the residual dropping a→b exists, so b is
        // credulously accepted in that residual.
        assert!(state.is_credulously_accepted(&b).unwrap());
    }

    #[test]
    fn skeptical_is_stricter_than_credulous() {
        let mut state = EncounterArgumentationState::new(default_catalog())
            .at_intensity(Budget::new(0.5).unwrap());
        let a = ArgumentId::new("a");
        let b = ArgumentId::new("b");
        state.add_weighted_attack(&a, &b, 0.4).unwrap();
        // At β=0.5, b is credulous (residual drops the attack) but NOT
        // skeptical (the full-framework residual still attacks b).
        assert!(state.is_credulously_accepted(&b).unwrap());
        assert!(!state.is_skeptically_accepted(&b).unwrap());
    }

    #[test]
    fn no_supports_means_no_coalitions() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        state.add_weighted_attack(&ArgumentId::new("a"), &ArgumentId::new("b"), 0.5).unwrap();
        let coalitions = state.coalitions().unwrap();
        // Each argument is its own singleton SCC; `detect_coalitions`
        // returns singletons too, so the invariant is that every
        // coalition has exactly one member when there are no supports.
        assert!(coalitions.iter().all(|c| c.members.len() == 1));
    }

    #[test]
    fn mutual_support_forms_coalition() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        let a = ArgumentId::new("a");
        let b = ArgumentId::new("b");
        state.add_weighted_support(&a, &b, 0.5).unwrap();
        state.add_weighted_support(&b, &a, 0.5).unwrap();
        let coalitions = state.coalitions().unwrap();
        // At least one coalition has both a and b.
        assert!(coalitions.iter().any(|c| c.members.len() == 2
            && c.members.contains(&a)
            && c.members.contains(&b)));
    }
}
