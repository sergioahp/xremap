use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use std::fs;
use std::path::PathBuf;

/// Split xremap config files with descriptions into clean config and dmenu-friendly files
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Input YAML config file with descriptions
    #[clap(value_parser)]
    input: PathBuf,

    /// Output clean config file (without descriptions)
    #[clap(short = 'c', long, value_parser)]
    config_output: PathBuf,

    /// Output dmenu-friendly file (with keymaps and descriptions)
    #[clap(short = 'd', long, value_parser)]
    dmenu_output: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DmenuBinding {
    /// The key combination (e.g., "C-a", "Super-t")
    binding: String,

    /// Human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    /// The action to perform
    #[serde(skip_serializing_if = "Option::is_none")]
    action: Option<Value>,

    /// Mode this binding belongs to
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<ModeValue>,

    /// Name of the keymap/modmap
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    /// Application filter
    #[serde(skip_serializing_if = "Option::is_none")]
    application: Option<Value>,

    /// Window filter
    #[serde(skip_serializing_if = "Option::is_none")]
    window: Option<Value>,

    /// Device filter
    #[serde(skip_serializing_if = "Option::is_none")]
    device: Option<Value>,

    /// Exact match flag (keymap only)
    #[serde(skip_serializing_if = "Option::is_none")]
    exact_match: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum ModeValue {
    Single(String),
    Multiple(Vec<String>),
}

impl ModeValue {
    fn to_string(&self) -> String {
        match self {
            ModeValue::Single(s) => s.clone(),
            ModeValue::Multiple(v) => v.join(", "),
        }
    }
}

#[derive(Debug, Serialize)]
struct DmenuOutput {
    bindings: Vec<DmenuBinding>,
}

fn extract_description(value: &mut Value) -> Option<String> {
    if let Value::Mapping(map) = value {
        if let Some(desc) = map.remove(&Value::String("description".to_string())) {
            if let Value::String(s) = desc {
                return Some(s);
            }
        }
    }
    None
}

fn process_remap(
    remap: &Value,
    bindings: &mut Vec<DmenuBinding>,
    name: Option<String>,
    mode: Option<ModeValue>,
    application: Option<Value>,
    window: Option<Value>,
    device: Option<Value>,
    exact_match: Option<bool>,
) -> Value {
    let mut clean_remap = Mapping::new();

    if let Value::Mapping(map) = remap {
        for (key, value) in map.iter() {
            if let Value::String(key_str) = key {
                let mut value_clone = value.clone();
                let description = extract_description(&mut value_clone);

                // For launch actions, we might have description at the same level
                let action_clone = if let Value::Mapping(action_map) = &value_clone {
                    // Check if this is a launch action with description
                    let mut new_map = action_map.clone();
                    new_map.remove(&Value::String("description".to_string()));
                    Value::Mapping(new_map)
                } else {
                    value_clone.clone()
                };

                // Add to dmenu bindings
                bindings.push(DmenuBinding {
                    binding: key_str.clone(),
                    description,
                    action: Some(action_clone.clone()),
                    mode: mode.clone(),
                    name: name.clone(),
                    application: application.clone(),
                    window: window.clone(),
                    device: device.clone(),
                    exact_match,
                });

                // Add to clean config
                clean_remap.insert(key.clone(), action_clone);
            }
        }
    }

    Value::Mapping(clean_remap)
}

fn process_keymap_or_modmap(
    items: &Value,
    bindings: &mut Vec<DmenuBinding>,
) -> Result<Value> {
    let mut clean_items = Vec::new();

    if let Value::Sequence(seq) = items {
        for item in seq {
            if let Value::Mapping(map) = item {
                let mut clean_map = Mapping::new();

                let name = map
                    .get(&Value::String("name".to_string()))
                    .and_then(|v| {
                        if let Value::String(s) = v {
                            Some(s.clone())
                        } else {
                            None
                        }
                    });

                let mode = map
                    .get(&Value::String("mode".to_string()))
                    .map(|v| serde_yaml::from_value::<ModeValue>(v.clone()))
                    .transpose()?;

                let application = map
                    .get(&Value::String("application".to_string()))
                    .cloned();

                let window = map.get(&Value::String("window".to_string())).cloned();

                let device = map.get(&Value::String("device".to_string())).cloned();

                let exact_match = map
                    .get(&Value::String("exact_match".to_string()))
                    .and_then(|v| {
                        if let Value::Bool(b) = v {
                            Some(*b)
                        } else {
                            None
                        }
                    });

                for (key, value) in map.iter() {
                    if let Value::String(key_str) = key {
                        if key_str == "remap" {
                            let clean_remap = process_remap(
                                value,
                                bindings,
                                name.clone(),
                                mode.clone(),
                                application.clone(),
                                window.clone(),
                                device.clone(),
                                exact_match,
                            );
                            clean_map.insert(key.clone(), clean_remap);
                        } else {
                            clean_map.insert(key.clone(), value.clone());
                        }
                    }
                }

                clean_items.push(Value::Mapping(clean_map));
            }
        }
    }

    Ok(Value::Sequence(clean_items))
}

fn process_config(input_value: &Value) -> Result<(Value, DmenuOutput)> {
    let mut bindings = Vec::new();
    let mut clean_config = Mapping::new();

    if let Value::Mapping(map) = input_value {
        for (key, value) in map.iter() {
            if let Value::String(key_str) = key {
                match key_str.as_str() {
                    "keymap" | "modmap" => {
                        let clean_items = process_keymap_or_modmap(value, &mut bindings)?;
                        clean_config.insert(key.clone(), clean_items);
                    }
                    _ => {
                        clean_config.insert(key.clone(), value.clone());
                    }
                }
            }
        }
    }

    let clean_value = Value::Mapping(clean_config);
    let dmenu_output = DmenuOutput { bindings };

    Ok((clean_value, dmenu_output))
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read input file
    let input_content = fs::read_to_string(&args.input)
        .context(format!("Failed to read input file: {:?}", args.input))?;

    // Parse YAML
    let input_value: Value = serde_yaml::from_str(&input_content)
        .context("Failed to parse input YAML")?;

    // Process config
    let (clean_config, dmenu_output) = process_config(&input_value)?;

    // Write clean config
    let clean_yaml = serde_yaml::to_string(&clean_config)
        .context("Failed to serialize clean config")?;
    fs::write(&args.config_output, clean_yaml)
        .context(format!("Failed to write config output: {:?}", args.config_output))?;

    // Write dmenu output
    let dmenu_yaml = serde_yaml::to_string(&dmenu_output)
        .context("Failed to serialize dmenu output")?;
    fs::write(&args.dmenu_output, dmenu_yaml)
        .context(format!("Failed to write dmenu output: {:?}", args.dmenu_output))?;

    println!("✓ Generated clean config: {:?}", args.config_output);
    println!("✓ Generated dmenu file: {:?}", args.dmenu_output);
    println!("✓ Extracted {} bindings", dmenu_output.bindings.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_description() {
        let mut value = serde_yaml::from_str::<Value>(
            r#"
            launch: [xterm]
            description: "Open terminal"
            "#,
        )
        .unwrap();

        let desc = extract_description(&mut value);
        assert_eq!(desc, Some("Open terminal".to_string()));

        // Description should be removed
        if let Value::Mapping(map) = &value {
            assert!(!map.contains_key(&Value::String("description".to_string())));
        }
    }

    #[test]
    fn test_process_simple_keymap() {
        let input = serde_yaml::from_str::<Value>(
            r#"
            keymap:
              - name: Test
                remap:
                  C-a: Home
                  C-e: End
            "#,
        )
        .unwrap();

        let (clean, dmenu) = process_config(&input).unwrap();

        // Check clean config
        let clean_str = serde_yaml::to_string(&clean).unwrap();
        assert!(clean_str.contains("C-a"));
        assert!(clean_str.contains("Home"));
        assert!(!clean_str.contains("description"));

        // Check dmenu output
        assert_eq!(dmenu.bindings.len(), 2);
        assert!(dmenu.bindings.iter().any(|b| b.binding == "C-a"));
        assert!(dmenu.bindings.iter().any(|b| b.binding == "C-e"));
    }

    #[test]
    fn test_process_keymap_with_descriptions() {
        let input = serde_yaml::from_str::<Value>(
            r#"
            keymap:
              - name: Launch
                mode: default
                remap:
                  Super-t:
                    launch: [xterm]
                    description: "Open terminal"
                  Super-b:
                    launch: [firefox]
                    description: "Open browser"
            "#,
        )
        .unwrap();

        let (clean, dmenu) = process_config(&input).unwrap();

        // Check clean config
        let clean_str = serde_yaml::to_string(&clean).unwrap();
        assert!(!clean_str.contains("description"));
        assert!(clean_str.contains("Super-t"));
        assert!(clean_str.contains("xterm"));

        // Check dmenu output
        assert_eq!(dmenu.bindings.len(), 2);

        let terminal_binding = dmenu.bindings.iter()
            .find(|b| b.binding == "Super-t")
            .expect("Super-t binding not found");
        assert_eq!(terminal_binding.description, Some("Open terminal".to_string()));
        assert_eq!(terminal_binding.name, Some("Launch".to_string()));

        let browser_binding = dmenu.bindings.iter()
            .find(|b| b.binding == "Super-b")
            .expect("Super-b binding not found");
        assert_eq!(browser_binding.description, Some("Open browser".to_string()));
    }

    #[test]
    fn test_process_with_application_filter() {
        let input = serde_yaml::from_str::<Value>(
            r#"
            keymap:
              - name: Emacs
                application:
                  not: [Emacs, Terminal]
                mode: [default, insert]
                remap:
                  C-a:
                    action: Home
                    description: "Move to beginning of line"
            "#,
        )
        .unwrap();

        let (clean, dmenu) = process_config(&input).unwrap();

        // Check dmenu output has metadata
        assert_eq!(dmenu.bindings.len(), 1);
        let binding = &dmenu.bindings[0];
        assert_eq!(binding.binding, "C-a");
        assert_eq!(binding.description, Some("Move to beginning of line".to_string()));
        assert_eq!(binding.name, Some("Emacs".to_string()));
        assert!(binding.application.is_some());
        assert!(binding.mode.is_some());

        // Check clean config doesn't have description
        let clean_str = serde_yaml::to_string(&clean).unwrap();
        assert!(!clean_str.contains("Move to beginning"));
    }

    #[test]
    fn test_modmap_processing() {
        let input = serde_yaml::from_str::<Value>(
            r#"
            modmap:
              - name: Global
                remap:
                  CapsLock: Esc
                  Alt_L: Ctrl_L
            "#,
        )
        .unwrap();

        let (clean, dmenu) = process_config(&input).unwrap();

        // Check dmenu has modmap bindings
        assert_eq!(dmenu.bindings.len(), 2);
        assert!(dmenu.bindings.iter().any(|b| b.binding == "CapsLock"));
        assert!(dmenu.bindings.iter().any(|b| b.binding == "Alt_L"));
    }
}
