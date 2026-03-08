use crate::config::application::deserialize_string_or_vec;
use crate::config::application::OnlyOrNot;
use crate::config::key_press::KeyPress;
use crate::config::keymap_action::{Actions, KeymapAction};
use evdev::KeyCode as Key;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

use super::device::Device;
use super::key_press::Modifier;

// Config interface
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Keymap {
    #[allow(dead_code)]
    #[serde(default = "String::new")]
    pub name: String,
    #[serde(deserialize_with = "deserialize_remap")]
    pub remap: HashMap<KeyPress, KeymapActionsSplit>,
    pub application: Option<OnlyOrNot>,
    pub window: Option<OnlyOrNot>,
    pub device: Option<Device>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub mode: Option<Vec<String>>,
    #[serde(default)]
    pub exact_match: bool,
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

// Internals for efficient keymap lookup
#[derive(Clone, Debug)]
pub struct KeymapEntry {
    pub press_actions: Vec<KeymapAction>,
    pub release_actions: Vec<KeymapAction>,
    pub repeat_actions: Vec<KeymapAction>,
    pub modifiers: Vec<Modifier>,
    pub application: Option<OnlyOrNot>,
    pub title: Option<OnlyOrNot>,
    pub device: Option<Device>,
    pub mode: Option<Vec<String>>,
    pub exact_match: bool,
}

// Convert an array of keymaps to a single hashmap whose key is a triggering key.
//
// For each key, Vec<KeymapEntry> is scanned once, matching the exact modifiers,
// and then it's scanned again, allowing extra modifiers.
//
// First matching KeymapEntry wins at each iteration.
pub fn build_keymap_table(keymaps: &Vec<Keymap>) -> HashMap<Key, Vec<KeymapEntry>> {
    let mut table: HashMap<Key, Vec<KeymapEntry>> = HashMap::new();
    for keymap in keymaps {
        for (key_press, actions) in keymap.remap.iter() {
            let mut entries: Vec<KeymapEntry> = match table.get(&key_press.key) {
                Some(entries) => entries.to_vec(),
                None => vec![],
            };
            entries.push(KeymapEntry {
                press_actions: actions.on_press.clone(),
                release_actions: actions.on_release.clone(),
                repeat_actions: actions.on_repeat.clone(),
                modifiers: key_press.modifiers.clone(),
                application: keymap.application.clone(),
                title: keymap.window.clone(),
                device: keymap.device.clone(),
                mode: keymap.mode.clone(),
                exact_match: keymap.exact_match,
            });
            table.insert(key_press.key, entries);
        }
    }
    table
}

// Subset of KeymapEntry for override_remap
#[derive(Clone)]
pub struct OverrideEntry {
    pub press_actions: Vec<KeymapAction>,
    pub release_actions: Vec<KeymapAction>,
    pub repeat_actions: Vec<KeymapAction>,
    pub modifiers: Vec<Modifier>,
    pub exact_match: bool,
}

// This is executed on runtime unlike build_keymap_table, but hopefully not called so often.
pub fn build_override_table(
    remap: &HashMap<KeyPress, KeymapActionsSplit>,
    exact_match: bool,
) -> HashMap<Key, Vec<OverrideEntry>> {
    let mut table: HashMap<Key, Vec<OverrideEntry>> = HashMap::new();
    for (key_press, actions) in remap.iter() {
        let mut entries: Vec<OverrideEntry> = match table.get(&key_press.key) {
            Some(entries) => entries.to_vec(),
            None => vec![],
        };
        entries.push(OverrideEntry {
            press_actions: actions.on_press.clone(),
            release_actions: actions.on_release.clone(),
            repeat_actions: actions.on_repeat.clone(),
            modifiers: key_press.modifiers.clone(),
            exact_match,
        });
        table.insert(key_press.key, entries);
    }
    table
}

#[derive(Clone, Debug, Default)]
pub struct KeymapActionsSplit {
    pub on_press: Vec<KeymapAction>,
    pub on_release: Vec<KeymapAction>,
    pub on_repeat: Vec<KeymapAction>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum KeymapActionsRaw {
    Simple(Actions),
    Split {
        #[serde(default)]
        on_press: Actions,
        #[serde(default)]
        on_release: Actions,
        #[serde(default)]
        on_repeat: Actions,
    },
}

impl KeymapActionsRaw {
    pub fn into_split(self) -> KeymapActionsSplit {
        match self {
            KeymapActionsRaw::Split {
                on_press,
                on_release,
                on_repeat,
            } => KeymapActionsSplit {
                on_press: on_press.into_vec(),
                on_release: on_release.into_vec(),
                on_repeat: on_repeat.into_vec(),
            },
            KeymapActionsRaw::Simple(actions) => KeymapActionsSplit {
                on_press: actions.into_vec(),
                ..Default::default()
            },
        }
    }
}
