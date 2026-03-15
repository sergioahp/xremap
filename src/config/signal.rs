use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;

use crate::config::keymap_action::Actions;
use crate::config::keymap_action::KeymapAction;

#[derive(Clone, Debug, Deserialize)]
pub struct SignalBindingConfig {
    #[serde(default, deserialize_with = "deserialize_actions_vec")]
    pub actions: Vec<KeymapAction>,
    #[serde(default = "default_repeat_false")]
    pub repeat: bool,
    #[serde(default)]
    pub interval_ms: Option<u64>,
}

impl SignalBindingConfig {
    pub fn to_runtime(&self) -> (Vec<KeymapAction>, Option<Duration>) {
        let repeat = if self.repeat {
            self.interval_ms.map(Duration::from_millis)
        } else {
            None
        };
        (self.actions.clone(), repeat)
    }
}

fn default_repeat_false() -> bool {
    false
}

pub fn parse_signal_bindings(
    bindings: HashMap<String, SignalBindingConfig>,
) -> HashMap<String, (Vec<KeymapAction>, Option<Duration>)> {
    bindings
        .into_iter()
        .map(|(name, cfg)| (name, cfg.to_runtime()))
        .collect()
}

// Helper to reuse Actions deserializer
pub fn deserialize_actions_vec<'de, D>(deserializer: D) -> Result<Vec<KeymapAction>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Actions::deserialize(deserializer)?.into_vec())
}

