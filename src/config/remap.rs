use evdev::KeyCode as Key;
use serde::{Deserialize, Deserializer};

use crate::config::application::deserialize_string_or_vec;
use crate::config::key_press::KeyPress;
use crate::config::keymap::{KeymapActionsRaw, KeymapActionsSplit};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Remap {
    pub remap: HashMap<KeyPress, KeymapActionsSplit>,
    pub timeout: Option<Duration>,
    pub timeout_key: Option<Vec<Key>>,
}

// USed only for deserialization
#[derive(Debug, Deserialize)]
pub struct RemapActions {
    #[serde(deserialize_with = "deserialize_remap")]
    pub remap: HashMap<KeyPress, KeymapActionsSplit>,
    pub timeout_millis: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub timeout_key: Option<Vec<String>>,
}

fn deserialize_remap<'de, D>(deserializer: D) -> Result<HashMap<KeyPress, KeymapActionsSplit>, D::Error>
where
    D: Deserializer<'de>,
{
    let remap = HashMap::<KeyPress, KeymapActionsRaw>::deserialize(deserializer)?;
    Ok(remap
        .into_iter()
        .map(|(key_press, actions)| (key_press, actions.into_split()))
        .collect())
}
