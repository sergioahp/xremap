pub mod application;
pub mod device;
mod key;
pub mod key_press;
pub mod keymap;
pub mod keymap_action;
mod modmap;
pub mod modmap_action;
pub mod signal;
pub use key::{parse_key, DISGUISED_EVENT_OFFSETTER, KEY_MATCH_ANY};
pub mod socket;

pub mod remap;
#[cfg(test)]
mod tests;

extern crate serde_yaml;
extern crate toml;

use crate::pattern::{compile_patterns, edge_key, CompiledPattern};
use evdev::KeyCode as Key;
use keymap::Keymap;
use modmap::Modmap;
use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};
use serde::{de::IgnoredAny, Deserialize, Deserializer};
use std::{
    collections::HashMap,
    error, fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use self::{keymap::{build_keymap_table, KeymapEntry}, keymap_action::KeymapAction, signal::parse_signal_bindings};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    // Config interface
    #[serde(default = "Vec::new")]
    pub modmap: Vec<Modmap>,
    #[serde(default = "Vec::new")]
    pub keymap: Vec<Keymap>,
    #[serde(default = "HashMap::new")]
    pub signals: HashMap<String, signal::SignalBindingConfig>,
    #[serde(default = "HashMap::new")]
    pub patterns: HashMap<String, String>,
    #[serde(default)]
    pub socket_path: Option<String>,
    #[serde(default = "default_mode")]
    pub default_mode: String,
    #[serde(deserialize_with = "deserialize_virtual_modifiers", default = "Vec::new")]
    pub virtual_modifiers: Vec<Key>,
    #[serde(default)]
    pub keypress_delay_ms: u64,

    // Data is not used by any part of the application.
    // but can be used with Anchors and Aliases
    #[allow(dead_code)]
    #[serde(default)]
    pub shared: IgnoredAny,

    // Internals
    #[serde(skip)]
    pub modify_time: Option<SystemTime>,
    #[serde(skip)]
    pub keymap_table: HashMap<Key, Vec<KeymapEntry>>,
    #[serde(default = "const_true")]
    pub enable_wheel: bool,
    #[serde(skip)]
    pub signal_bindings: HashMap<String, (Vec<KeymapAction>, Option<std::time::Duration>)>,
    #[serde(skip)]
    pub compiled_patterns: Vec<CompiledPattern>,
    #[serde(skip)]
    pub pattern_start_table: HashMap<(Key, bool), Vec<usize>>,
    #[serde(skip)]
    pub socket_path_runtime: Option<String>,
}

enum ConfigFiletype {
    Yaml,
    Toml,
}

fn get_file_ext(filename: &Path) -> ConfigFiletype {
    match filename.extension() {
        Some(f) => {
            if f.to_str().unwrap_or("").to_lowercase() == "toml" {
                ConfigFiletype::Toml
            } else {
                ConfigFiletype::Yaml
            }
        }
        _ => ConfigFiletype::Yaml,
    }
}

pub fn load_configs(filenames: &[PathBuf]) -> Result<Config, Box<dyn error::Error>> {
    // Assumes filenames is non-empty
    let config_contents = fs::read_to_string(&filenames[0])?;

    let mut config: Config = match get_file_ext(&filenames[0]) {
        ConfigFiletype::Yaml => serde_yaml::from_str(&config_contents)?,
        ConfigFiletype::Toml => toml::from_str(&config_contents)?,
    };

    for filename in &filenames[1..] {
        let config_contents = fs::read_to_string(filename)?;
        let c: Config = match get_file_ext(filename) {
            ConfigFiletype::Yaml => serde_yaml::from_str(&config_contents)?,
            ConfigFiletype::Toml => toml::from_str(&config_contents)?,
        };

        config.modmap.extend(c.modmap);
        config.keymap.extend(c.keymap);
        config.virtual_modifiers.extend(c.virtual_modifiers);
        config.signals.extend(c.signals);
    config.patterns.extend(c.patterns);
    if config.socket_path.is_none() {
        config.socket_path = c.socket_path;
    }
    }

    // Timestamp for --watch=config
    config.modify_time = filenames.last().and_then(|path| path.metadata().ok()?.modified().ok());

    // Convert keymap for efficient keymap lookup
    config.keymap_table = build_keymap_table(&config.keymap);
    // Prepare signal bindings (runtime form)
    config.signal_bindings = parse_signal_bindings(config.signals.clone());
    config.socket_path_runtime = config.socket_path.clone();
    // Compile patterns
    let compiled = compile_patterns(&config.patterns)?;
    let mut table: HashMap<(Key, bool), Vec<usize>> = HashMap::new();
    for (idx, pat) in compiled.iter().enumerate() {
        for edge in &pat.start_edges {
            table.entry(edge_key(edge)).or_default().push(idx);
        }
    }
    config.pattern_start_table = table;
    config.compiled_patterns = compiled;

    Ok(config)
}

pub fn config_watcher(watch: bool, files: &Vec<PathBuf>) -> anyhow::Result<Option<Inotify>> {
    if watch {
        let inotify = Inotify::init(InitFlags::IN_NONBLOCK)?;
        for file in files {
            inotify.add_watch(
                file.parent().expect("config file has a parent directory"),
                AddWatchFlags::IN_CREATE | AddWatchFlags::IN_MOVED_TO,
            )?;
            inotify.add_watch(file, AddWatchFlags::IN_MODIFY)?;
        }
        Ok(Some(inotify))
    } else {
        Ok(None)
    }
}

fn default_mode() -> String {
    "default".to_string()
}

fn deserialize_virtual_modifiers<'de, D>(deserializer: D) -> Result<Vec<Key>, D::Error>
where
    D: Deserializer<'de>,
{
    let key_strs = Vec::<String>::deserialize(deserializer)?;
    let mut keys: Vec<Key> = vec![];
    for key_str in key_strs {
        keys.push(parse_key(&key_str).map_err(serde::de::Error::custom)?);
    }
    Ok(keys)
}

fn const_true() -> bool {
    true
}
