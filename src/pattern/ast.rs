use evdev::KeyCode as Key;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edge {
    Press(Key),
    Release(Key),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepeatKind {
    ZeroOrMore,
    OneOrMore,
    Optional,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionSpec {
    Emit(String, crate::signal::SignalKind),
    Noop,
    End,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Epsilon,
    Event(Edge),
    Seq(Vec<Node>),
    Alt(Vec<Node>),
    Repeat {
        node: Box<Node>,
        kind: RepeatKind,
    },
    Timeout {
        node: Box<Node>,
        duration: std::time::Duration,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub kind: NodeKind,
    pub actions: Vec<ActionSpec>,
}

impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Node {
            kind,
            actions: vec![],
        }
    }
}
