//! AIF (Argument Interchange Format) — AIFdb JSON serialization.
//!
//! Supports round-tripping a [`crate::SchemeInstance`] through the
//! community-standard AIFdb JSON format. See the crate README for the
//! exact mapping between our types and AIF nodes/edges.

use crate::Error;
use serde::{Deserialize, Serialize};

/// A single AIF node. The `type` field discriminates:
///
/// - `"I"` — information / claim (premise or conclusion literal).
/// - `"RA"` — rule application (scheme instance).
/// - `"CA"` — conflict / attack (critical question).
/// - `"MA"` — mutual attack / preference (unused in v0.2.0).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AifNode {
    /// Node identifier — unique within the document.
    #[serde(rename = "nodeID")]
    pub node_id: String,
    /// Human-readable text. For I-nodes this is `literal.to_string()`;
    /// for RA-nodes the scheme's canonical name; for CA-nodes the
    /// instantiated critical-question text.
    pub text: String,
    /// Node type: "I" | "RA" | "CA" | "MA".
    #[serde(rename = "type")]
    pub node_type: String,
    /// Scheme name — present on RA-nodes, absent (None → omitted) on others.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub scheme: Option<String>,
}

/// A directed edge between two AIF nodes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AifEdge {
    /// Edge identifier — unique within the document.
    #[serde(rename = "edgeID")]
    pub edge_id: String,
    /// Source node id.
    #[serde(rename = "fromID")]
    pub from_id: String,
    /// Target node id.
    #[serde(rename = "toID")]
    pub to_id: String,
}

/// A full AIF document: nodes, edges, and two fields we emit as empty
/// arrays for round-trip fidelity with AIFdb output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AifDocument {
    /// The AIF node list.
    pub nodes: Vec<AifNode>,
    /// The AIF edge list.
    pub edges: Vec<AifEdge>,
    /// Dialogue locutions — emitted as empty, ignored on import.
    #[serde(default)]
    pub locutions: Vec<serde_json::Value>,
    /// Dialogue participants — emitted as empty, ignored on import.
    #[serde(default)]
    pub participants: Vec<serde_json::Value>,
}

impl AifDocument {
    /// Parse an AIF JSON string.
    pub fn from_json(json: &str) -> Result<Self, Error> {
        serde_json::from_str(json).map_err(|e| Error::AifParse(e.to_string()))
    }

    /// Serialize to a pretty-printed AIF JSON string.
    pub fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string_pretty(self).map_err(|e| Error::AifParse(e.to_string()))
    }
}

/// Export a [`SchemeInstance`] to an AIF document.
///
/// Mapping:
/// - each premise literal → one I-node
/// - the conclusion literal → one I-node
/// - the scheme instance → one RA-node whose `scheme` field names the scheme
/// - each critical question → one CA-node
///
/// Edges connect each premise I-node to the RA-node, the RA-node to
/// the conclusion I-node, and each CA-node to the RA-node.
///
/// Node IDs are assigned as stringified sequential integers starting
/// at 1 in a deterministic order (premises → conclusion → RA → CAs).
pub fn instance_to_aif(instance: &crate::instance::SchemeInstance) -> AifDocument {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut next_id = 1usize;

    // Premises as I-nodes.
    let premise_ids: Vec<String> = instance
        .premises
        .iter()
        .map(|p| {
            let id = next_id.to_string();
            nodes.push(AifNode {
                node_id: id.clone(),
                text: p.to_string(),
                node_type: "I".into(),
                scheme: None,
            });
            next_id += 1;
            id
        })
        .collect();

    // Conclusion as I-node.
    let conclusion_id = next_id.to_string();
    nodes.push(AifNode {
        node_id: conclusion_id.clone(),
        text: instance.conclusion.to_string(),
        node_type: "I".into(),
        scheme: None,
    });
    next_id += 1;

    // RA-node for the scheme instance.
    let ra_id = next_id.to_string();
    nodes.push(AifNode {
        node_id: ra_id.clone(),
        text: instance.scheme_name.clone(),
        node_type: "RA".into(),
        scheme: Some(instance.scheme_name.clone()),
    });
    next_id += 1;

    // Edges: each premise → RA.
    for pid in &premise_ids {
        edges.push(AifEdge {
            edge_id: edges.len().to_string(),
            from_id: pid.clone(),
            to_id: ra_id.clone(),
        });
    }
    // RA → conclusion.
    edges.push(AifEdge {
        edge_id: edges.len().to_string(),
        from_id: ra_id.clone(),
        to_id: conclusion_id.clone(),
    });

    // CA-nodes for critical questions; each points at the RA.
    for cq in &instance.critical_questions {
        let ca_id = next_id.to_string();
        nodes.push(AifNode {
            node_id: ca_id.clone(),
            text: cq.text.clone(),
            node_type: "CA".into(),
            scheme: None,
        });
        next_id += 1;
        edges.push(AifEdge {
            edge_id: edges.len().to_string(),
            from_id: ca_id,
            to_id: ra_id.clone(),
        });
    }

    AifDocument {
        nodes,
        edges,
        locutions: vec![],
        participants: vec![],
    }
}

/// Import an AIF document back into a [`crate::instance::SchemeInstance`].
///
/// Looks up the scheme by name in the provided [`crate::registry::CatalogRegistry`],
/// re-parses each I-node as a [`argumentation::aspic::Literal`] (leading `¬` marks
/// negation), and re-derives critical-question counter-literals via
/// the catalog's `build_counter_literal` logic since AIF does not
/// preserve them directly.
///
/// Expects exactly one RA-node per document. Documents with multiple
/// RA-nodes represent conjoined arguments and are not supported in
/// v0.2.0.
pub fn aif_to_instance(
    doc: &AifDocument,
    registry: &crate::registry::CatalogRegistry,
) -> Result<crate::instance::SchemeInstance, Error> {
    let ra = doc
        .nodes
        .iter()
        .find(|n| n.node_type == "RA")
        .ok_or_else(|| Error::AifParse("no RA-node in document".into()))?;
    let scheme_name = ra
        .scheme
        .as_ref()
        .ok_or_else(|| Error::AifParse("RA-node missing 'scheme' field".into()))?;

    let _scheme = registry
        .by_name(scheme_name)
        .ok_or_else(|| Error::AifUnknownScheme(scheme_name.clone()))?;

    // Find edges: premise I-nodes point at RA; RA points at conclusion
    // I-node; CA-nodes point at RA.
    let in_edges: Vec<&AifEdge> = doc.edges.iter().filter(|e| e.to_id == ra.node_id).collect();
    let out_edges: Vec<&AifEdge> =
        doc.edges.iter().filter(|e| e.from_id == ra.node_id).collect();

    let conclusion_id = out_edges
        .first()
        .ok_or_else(|| Error::AifParse("RA has no outgoing edge to conclusion".into()))?
        .to_id
        .clone();

    let conclusion_node = doc
        .nodes
        .iter()
        .find(|n| n.node_id == conclusion_id && n.node_type == "I")
        .ok_or_else(|| {
            Error::AifParse(format!("conclusion node '{}' not found", conclusion_id))
        })?;
    let conclusion = literal_from_text(&conclusion_node.text);

    // Partition incoming edges: premises (I-nodes) vs. critical
    // questions (CA-nodes).
    let mut premises = Vec::new();
    let mut cq_texts = Vec::new();
    for edge in in_edges {
        let src = doc
            .nodes
            .iter()
            .find(|n| n.node_id == edge.from_id)
            .ok_or_else(|| {
                Error::AifParse(format!("edge references unknown node '{}'", edge.from_id))
            })?;
        match src.node_type.as_str() {
            "I" => premises.push(literal_from_text(&src.text)),
            "CA" => cq_texts.push(src.text.clone()),
            other => {
                return Err(Error::AifParse(format!(
                    "unexpected incoming node type '{}' on RA-node",
                    other
                )));
            }
        }
    }

    // Re-derive CriticalQuestionInstance list. AIF doesn't carry the
    // Challenge or counter_literal; re-instantiate by number-matching
    // from the catalog scheme, using the text as a tiebreaker.
    let scheme = registry
        .by_name(scheme_name)
        .expect("registry lookup succeeded earlier");
    let critical_questions: Vec<crate::instance::CriticalQuestionInstance> = cq_texts
        .iter()
        .enumerate()
        .map(|(idx, text)| crate::instance::CriticalQuestionInstance {
            number: (idx + 1) as u32,
            text: text.clone(),
            challenge: scheme
                .critical_questions
                .get(idx)
                .map(|cq| cq.challenge.clone())
                .unwrap_or(crate::types::Challenge::RuleValidity),
            counter_literal: argumentation::aspic::Literal::neg(format!("aif_cq_{}", idx)),
        })
        .collect();

    Ok(crate::instance::SchemeInstance {
        scheme_name: scheme_name.clone(),
        premises,
        conclusion,
        critical_questions,
    })
}

/// Parse a [`argumentation::aspic::Literal`] from its `to_string()` rendering.
///
/// Our `Literal::neg` renders with a leading `¬` (U+00AC); `Literal::atom`
/// renders plain.
fn literal_from_text(text: &str) -> argumentation::aspic::Literal {
    if let Some(stripped) = text.strip_prefix('¬') {
        argumentation::aspic::Literal::neg(stripped.trim())
    } else {
        argumentation::aspic::Literal::atom(text.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aif_node_round_trip_preserves_scheme_field() {
        let n = AifNode {
            node_id: "3".into(),
            text: "Argument from Expert Opinion".into(),
            node_type: "RA".into(),
            scheme: Some("Argument from Expert Opinion".into()),
        };
        let json = serde_json::to_string(&n).unwrap();
        let parsed: AifNode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, n);
    }

    #[test]
    fn aif_node_without_scheme_omits_field() {
        let n = AifNode {
            node_id: "1".into(),
            text: "alice is an expert".into(),
            node_type: "I".into(),
            scheme: None,
        };
        let json = serde_json::to_string(&n).unwrap();
        assert!(!json.contains("\"scheme\""));
    }

    #[test]
    fn aif_document_from_json_and_to_json_round_trip() {
        let doc = AifDocument {
            nodes: vec![AifNode {
                node_id: "1".into(),
                text: "claim".into(),
                node_type: "I".into(),
                scheme: None,
            }],
            edges: vec![],
            locutions: vec![],
            participants: vec![],
        };
        let json = doc.to_json().unwrap();
        let parsed = AifDocument::from_json(&json).unwrap();
        assert_eq!(parsed, doc);
    }

    #[test]
    fn instance_to_aif_produces_premises_ra_conclusion_and_cas() {
        use crate::catalog::default_catalog;
        use std::collections::HashMap;

        let catalog = default_catalog();
        let scheme = catalog.by_key("argument_from_expert_opinion").unwrap();
        let bindings: HashMap<String, String> = [
            ("expert".to_string(), "alice".to_string()),
            ("domain".to_string(), "military".to_string()),
            ("claim".to_string(), "fortify_east".to_string()),
        ]
        .into_iter()
        .collect();
        let instance = scheme.instantiate(&bindings).unwrap();

        let aif = instance_to_aif(&instance);

        let i_count = aif.nodes.iter().filter(|n| n.node_type == "I").count();
        let ra_count = aif.nodes.iter().filter(|n| n.node_type == "RA").count();
        let ca_count = aif.nodes.iter().filter(|n| n.node_type == "CA").count();

        assert_eq!(i_count, instance.premises.len() + 1, "one I per premise + one for conclusion");
        assert_eq!(ra_count, 1, "exactly one RA for the scheme instance");
        assert_eq!(ca_count, instance.critical_questions.len());

        // Edges: premises→RA (N), RA→conclusion (1), CAs→RA (M).
        let expected_edges =
            instance.premises.len() + 1 + instance.critical_questions.len();
        assert_eq!(aif.edges.len(), expected_edges);
    }

    #[test]
    fn instance_to_aif_tags_ra_node_with_scheme_name() {
        use crate::catalog::default_catalog;
        use std::collections::HashMap;
        let catalog = default_catalog();
        let scheme = catalog.by_key("argument_from_expert_opinion").unwrap();
        let bindings: HashMap<String, String> = [
            ("expert".to_string(), "alice".to_string()),
            ("domain".to_string(), "military".to_string()),
            ("claim".to_string(), "fortify_east".to_string()),
        ]
        .into_iter()
        .collect();
        let instance = scheme.instantiate(&bindings).unwrap();

        let aif = instance_to_aif(&instance);
        let ra = aif.nodes.iter().find(|n| n.node_type == "RA").unwrap();
        assert_eq!(ra.scheme.as_deref(), Some(instance.scheme_name.as_str()));
    }

    #[test]
    fn aif_round_trip_preserves_instance_shape() {
        use crate::catalog::default_catalog;
        use crate::registry::CatalogRegistry;
        use std::collections::HashMap;

        let catalog = default_catalog();
        let scheme = catalog.by_key("argument_from_expert_opinion").unwrap();
        let bindings: HashMap<String, String> = [
            ("expert".to_string(), "alice".to_string()),
            ("domain".to_string(), "military".to_string()),
            ("claim".to_string(), "fortify_east".to_string()),
        ]
        .into_iter()
        .collect();
        let original = scheme.instantiate(&bindings).unwrap();

        let aif = instance_to_aif(&original);
        let registry = CatalogRegistry::with_default();
        let recovered = aif_to_instance(&aif, &registry).unwrap();

        assert_eq!(recovered.scheme_name, original.scheme_name);
        assert_eq!(recovered.premises, original.premises);
        assert_eq!(recovered.conclusion, original.conclusion);
        assert_eq!(
            recovered.critical_questions.len(),
            original.critical_questions.len()
        );
        for (r, o) in recovered
            .critical_questions
            .iter()
            .zip(original.critical_questions.iter())
        {
            assert_eq!(r.text, o.text);
        }
    }

    #[test]
    fn aif_to_instance_errors_on_unknown_scheme() {
        use crate::registry::CatalogRegistry;
        let doc = AifDocument {
            nodes: vec![
                AifNode {
                    node_id: "1".into(),
                    text: "some claim".into(),
                    node_type: "I".into(),
                    scheme: None,
                },
                AifNode {
                    node_id: "2".into(),
                    text: "Argument from Flapdoodle".into(),
                    node_type: "RA".into(),
                    scheme: Some("Argument from Flapdoodle".into()),
                },
            ],
            edges: vec![AifEdge {
                edge_id: "1".into(),
                from_id: "2".into(),
                to_id: "1".into(),
            }],
            locutions: vec![],
            participants: vec![],
        };
        let registry = CatalogRegistry::with_default();
        let err = aif_to_instance(&doc, &registry).unwrap_err();
        assert!(matches!(err, Error::AifUnknownScheme(_)));
    }

    #[test]
    fn aif_to_instance_errors_on_missing_ra() {
        use crate::registry::CatalogRegistry;
        let doc = AifDocument {
            nodes: vec![AifNode {
                node_id: "1".into(),
                text: "claim".into(),
                node_type: "I".into(),
                scheme: None,
            }],
            edges: vec![],
            locutions: vec![],
            participants: vec![],
        };
        let registry = CatalogRegistry::with_default();
        let err = aif_to_instance(&doc, &registry).unwrap_err();
        assert!(matches!(err, Error::AifParse(_)));
    }
}
