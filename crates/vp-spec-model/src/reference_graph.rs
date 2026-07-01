//! Typed graph of symbolic references across the specification corpus.

use std::collections::HashMap;
use std::path::Path;

use vp_diagnostics::Location;

use crate::document_corpus::DocumentCorpus;
use crate::link_resolve::{path_key, resolve_relative_link, split_link_target};
use crate::reference_discovery::{
    DiscoveredReference, MarkdownDiscovery, ReferenceDiscovery, ReferenceKind,
};
use crate::registry_set::RegistrySet;

/// Entity kinds represented in the reference graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReferenceNodeKind {
    Document,
    Section,
    VpTerm,
    VpRfc,
    VpCs,
    Anchor,
    External,
}

/// One node in the reference graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceNode {
    pub id: String,
    pub kind: ReferenceNodeKind,
    pub display_name: String,
}

/// One directed reference edge with source location metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceEdge {
    pub source: String,
    pub target: String,
    pub reference_kind: ReferenceKind,
    /// Raw discovered target string (e.g. `missing.md`, `VP-TERM-001`).
    pub symbolic_target: String,
    pub source_location: Location,
}

/// Immutable graph of discovered symbolic references.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReferenceGraph {
    nodes: Vec<ReferenceNode>,
    edges: Vec<ReferenceEdge>,
    by_id: HashMap<String, usize>,
    incoming: HashMap<String, Vec<usize>>,
    outgoing: HashMap<String, Vec<usize>>,
}

impl ReferenceGraph {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn nodes(&self) -> &[ReferenceNode] {
        &self.nodes
    }

    pub fn edges(&self) -> &[ReferenceEdge] {
        &self.edges
    }

    pub fn lookup(&self, id: &str) -> Option<&ReferenceNode> {
        self.by_id.get(id).map(|index| &self.nodes[*index])
    }

    pub fn incoming(&self, node_id: &str) -> Vec<&ReferenceEdge> {
        self.incoming
            .get(node_id)
            .into_iter()
            .flat_map(|indices| indices.iter().map(|index| &self.edges[*index]))
            .collect()
    }

    pub fn outgoing(&self, node_id: &str) -> Vec<&ReferenceEdge> {
        self.outgoing
            .get(node_id)
            .into_iter()
            .flat_map(|indices| indices.iter().map(|index| &self.edges[*index]))
            .collect()
    }
}

pub(crate) fn build_reference_graph(
    registry_set: &RegistrySet,
    document_corpus: &DocumentCorpus,
) -> ReferenceGraph {
    let mut builder = ReferenceGraphBuilder::new();

    for entry in registry_set.terminology.entries() {
        builder.insert_node(
            term_node_id(&entry.id),
            ReferenceNodeKind::VpTerm,
            entry.title.clone(),
        );
    }

    for entry in registry_set.rfcs.entries() {
        builder.insert_node(
            rfc_node_id(&entry.id),
            ReferenceNodeKind::VpRfc,
            entry.title.clone(),
        );
    }

    for document in document_corpus.documents() {
        builder.insert_node(
            document_node_id(&document.relative_path),
            ReferenceNodeKind::Document,
            document.relative_path.clone(),
        );

        for section in &document.sections {
            if section.level == 0 {
                builder.insert_node(
                    anchor_node_id(&document.relative_path, &section.anchor),
                    ReferenceNodeKind::Anchor,
                    section.anchor.clone(),
                );
            } else {
                builder.insert_node(
                    section_node_id(&document.relative_path, &section.anchor),
                    ReferenceNodeKind::Section,
                    section.title.clone(),
                );
            }
        }
    }

    let discovery = MarkdownDiscovery::new();
    for document in document_corpus.documents() {
        let source_path = Path::new(&document.relative_path);
        let source_id = document_node_id(&document.relative_path);

        for reference in discovery.discover(source_path, &document.raw_text) {
            let (target_id, display_name, target_kind) =
                target_node_for_reference(source_path, &reference);
            builder.insert_node(target_id.clone(), target_kind, display_name);
            builder.insert_edge(ReferenceEdge {
                source: source_id.clone(),
                target: target_id,
                reference_kind: reference.kind,
                symbolic_target: reference.target.clone(),
                source_location: reference.location.clone(),
            });
        }
    }

    builder.finish()
}

fn target_node_for_reference(
    source_path: &Path,
    reference: &DiscoveredReference,
) -> (String, String, ReferenceNodeKind) {
    match reference.kind {
        ReferenceKind::Terminology => (
            term_node_id(&reference.target),
            reference.target.clone(),
            ReferenceNodeKind::VpTerm,
        ),
        ReferenceKind::Rfc => (
            rfc_node_id(&reference.target),
            reference.target.clone(),
            ReferenceNodeKind::VpRfc,
        ),
        ReferenceKind::ArchitectureSection => (
            architecture_section_node_id(&reference.target),
            reference.target.clone(),
            ReferenceNodeKind::Section,
        ),
        ReferenceKind::MarkdownFile => {
            let resolved = path_key(&resolve_relative_link(source_path, &reference.target));
            (
                document_node_id(&resolved),
                resolved.clone(),
                ReferenceNodeKind::Document,
            )
        }
        ReferenceKind::MarkdownAnchor => {
            let (path_part, anchor) = split_link_target(&reference.target);
            let resolved = if path_part.is_empty() {
                path_key(source_path)
            } else {
                path_key(&resolve_relative_link(source_path, &path_part))
            };
            let fragment = anchor.unwrap_or_default();
            (
                anchor_node_id(&resolved, &fragment),
                format!("{resolved}#{fragment}"),
                ReferenceNodeKind::Anchor,
            )
        }
        ReferenceKind::Future => (
            external_node_id(&reference.target),
            reference.target.clone(),
            ReferenceNodeKind::External,
        ),
    }
}

fn document_node_id(path: &str) -> String {
    format!("document:{path}")
}

fn term_node_id(id: &str) -> String {
    format!("term:{id}")
}

fn rfc_node_id(id: &str) -> String {
    format!("rfc:{id}")
}

fn architecture_section_node_id(id: &str) -> String {
    format!("section:{id}")
}

fn section_node_id(document_path: &str, anchor: &str) -> String {
    format!("section:{document_path}:{anchor}")
}

fn anchor_node_id(document_path: &str, anchor: &str) -> String {
    format!("anchor:{document_path}:{anchor}")
}

fn external_node_id(target: &str) -> String {
    format!("external:{target}")
}

struct ReferenceGraphBuilder {
    nodes: Vec<ReferenceNode>,
    edges: Vec<ReferenceEdge>,
    by_id: HashMap<String, usize>,
    incoming: HashMap<String, Vec<usize>>,
    outgoing: HashMap<String, Vec<usize>>,
}

impl ReferenceGraphBuilder {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            by_id: HashMap::new(),
            incoming: HashMap::new(),
            outgoing: HashMap::new(),
        }
    }

    fn insert_node(&mut self, id: String, kind: ReferenceNodeKind, display_name: String) {
        if self.by_id.contains_key(&id) {
            return;
        }
        let index = self.nodes.len();
        self.by_id.insert(id.clone(), index);
        self.nodes.push(ReferenceNode {
            id,
            kind,
            display_name,
        });
    }

    fn insert_edge(&mut self, edge: ReferenceEdge) {
        let index = self.edges.len();
        self.incoming
            .entry(edge.target.clone())
            .or_default()
            .push(index);
        self.outgoing
            .entry(edge.source.clone())
            .or_default()
            .push(index);
        self.edges.push(edge);
    }

    fn finish(self) -> ReferenceGraph {
        ReferenceGraph {
            nodes: self.nodes,
            edges: self.edges,
            by_id: self.by_id,
            incoming: self.incoming,
            outgoing: self.outgoing,
        }
    }
}
