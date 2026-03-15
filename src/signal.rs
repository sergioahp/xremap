use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::config::keymap_action::KeymapAction;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SignalKind {
    Fire,
    StartRepeat,
    StopRepeat,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signal {
    pub name: String,
    pub kind: SignalKind,
}

#[derive(Clone, Debug)]
pub struct SignalBinding {
    pub actions: Vec<KeymapAction>,
    pub repeat: Option<Duration>,
}

#[derive(Debug)]
struct ActiveRepeat {
    interval: Duration,
    next_due: Instant,
    actions: Vec<KeymapAction>,
}

/// Converts signals emitted by the pattern matcher into concrete KeymapActions,
/// and schedules repeaters when requested.
pub struct SignalDispatcher {
    bindings: HashMap<String, SignalBinding>,
    repeats: HashMap<String, ActiveRepeat>,
}

impl SignalDispatcher {
    pub fn new(bindings: HashMap<String, SignalBinding>) -> Self {
        SignalDispatcher {
            bindings,
            repeats: HashMap::new(),
        }
    }

    /// Handle incoming signals, returning immediate actions and updating repeat state.
    pub fn handle_signals(&mut self, signals: Vec<Signal>, now: Instant) -> Vec<KeymapAction> {
        let mut actions: Vec<KeymapAction> = vec![];
        for signal in signals {
            if let Some(binding) = self.bindings.get(&signal.name) {
                match signal.kind {
                    SignalKind::Fire => {
                        // One-shot
                        actions.extend(binding.actions.clone());
                    }
                    SignalKind::StartRepeat => {
                        if let Some(interval) = binding.repeat {
                            let entry = ActiveRepeat {
                                interval,
                                next_due: now + interval,
                                actions: binding.actions.clone(),
                            };
                            self.repeats.insert(signal.name.clone(), entry);
                            // Fire once immediately
                            actions.extend(binding.actions.clone());
                        } else {
                            // No repeat configured, treat as one-shot
                            actions.extend(binding.actions.clone());
                        }
                    }
                    SignalKind::StopRepeat => {
                        self.repeats.remove(&signal.name);
                    }
                }
            }
        }
        actions
    }

    /// Produce actions for repeats that are due at `now` and return the time until the next due repeat.
    pub fn tick(&mut self, now: Instant) -> (Vec<KeymapAction>, Option<Duration>) {
        let mut actions = vec![];
        let mut next_due: Option<Instant> = None;
        let keys: Vec<String> = self.repeats.keys().cloned().collect();
        for key in keys {
            if let Some(rep) = self.repeats.get_mut(&key) {
                if rep.next_due <= now {
                    actions.extend(rep.actions.clone());
                    rep.next_due += rep.interval;
                }
                next_due = match next_due {
                    Some(cur) => Some(cur.min(rep.next_due)),
                    None => Some(rep.next_due),
                };
            }
        }
        let delay = next_due.map(|due| due.saturating_duration_since(now));
        (actions, delay)
    }

    pub fn has_repeats(&self) -> bool {
        !self.repeats.is_empty()
    }

    pub fn next_due(&self) -> Option<Instant> {
        self.repeats.values().map(|r| r.next_due).min()
    }

    pub fn stop_all(&mut self) {
        self.repeats.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::keymap_action::KeymapAction;

    #[test]
    fn start_and_stop_repeat() {
        let mut dispatcher = SignalDispatcher::new(HashMap::from([(
            "nav.left".into(),
            SignalBinding {
                actions: vec![KeymapAction::EscapeNextKey(false)],
                repeat: Some(Duration::from_millis(10)),
            },
        )]));
        let now = Instant::now();
        let actions = dispatcher.handle_signals(
            vec![Signal {
                name: "nav.left".into(),
                kind: SignalKind::StartRepeat,
            }],
            now,
        );
        assert_eq!(actions.len(), 1);
        let (tick_actions, next) = dispatcher.tick(now + Duration::from_millis(10));
        assert_eq!(tick_actions.len(), 1);
        assert!(next.is_some());
        dispatcher.handle_signals(
            vec![Signal {
                name: "nav.left".into(),
                kind: SignalKind::StopRepeat,
            }],
            now,
        );
        assert!(!dispatcher.has_repeats());
    }
}
