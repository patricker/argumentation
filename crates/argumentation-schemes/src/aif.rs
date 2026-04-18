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
}
