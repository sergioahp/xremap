pub mod ast;
pub mod nfa;
pub mod parser;

use std::collections::HashMap;

use ast::Edge;
use evdev::KeyCode as Key;
use nfa::{compile_to_nfa, Machine, Nfa};
use parser::parse_pattern;

#[derive(Clone, Debug)]
pub struct CompiledPattern {
    pub name: String,
    pub nfa: Nfa,
    pub start_edges: Vec<Edge>,
}

impl CompiledPattern {
    pub fn machine(&self) -> Machine {
        Machine::new(self.nfa.clone())
    }
}

pub fn compile_patterns(patterns: &HashMap<String, String>) -> Result<Vec<CompiledPattern>, String> {
    let mut compiled = vec![];
    for (name, src) in patterns {
        let ast = parse_pattern(src).map_err(|e| e.0)?;
        let nfa = compile_to_nfa(&ast);
        let start_edges = first_edges(&nfa);
        compiled.push(CompiledPattern {
            name: name.clone(),
            nfa,
            start_edges,
        });
    }
    Ok(compiled)
}

fn first_edges(nfa: &Nfa) -> Vec<Edge> {
    let mut edges = vec![];
    for t in &nfa.states[nfa.start].transitions {
        if let Some(e) = &t.edge {
            edges.push(e.clone());
        }
    }
    edges
}

pub fn edge_key(edge: &Edge) -> (Key, bool) {
    match edge {
        Edge::Press(k) => (*k, false),
        Edge::Release(k) => (*k, true),
    }
}

