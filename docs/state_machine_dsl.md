State-machine key DSL: design + roadmap
=======================================

Context and goals
-----------------

We need a richer way to describe multi-step key interactions (press/release ordering, optional releases, timeouts, repeats) and to emit user-defined signals that another module will turn into actions/commands. The current keymap/remap system is prefix-based and cannot express “Super hold, Space tap/optional release, then repeat hjkl with 50 ms cadence until Super is released”. We will add a regex-like DSL for matching key events and a small signal bus that keeps event matching separate from action execution.

Non-goals
---------
- No hard-coded semantics like “nav.up”; all emitted names are user-provided strings resolved through a mapping.
- No hidden behaviour tied to specific pattern names; everything must be driven by config.
- Do not change existing keymap/remap behaviour or syntax unless the new DSL is invoked.

High-level architecture
-----------------------
1) **Matcher (state machine engine)** lives in `EventHandler`, before normal keymap lookup. It parses DSL patterns into NFAs and drives them with incoming key press/release events.
2) **Signals**: Matcher emits lightweight `Signal { name: String, kind: Fire | StartRepeat | StopRepeat, repeat: Option<RepeatSpec> }`.
3) **Signal dispatcher**: New module (likely `signal_dispatcher.rs`) that consumes signals, looks up user-defined bindings, and pushes existing `Action`s (commands, key events, delays) into the `Action` queue.
4) **Repeat scheduler**: Shared small scheduler using the existing timerfd to trigger repeat actions at configured intervals until a StopRepeat arrives or the pattern exits.

DSL sketch (regex-inspired)
---------------------------
Tokens:
- `KEY` = press; `KEY!` = release.
- Concatenation = sequence; `|` = alternation; `()` = group; `*`/`+`/`?` = usual regex quantifiers; `{n}`, `{n,m}` counts.
- `~Xms` after a token/group = timeout to reach that point.
- `=> action` attaches actions to a token/group completion. Actions can be `emit(name)` (signal), `key(press|release|repeat KEY)`, `command([...])`, `set_mode(...)`, etc.
- “Noise/no-op” is just another branch that emits nothing (e.g., `(Space!|j!|k!) => noop`), replacing the special “allow” notion.
- `end_on(K1!|K2!) => ...` sugar for a trailing branch that, on those releases, finalises the machine and runs the attached actions.

Example patterns
----------------
Super+g with optional g release, ending on Super release:
```
pattern SuperG =
  Super g (g!|ε)
  (h|j|k|l)+ => emit(user.nav.<key>)   # user maps signals to actions
  end_on(Super!) => emit(user.nav.stop)
  (g!) => noop                         # consume g release without ending
```

Navigator with noise as explicit branch (no special “allow”):
```
pattern SuperSpaceNav =
  Super Space ~400ms
  (
      j+ => emit(user.down)
    | k+ => emit(user.up)
    | h+ => emit(user.left)
    | l+ => emit(user.right)
    | (Space!|j!|k!|h!|l!) => noop   # noise branch
  )*
  end_on(Super!) => emit(user.stop)
```

Tap/hold prefix that aborts on unexpected input, but tolerates modifier releases:
```
pattern CleanPrefix =
  Super p
  (Shift!|Control!|Alt!) => noop
  end_on(Super!|p!) => emit(user.prefix.fire)
```

Signal binding (separate from patterns)
---------------------------------------
Config section maps signal names to actions, keeping patterns declarative:
```
signals:
  user.down:
    repeat: true
    interval_ms: 50
    actions: [{ command: ["hyperctl", "resize", "down"] }]
  user.stop:
    actions: [{ command: ["hyperctl", "stop-nav"] }]
```
If a signal is undefined, it is ignored (log debug).

Data structures (planned)
-------------------------
- `struct Pattern { name: String, ast: AstNode, … }`
- `enum AstNode { KeyEdge{key, edge}, Seq(Vec<AstNode>), Alt(Vec<AstNode>), Repeat{node, kind}, Timeout{node,dur}, Action{node, actions}, Noise, EndOn{keys, actions}, … }`
- Runtime matcher keeps an NFA state set + per-branch timers + active repeats keyed by (pattern, signal-name).
- `Signal` enum as described above.

Integration points
------------------
- `Config` loader: parse new `patterns:` block and optional `signals:` block; compile patterns to internal form; build lookup by starting key (and modifiers) for quick activation.
- `EventHandler::on_key_event`: before normal keymap search, feed event into active machines; if none active and event matches a pattern start key, spawn a machine.
- `ActionDispatcher`: unchanged; it will just receive more `Action::Command` / key events from the signal dispatcher.
- Timer reuse: extend the existing timerfd to service both remap timeouts and repeat schedules (next-expiration min-heap).

Edge cases to handle
--------------------
- Multiple active machines (different start keys) overlapping.
- Cancel on unexpected events unless a “noise” branch catches them.
- Mode/application/device guards should still apply when patterns are started from keymap entries.
- Cleanup: on pattern exit, stop all repeats started by that pattern.

Roadmap (kept up-to-date while iterating)
-----------------------------------------
1) **Baseline & tooling**
   - Duplicate flake and confirm `nix build` (done).
   - Run `cargo test` to establish a clean baseline (done).
2) **Config & parsing**
   - Add `patterns:` and `signals:` sections to config schema.
   - Implement DSL parser (Pratt or small recursive-descent) to AST; compile to NFA.
   - Unit tests for parsing and NFA construction.
3) **Signal layer**
   - Define `Signal` and `SignalBinding` structs.
   - Implement dispatcher that maps signals to actions and manages repeat schedules.
   - Tests: start/stop repeat; undefined signal; multiple signals.
4) **Matcher runtime**
   - Integrate into `EventHandler`: activation, stepping, noise handling, end_on, timeouts.
   - Ensure existing remap/keymap paths stay unchanged when no patterns active.
   - Tests: golden event sequences for nav example, Super+g example, timeout abort, noise branch.
5) **Timer integration**
   - Extend timerfd loop to handle both override timeouts and signal repeats (min-heap of next expirations).
   - Tests: repeat cadence within tolerance.
6) **Docs & examples**
   - Add README section with DSL examples (regex style only).
   - Provide sample config showing signals mapping to commands (no hard-coded names in code).
7) **Polish**
   - Logging, error messages, config validation.
   - Benchmarks/quick perf sanity if needed.
8) **Cleanup**
   - Remove temporary scaffolding; ensure formatting and clippy.
   - Update design doc if plan adjusted; keep roadmap section accurate.

Notes for future tweaks
-----------------------
- If the timerfd multiplexing gets messy, we can create a dedicated repeat scheduler thread, but initial plan is single timerfd.
- If users request, we can add a block-y syntax later, but we’ll keep the primary DSL as the regex-ish inline form.

