# Roadmap: press vs release-triggered actions

This document tracks the plan to add key-release-aware command bindings (e.g., run one command on Super+K press, another when the chord ends).

## Milestones
- **M1**: Config surface
  - Add `on_press` / `on_release` (and optional `on_repeat`) action lists for a single keymap entry without breaking existing YAML/TOML.
- **M2**: Dispatcher plumbing
  - Extend keymap lookup to fire actions on releases as well as presses.
  - Track active chord state so release actions trigger when any involved physical key is released.
  - Optionally swallow underlying key-up to avoid stray events.
- **M3**: Tests
  - Unit tests for press/release command execution and swallowing behavior.
  - Config parsing test for new schema.
- **M4**: Docs & examples
  - README example for press/release commands.
  - Note migration/compat expectations.

## Open decisions
- Should underlying key-up events be suppressed automatically when release actions exist?
- Allow `on_release` to run on either trigger key or any modifier used to satisfy the chord? (Current proposal: yes.)
- Do we need debounce/timeout for stuck modifiers?

## Nice-to-haves
- Support separate `on_repeat` for long-hold behavior.
- Telemetry/log line to aid debugging (e.g., "release action fired for Super+K via KEY_LEFTMETA up").
