//! `EncounterArgumentationState`: the encounter-level state object
//! composing schemes + bipolar + weighted + weighted-bipolar.
//!
//! Consumers build a state via `new(registry)`, optionally configure
//! a weight source and scene intensity via builders, add scheme
//! instances and raw edges, then query acceptance and coalitions.

use crate::affordance_key::AffordanceKey;
use crate::arg_id::ArgumentId;
use crate::error::Error;
use argumentation_schemes::instance::SchemeInstance;
use argumentation_schemes::registry::CatalogRegistry;
use argumentation_weighted::types::Budget;
use argumentation_weighted_bipolar::WeightedBipolarFramework;
use std::collections::HashMap;
use std::sync::Mutex;

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
    /// Forward index: maps affordance-keyed scheme instances to their
    /// argument id in the framework. Populated by
    /// [`Self::add_scheme_instance_for_affordance`]. Enables bridge
    /// consumers (`StateActionScorer`, `StateAcceptanceEval`) to look
    /// up the right argument node at `evaluate`/`score_actions` time
    /// given only an `(actor, affordance_name, bindings)` triple.
    argument_id_by_affordance: HashMap<AffordanceKey, ArgumentId>,
    /// Current scene intensity (β). Stored in `Mutex` so it can be
    /// mutated through a shared reference (`&self`) — required for
    /// bridge consumers that hold `&State` during encounter's
    /// `resolve` loops — while still being `Sync`, which encounter's
    /// `AcceptanceEval`/`ActionScorer` traits require.
    intensity: Mutex<Budget>,
    /// Error latch: last error observed by a bridge impl whose trait
    /// signature can't propagate `Result` (e.g.,
    /// `AcceptanceEval::evaluate`). Consumers drain via
    /// [`Self::take_latest_error`] after encounter's resolve returns.
    latest_error: Mutex<Option<Error>>,
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
            argument_id_by_affordance: HashMap::new(),
            intensity: Mutex::new(Budget::zero()),
            latest_error: Mutex::new(None),
        }
    }

    /// Read-only access to the current scene intensity.
    #[must_use]
    pub fn intensity(&self) -> Budget {
        *self.intensity_guard()
    }

    // Lock intensity, recovering from poisoning — a prior panic
    // elsewhere must not turn into a panic here, since the bridge's
    // D5 contract requires permissive-on-failure behaviour.
    fn intensity_guard(&self) -> std::sync::MutexGuard<'_, Budget> {
        self.intensity.lock().unwrap_or_else(|e| e.into_inner())
    }

    // Lock latest_error, recovering from poisoning — see
    // `intensity_guard` for rationale.
    fn latest_error_guard(&self) -> std::sync::MutexGuard<'_, Option<Error>> {
        self.latest_error.lock().unwrap_or_else(|e| e.into_inner())
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

    /// Add a scheme instance asserted by `actor` for the named
    /// affordance with the given bindings. Functionally identical to
    /// [`Self::add_scheme_instance`] plus an entry in the affordance
    /// forward index so consumers can later look up this argument
    /// from an `(actor, affordance_name, bindings)` triple.
    ///
    /// If two scheme instances produce the same conclusion literal
    /// (same `ArgumentId`), both actors are recorded against that
    /// single argument node — convergence behaviour is the same as
    /// [`Self::add_scheme_instance`]. Both keys in the forward index
    /// point at the shared id.
    ///
    /// **Re-seeding the same key silently overwrites.** Calling this
    /// method twice with identical `(actor, affordance_name, bindings)`
    /// triples replaces the previous entry; the most recent
    /// `ArgumentId` wins. Callers should seed each affordance at most
    /// once per scene.
    pub fn add_scheme_instance_for_affordance(
        &mut self,
        actor: &str,
        affordance_name: &str,
        bindings: &std::collections::HashMap<String, String>,
        instance: SchemeInstance,
    ) -> ArgumentId {
        let id = self.add_scheme_instance(actor, instance);
        let key = AffordanceKey::new(actor, affordance_name, bindings);
        self.argument_id_by_affordance.insert(key, id.clone());
        id
    }

    /// Look up the argument id associated with an affordance key, if
    /// one was seeded via [`Self::add_scheme_instance_for_affordance`].
    #[must_use]
    pub fn argument_id_for(
        &self,
        key: &AffordanceKey,
    ) -> Option<ArgumentId> {
        self.argument_id_by_affordance.get(key).cloned()
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

    /// Return the direct attackers of `target` in the current
    /// framework. Ignores support edges and does NOT resolve the
    /// β-inconsistent residual — this is a structural query.
    ///
    /// Consumers that want "is there a credulously accepted attacker
    /// at current β?" should query each attacker via
    /// [`Self::is_credulously_accepted`].
    ///
    /// If the underlying framework contains multiple attack edges for
    /// the same `(attacker, target)` pair (the framework does not
    /// deduplicate), the attacker id appears once per edge in the
    /// returned `Vec`. Consumers that want a set projection should
    /// `.dedup()` or collect through a `HashSet`.
    #[must_use]
    pub fn attackers_of(&self, target: &ArgumentId) -> Vec<ArgumentId> {
        self.framework
            .attacks()
            .filter(|atk| &atk.target == target)
            .map(|atk| atk.attacker.clone())
            .collect()
    }

    /// Return `true` iff `responder` has put forward (via
    /// [`Self::add_scheme_instance`] or
    /// [`Self::add_scheme_instance_for_affordance`]) some argument that
    /// (1) directly attacks `target`, AND
    /// (2) is credulously accepted at the current scene intensity.
    ///
    /// This is the per-responder counter-argument query used by the
    /// bridge's [`crate::state_acceptance::StateAcceptanceEval`] to
    /// decide whether a responder rejects a proposed action. It differs
    /// from [`Self::is_credulously_accepted`] (which is a global β
    /// acceptance check regardless of who asserted the argument).
    ///
    /// Returns `Err` if the framework exceeds the weighted-bipolar
    /// residual enumeration limit. The bridge wraps this error into
    /// its error latch; consumers should rarely see `Err` surface
    /// directly.
    pub fn has_accepted_counter_by(
        &self,
        responder: &str,
        target: &ArgumentId,
    ) -> Result<bool, Error> {
        for attacker in self.attackers_of(target) {
            let asserted_by_responder = self
                .actors_for(&attacker)
                .iter()
                .any(|a| a == responder);
            if !asserted_by_responder {
                continue;
            }
            if self.is_credulously_accepted(&attacker)? {
                return Ok(true);
            }
        }
        Ok(false)
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
    pub fn at_intensity(self, intensity: Budget) -> Self {
        *self.intensity_guard() = intensity;
        self
    }

    /// Mutate the scene intensity (β) through a shared reference.
    /// Used by consumers — notably the bridge's `StateAcceptanceEval`
    /// and `StateActionScorer` — that hold `&self` during encounter's
    /// `resolve` loops but still want to escalate β mid-scene.
    ///
    /// For new-state construction prefer the by-value builder
    /// [`Self::at_intensity`].
    pub fn set_intensity(&self, intensity: Budget) {
        *self.intensity_guard() = intensity;
    }

    /// Whether the argument is credulously accepted under the current
    /// scene intensity (at least one preferred extension of at least
    /// one β-inconsistent residual contains it).
    pub fn is_credulously_accepted(&self, arg: &ArgumentId) -> Result<bool, Error> {
        Ok(argumentation_weighted_bipolar::is_credulously_accepted_at(
            &self.framework,
            arg,
            self.intensity(),
        )?)
    }

    /// Whether the argument is skeptically accepted under the current
    /// scene intensity (every preferred extension of every
    /// β-inconsistent residual contains it).
    pub fn is_skeptically_accepted(&self, arg: &ArgumentId) -> Result<bool, Error> {
        Ok(argumentation_weighted_bipolar::is_skeptically_accepted_at(
            &self.framework,
            arg,
            self.intensity(),
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

    /// Drain the last error observed by a bridge impl. Clears the
    /// latch. Returns `None` if no error has been stashed since the
    /// last drain.
    #[must_use]
    pub fn take_latest_error(&self) -> Option<Error> {
        self.latest_error_guard().take()
    }

    /// Record an error into the latch. Called by bridge impls whose
    /// trait signature can't propagate `Result`.
    ///
    /// **Overwrites any prior unread error** — callers must assume only
    /// one error survives per drain window. Consumers should call
    /// [`Self::take_latest_error`] after each `resolve` invocation if
    /// they need per-resolve error fidelity.
    pub(crate) fn record_error(&self, err: Error) {
        *self.latest_error_guard() = Some(err);
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
    fn no_supports_means_all_coalitions_are_singletons() {
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

    #[test]
    fn set_intensity_mutates_through_shared_ref() {
        let state = EncounterArgumentationState::new(default_catalog())
            .at_intensity(Budget::new(0.2).unwrap());
        assert_eq!(state.intensity().value(), 0.2);
        // set_intensity takes &self (shared ref). This line must
        // compile without &mut state.
        state.set_intensity(Budget::new(0.6).unwrap());
        assert_eq!(state.intensity().value(), 0.6);
    }

    #[test]
    fn intensity_is_mutable_from_two_shared_refs_in_sequence() {
        let state = EncounterArgumentationState::new(default_catalog());
        fn bump(s: &EncounterArgumentationState, b: f64) {
            s.set_intensity(Budget::new(b).unwrap());
        }
        bump(&state, 0.3);
        bump(&state, 0.5);
        assert_eq!(state.intensity().value(), 0.5);
    }

    #[test]
    fn add_scheme_instance_for_affordance_indexes_by_key() {
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();
        let mut bindings = std::collections::HashMap::new();
        bindings.insert("expert".to_string(), "alice".to_string());
        bindings.insert("domain".to_string(), "military".to_string());
        bindings.insert("claim".to_string(), "fortify_east".to_string());
        let instance = scheme.instantiate(&bindings).unwrap();

        let mut state = EncounterArgumentationState::new(registry);
        let id = state.add_scheme_instance_for_affordance(
            "alice",
            "argue_fortify_east",
            &bindings,
            instance,
        );

        let key = AffordanceKey::new("alice", "argue_fortify_east", &bindings);
        let looked_up = state.argument_id_for(&key);
        assert_eq!(looked_up, Some(id));
    }

    #[test]
    fn argument_id_for_returns_none_for_unseeded_key() {
        let bindings = std::collections::HashMap::new();
        let state = EncounterArgumentationState::new(default_catalog());
        let key = AffordanceKey::new("nobody", "nothing", &bindings);
        assert_eq!(state.argument_id_for(&key), None);
    }

    #[test]
    fn add_scheme_instance_for_affordance_is_consistent_with_add_scheme_instance() {
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();
        let mut bindings = std::collections::HashMap::new();
        bindings.insert("expert".to_string(), "alice".to_string());
        bindings.insert("domain".to_string(), "military".to_string());
        bindings.insert("claim".to_string(), "fortify_east".to_string());
        let instance = scheme.instantiate(&bindings).unwrap();
        let mut state = EncounterArgumentationState::new(registry);
        let id = state.add_scheme_instance_for_affordance(
            "alice",
            "argue_fortify_east",
            &bindings,
            instance,
        );
        assert_eq!(state.actors_for(&id), &["alice".to_string()]);
        assert_eq!(state.instances_for(&id).len(), 1);
    }

    #[test]
    fn two_distinct_affordance_keys_with_same_conclusion_point_at_shared_id() {
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();

        let mut alice_bindings = std::collections::HashMap::new();
        alice_bindings.insert("expert".to_string(), "alice".to_string());
        alice_bindings.insert("domain".to_string(), "military".to_string());
        alice_bindings.insert("claim".to_string(), "fortify_east".to_string());
        let alice_instance = scheme.instantiate(&alice_bindings).unwrap();

        let mut bob_bindings = std::collections::HashMap::new();
        bob_bindings.insert("expert".to_string(), "bob".to_string());
        bob_bindings.insert("domain".to_string(), "logistics".to_string());
        bob_bindings.insert("claim".to_string(), "fortify_east".to_string());
        let bob_instance = scheme.instantiate(&bob_bindings).unwrap();

        let mut state = EncounterArgumentationState::new(registry);
        let alice_id = state.add_scheme_instance_for_affordance(
            "alice",
            "argue_fortify_east",
            &alice_bindings,
            alice_instance,
        );
        let bob_id = state.add_scheme_instance_for_affordance(
            "bob",
            "second_expert_opinion",
            &bob_bindings,
            bob_instance,
        );

        assert_eq!(alice_id, bob_id, "same conclusion literal → shared ArgumentId");
        assert_eq!(state.argument_count(), 1);

        let alice_key = AffordanceKey::new("alice", "argue_fortify_east", &alice_bindings);
        let bob_key = AffordanceKey::new("bob", "second_expert_opinion", &bob_bindings);
        assert_eq!(state.argument_id_for(&alice_key), Some(alice_id.clone()));
        assert_eq!(state.argument_id_for(&bob_key), Some(alice_id.clone()));

        assert_eq!(
            state.actors_for(&alice_id),
            &["alice".to_string(), "bob".to_string()]
        );
    }

    #[test]
    fn attackers_of_returns_all_direct_attackers() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        let target = ArgumentId::new("target");
        let a1 = ArgumentId::new("a1");
        let a2 = ArgumentId::new("a2");
        let unrelated = ArgumentId::new("unrelated");
        state.add_weighted_attack(&a1, &target, 0.5).unwrap();
        state.add_weighted_attack(&a2, &target, 0.3).unwrap();
        state.add_weighted_attack(&unrelated, &ArgumentId::new("x"), 0.5).unwrap();
        let attackers: std::collections::HashSet<_> =
            state.attackers_of(&target).into_iter().collect();
        assert_eq!(attackers.len(), 2);
        assert!(attackers.contains(&a1));
        assert!(attackers.contains(&a2));
    }

    #[test]
    fn attackers_of_returns_empty_for_unattacked() {
        let state = EncounterArgumentationState::new(default_catalog());
        let lonely = ArgumentId::new("lonely");
        assert!(state.attackers_of(&lonely).is_empty());
    }

    #[test]
    fn attackers_of_preserves_duplicate_edges() {
        let mut state = EncounterArgumentationState::new(default_catalog());
        let target = ArgumentId::new("target");
        let a1 = ArgumentId::new("a1");
        state.add_weighted_attack(&a1, &target, 0.5).unwrap();
        state.add_weighted_attack(&a1, &target, 0.7).unwrap();
        assert_eq!(state.attackers_of(&target).len(), 2);
    }

    #[test]
    fn has_accepted_counter_by_detects_responder_attacker_at_beta() {
        let registry = default_catalog();
        let scheme = registry.by_key("argument_from_expert_opinion").unwrap();
        let mut target_bindings = std::collections::HashMap::new();
        target_bindings.insert("expert".to_string(), "alice".to_string());
        target_bindings.insert("domain".to_string(), "military".to_string());
        target_bindings.insert("claim".to_string(), "fortify_east".to_string());
        let target_instance = scheme.instantiate(&target_bindings).unwrap();
        let mut counter_bindings = std::collections::HashMap::new();
        counter_bindings.insert("expert".to_string(), "bob".to_string());
        counter_bindings.insert("domain".to_string(), "logistics".to_string());
        counter_bindings.insert("claim".to_string(), "abandon_east".to_string());
        let counter_instance = scheme.instantiate(&counter_bindings).unwrap();

        let mut state = EncounterArgumentationState::new(registry);
        let target_id = state.add_scheme_instance("alice", target_instance);
        let counter_id = state.add_scheme_instance("bob", counter_instance);
        state.add_weighted_attack(&counter_id, &target_id, 0.5).unwrap();

        // At β=0 bob's counter is credulously accepted (unattacked) and
        // attacks alice's target → has_accepted_counter_by(bob, target)=true,
        // has_accepted_counter_by(alice, target)=false.
        assert!(state.has_accepted_counter_by("bob", &target_id).unwrap());
        assert!(!state.has_accepted_counter_by("alice", &target_id).unwrap());
    }

    #[test]
    fn take_latest_error_round_trips_a_stashed_error() {
        let state = EncounterArgumentationState::new(default_catalog());
        // No error yet.
        assert!(state.take_latest_error().is_none());
        // Stash an error manually (internal helper).
        state.record_error(Error::SchemeNotFound("x".into()));
        let err = state.take_latest_error();
        assert!(matches!(err, Some(Error::SchemeNotFound(_))));
        // Second call: drained.
        assert!(state.take_latest_error().is_none());
    }
}
