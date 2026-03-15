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

High-level architecture (current state)
---------------------------------------
1) **Matcher (state machine engine)** implemented: DSL → AST → NFA; runs inside `EventHandler` before normal keymap lookup.
2) **Signals**: DSL actions produce `Signal { name, kind: Fire | StartRepeat | StopRepeat }`.
3) **Signal dispatcher**: Implemented (`src/signal.rs`). Maps signals to user `signals:` bindings (actions + optional repeat interval) and injects those actions into the normal `Action` queue.
4) **Repeat scheduler**: Implemented. Uses the existing timerfd (now dual-purpose) to drive repeats until `StopRepeat` or pattern exit.

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
1) **Baseline & tooling** — done (nix build, cargo test).
2) **Config & parsing** — done: `patterns:` and `signals:`, parser+NFA, parser unit test.
3) **Signal layer** — done: dispatcher with repeat scheduling, dispatcher unit test.
4) **Matcher runtime** — initial integration done (patterns run before keymap). TODO: timeouts (`~Xms`), explicit `end_on` sugar (currently use `=> end`), nicer noise sugar (currently use `noop` branch).
5) **Timer integration** — done: signal repeats share timerfd; event loop handles `SignalTimeout`.
6) **Docs & examples** — TODO: user-facing README section + sample configs wired to signals.
7) **Polish** — TODO: better errors/logging, config validation, reduce dead-code warnings.
8) **Tests** — added parser + signal dispatcher + minimal pattern→signal integration test; still need golden nav / Super+g sequences and timeout abort coverage.

Notes for future tweaks
-----------------------
- If the timerfd multiplexing gets messy, we can create a dedicated repeat scheduler thread, but initial plan is single timerfd.
- If users request, we can add a block-y syntax later, but we’ll keep the primary DSL as the regex-ish inline form.
