# Changelog

## [0.1.0] - 2026-04-13

### Added
- `BipolarFramework<A>` with distinct attack and support edge sets,
  plus mutation and query APIs for each.
- Derived attack closure per Cayrol & Lagasquie-Schiex 2005 and Amgoud
  et al. 2008: supported, secondary, and mediated attacks computed as a
  fixed point over the support graph.
- Flattening: convert a bipolar framework into an equivalent
  `argumentation::ArgumentationFramework` whose attack relation is the
  closed attack set.
- Necessary-support semantics via Dung flatten + support-closure filter:
  `bipolar_preferred_extensions`, `bipolar_complete_extensions`,
  `bipolar_stable_extensions`, `bipolar_grounded_extension`.
- Coalition detection via Tarjan SCC on the support graph.
- Transitive supporter/attacker queries and coalition-membership lookup.
- 35 unit tests + 13 integration tests covering UC1 (corroboration),
  UC2 (coalition), UC3 (betrayal), UC4 (mediated attack).

### Known limitations
- Only `SupportSemantics::Necessary` is implemented. `Deductive` and
  `Evidential` are pre-declared for API stability but return
  `Error::UnimplementedSemantics`.
- No weighted-bipolar composition with `argumentation-weighted` yet;
  the natural composition point is a v0.2.0 follow-up.
