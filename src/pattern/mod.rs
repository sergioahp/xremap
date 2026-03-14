pub mod ast;
pub mod nfa;
pub mod parser;

use std::collections::HashMap;

use ast::{Edge, Node, NodeKind};
use evdev::KeyCode as Key;
use nfa::{compile_to_nfa, Nfa};
use parser::parse_pattern;

pub use nfa::Machine;

/// Build a single fused NFA from all patterns. The fused NFA is an Alt of
/// every individual pattern NFA — one machine that runs all patterns in
/// parallel. The event handler resets it when nothing matches (loop-back)
/// and re-arms from held modifiers instead of maintaining a start table.
pub fn build_fused_nfa(patterns: &HashMap<String, String>) -> Result<Nfa, String> {
    let nodes: Result<Vec<Node>, String> = patterns
        .values()
        .map(|src| parse_pattern(src).map_err(|e| e.0))
        .collect();
    let nodes = nodes?;
    if nodes.is_empty() {
        return Ok(compile_to_nfa(&Node::new(NodeKind::Epsilon)));
    }
    Ok(compile_to_nfa(&Node::new(NodeKind::Alt(nodes))))
}

pub fn edge_key(edge: &Edge) -> Option<(Key, bool)> {
    match edge {
        Edge::Press(k) => Some((*k, false)),
        Edge::Release(k) => Some((*k, true)),
        Edge::Any => None,
    }
}
