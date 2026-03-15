pub mod ast;
pub mod nfa;
pub mod parser;

use std::collections::HashMap;

use ast::{Edge, Node};
use evdev::KeyCode as Key;
use nfa::{compile_fused_nfa, Nfa};
use parser::parse_pattern;

pub use nfa::Machine;

/// Build a single fused NFA from all named patterns.
/// Each pattern's states occupy a contiguous range recorded in
/// `nfa.pattern_ranges` so the event handler can identify the active pattern.
pub fn build_fused_nfa(patterns: &HashMap<String, String>) -> Result<Nfa, String> {
    // Sort by name for a deterministic state layout.
    let mut named_nodes: Vec<(String, Node)> = patterns
        .iter()
        .map(|(name, src)| parse_pattern(src).map_err(|e| e.0).map(|n| (name.clone(), n)))
        .collect::<Result<_, _>>()?;
    named_nodes.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(compile_fused_nfa(&named_nodes))
}

pub fn edge_key(edge: &Edge) -> Option<(Key, bool)> {
    match edge {
        Edge::Press(k) => Some((*k, false)),
        Edge::Release(k) => Some((*k, true)),
        Edge::Any => None,
    }
}
