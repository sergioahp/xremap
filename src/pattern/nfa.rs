use std::collections::HashSet;

use crate::pattern::ast::{ActionSpec, Edge, Node, NodeKind, RepeatKind};

#[derive(Clone, Debug)]
pub struct Transition {
    pub edge: Option<Edge>,
    pub target: usize,
    pub actions: Vec<ActionSpec>,
}

#[derive(Clone, Debug)]
pub struct State {
    pub transitions: Vec<Transition>,
}

#[derive(Clone, Debug)]
pub struct Nfa {
    pub states: Vec<State>,
    pub start: usize,
}

impl Nfa {
    pub fn new(states: Vec<State>, start: usize) -> Self {
        Nfa { states, start }
    }
}

pub fn compile_to_nfa(root: &Node) -> Nfa {
    let mut builder = Builder::new();
    let (start, end) = builder.build(root);
    // ensure end state exists
    builder.add_epsilon(end, None);
    Nfa::new(builder.states, start)
}

struct Builder {
    states: Vec<State>,
}

impl Builder {
    fn new() -> Self {
        Builder { states: vec![State { transitions: vec![] }] }
    }

    fn add_state(&mut self) -> usize {
        let idx = self.states.len();
        self.states.push(State { transitions: vec![] });
        idx
    }

    fn add_transition(&mut self, from: usize, edge: Option<Edge>, to: usize, actions: Vec<ActionSpec>) {
        self.states[from].transitions.push(Transition { edge, target: to, actions });
    }

    fn add_epsilon(&mut self, from: usize, to: Option<usize>) {
        let target = to.unwrap_or_else(|| self.add_state());
        self.add_transition(from, None, target, vec![]);
    }

    fn build(&mut self, node: &Node) -> (usize, usize) {
        match &node.kind {
            NodeKind::Epsilon => {
                let s = self.add_state();
                let e = self.add_state();
                self.add_transition(s, None, e, node.actions.clone());
                (s, e)
            }
            NodeKind::Event(edge) => {
                let s = self.add_state();
                let e = self.add_state();
                self.add_transition(s, Some(edge.clone()), e, node.actions.clone());
                (s, e)
            }
            NodeKind::Seq(nodes) => {
                let mut starts = vec![];
                let mut ends = vec![];
                for n in nodes {
                    let (s, e) = self.build(n);
                    starts.push(s);
                    ends.push(e);
                }
                for i in 0..ends.len() - 1 {
                    self.add_transition(ends[i], None, starts[i + 1], vec![]);
                }
                (starts[0], *ends.last().unwrap())
            }
            NodeKind::Alt(nodes) => {
                let start = self.add_state();
                let end = self.add_state();
                for n in nodes {
                    let (s, e) = self.build(n);
                    self.add_transition(start, None, s, vec![]);
                    self.add_transition(e, None, end, vec![]);
                }
                (start, end)
            }
            NodeKind::Timeout { node, .. } => {
                // Timeout not enforced yet; compile inner node directly.
                self.build(node)
            }
            NodeKind::Repeat { node, kind } => match kind {
                RepeatKind::ZeroOrMore => {
                    let start = self.add_state();
                    let end = self.add_state();
                    let (s, e) = self.build(node);
                    self.add_transition(start, None, end, vec![]); // skip
                    self.add_transition(start, None, s, vec![]);
                    self.add_transition(e, None, s, vec![]); // loop
                    self.add_transition(e, None, end, vec![]);
                    (start, end)
                }
                RepeatKind::OneOrMore => {
                    let start = self.add_state();
                    let end = self.add_state();
                    let (s, e) = self.build(node);
                    self.add_transition(start, None, s, vec![]);
                    self.add_transition(e, None, s, vec![]); // loop
                    self.add_transition(e, None, end, vec![]);
                    (start, end)
                }
                RepeatKind::Optional => {
                    let start = self.add_state();
                    let end = self.add_state();
                    let (s, e) = self.build(node);
                    self.add_transition(start, None, end, vec![]);
                    self.add_transition(start, None, s, vec![]);
                    self.add_transition(e, None, end, vec![]);
                    (start, end)
                }
            },
        }
    }
}

/// Runtime thread set stepping through the NFA.
#[derive(Clone, Debug)]
pub struct Machine {
    pub nfa: Nfa,
    pub current: HashSet<usize>,
}

#[derive(Debug)]
pub struct StepResult {
    pub actions: Vec<ActionSpec>,
    pub alive: bool,
    pub ended: bool,
}

impl Machine {
    pub fn new(nfa: Nfa) -> Self {
        let mut m = Machine { nfa, current: HashSet::new() };
        m.reset();
        m
    }

    pub fn reset(&mut self) {
        self.current.clear();
        self.add_state(self.nfa.start);
    }

    fn add_state(&mut self, state: usize) {
        let mut stack = vec![state];
        while let Some(s) = stack.pop() {
            if self.current.insert(s) {
                for t in &self.nfa.states[s].transitions {
                    if t.edge.is_none() {
                        stack.push(t.target);
                    }
                }
            }
        }
    }

    pub fn step(&mut self, edge: &Edge) -> StepResult {
        let mut next_states: HashSet<usize> = HashSet::new();
        let mut actions = vec![];
        for s in self.current.iter() {
            for t in &self.nfa.states[*s].transitions {
                if let Some(e) = &t.edge {
                    if e == edge {
                        actions.extend(t.actions.clone());
                        collect_epsilons(&self.nfa, t.target, &mut next_states, &mut actions);
                    }
                }
            }
        }
        let alive = !next_states.is_empty();
        let ended = actions.iter().any(|a| matches!(a, ActionSpec::End));
        self.current = next_states;
        StepResult { actions, alive, ended }
    }
}

fn collect_epsilons(
    nfa: &Nfa,
    start: usize,
    set: &mut HashSet<usize>,
    actions: &mut Vec<ActionSpec>,
) {
    let mut stack = vec![start];
    while let Some(s) = stack.pop() {
        if set.insert(s) {
            for t in &nfa.states[s].transitions {
                if t.edge.is_none() {
                    actions.extend(t.actions.clone());
                    stack.push(t.target);
                }
            }
        }
    }
}
