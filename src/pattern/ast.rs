use evdev::KeyCode as Key;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edge {
    Press(Key),
    Release(Key),
    /// Matches any key press or release. Used as a catch-all in loops to
    /// consume unrecognised keys without letting them fall through.
    Any,
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
    PushFrame,              // push current NFA states (existing behavior)
    PushFrameOf(Box<Node>), // inline sub-pattern: resolved at compile time
    PushFrameAt(usize),     // runtime: push epsilon-closure of this NFA state
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
