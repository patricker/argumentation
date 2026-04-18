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
}
