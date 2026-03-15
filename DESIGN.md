# xremap pattern DSL — design document

This document describes the architecture of the pattern state-machine layer
added on top of upstream xremap.  It covers the DSL syntax, the compiler
pipeline, the runtime execution model, the frame stack, state broadcasting,
and the subtle correctness properties that have bitten us.

---

## 1. Motivation

Upstream xremap supports key remapping and simple chords, but not **modal**
or **sequential** bindings that span multiple key events over time.  The
pattern layer adds a regex-like DSL that is compiled to a non-deterministic
finite automaton (NFA) and run in the event handler.

Typical use case — WinMgmt mode:

```
Super_L d => noop
( j => emit(wm.swap_next)
| k => emit(wm.swap_prev)
| c => emit(wm.kill)
| d! => push_frame(c => [emit(wm.kill), end])
| any => noop
)*
end_on(Super_L!) => noop
```

Holding `Super+D` enters a window-management mode where `j/k/c/…` fire
window actions.  Releasing `D` (but keeping `Super`) narrows the mode to
only `C` (kill), then any result exits the mode.

---

## 2. Pattern DSL syntax

```
pattern  ::= seq ( '|' seq )*
seq      ::= atom*
atom     ::= primary quantifier? ( '=>' actions )?
primary  ::= key_token       -- press event: KEY_NAME
           | key_token '!'   -- release event
           | 'any'           -- any press or release
           | '(' pattern ')' -- grouping
           | 'end_on' '(' edge ('|' edge)* ')'
           | 'ε'             -- explicit epsilon
quantifier ::= '*' | '+' | '?'
actions  ::= action | '[' action (',' action)* ']'
action   ::= 'emit' '(' signal_name ')'
           | 'emit_start' '(' signal_name ')'
           | 'emit_stop' '(' signal_name ')'
           | 'noop'
           | 'end'
           | 'push_frame'
           | 'push_frame' '(' pattern ')'
```

`end_on(k!)` is sugar for an `Alt` of release edges each carrying an `End`
action.  It causes a full reset of the pattern machine when matched.

`push_frame(expr)` is compiled inline: the sub-expression is compiled into a
sub-NFA fragment embedded in the same state graph, and at runtime the machine
switches to that fragment's start state (see §5).

---

## 3. Compiler pipeline

```
source string
    │
    ▼  parser.rs
  AST (Node / NodeKind)
    │
    ▼  nfa.rs  Builder
  NFA (states, transitions, start)
    │
    ▼  pattern/mod.rs  compile_fused_nfa()
  Fused NFA  (all patterns merged into one Alt machine)
```

### 3.1 AST

`Node` carries a `NodeKind` and a `Vec<ActionSpec>` (actions that fire when
the node is traversed):

```rust
enum NodeKind {
    Epsilon,
    Event(Edge),          // a single key edge
    Seq(Vec<Node>),
    Alt(Vec<Node>),
    Repeat { node, kind }, // *, +, ?
    Timeout { node, duration },
}

enum Edge { Press(Key), Release(Key), Any }

enum ActionSpec {
    Emit(String, SignalKind),
    Noop,
    End,
    PushFrame,             // save current states as restore point
    PushFrameOf(Box<Node>),// inline sub-pattern (compile-time)
    PushFrameAt(usize),    // runtime: switch to this NFA state
}
```

`PushFrameOf` exists only in the AST; the compiler resolves it to
`PushFrameAt(sub_start)` by building the sub-expression inline into the
same Builder and recording its start state.

### 3.2 NFA compilation (Thompson construction)

`Builder` allocates states sequentially.  Each `NodeKind` maps to a fragment
with a start and end state:

| NodeKind | Fragment |
|---|---|
| Epsilon | s --ε-→ e |
| Event(edge) | s --edge-→ e |
| Seq([n₁…nₙ]) | chain: eᵢ --ε-→ sᵢ₊₁ |
| Alt([n₁…nₙ]) | fork: start --ε-→ sᵢ, eᵢ --ε-→ end |
| Repeat ZeroOrMore | start --ε-→ {end, s}; e --ε-→ {s, end} |
| Repeat OneOrMore | start --ε-→ s; e --ε-→ {s, end} |
| Repeat Optional | start --ε-→ {end, s}; e --ε-→ end |

Actions on a `Node` are carried on the epsilon transition out of the node's
start state.

### 3.3 Fused NFA

All named patterns are compiled into **one shared NFA** via
`compile_fused_nfa()`.  A single Alt-style start state fans out to each
pattern's sub-NFA via epsilon transitions.  This lets one `Machine` run all
patterns simultaneously without maintaining a dispatch table.

Crucially, each pattern's states occupy a **contiguous half-open range
`[start, end)`** in the state array (because the `Builder` allocates
sequentially and each pattern is compiled atomically).  This range is stored
in `nfa.pattern_ranges: HashMap<String, (usize, usize)>` and is used at
runtime to identify which pattern is currently active (for state broadcasting).

---

## 4. Runtime machine

`Machine` holds a reference to the NFA and a `current: HashSet<usize>` of
active states (the NFA's "thread set").

### 4.1 Epsilon closure

States are always stored **post-epsilon-closure**: whenever a state is added
to `current`, all states reachable from it via ε-transitions are added
immediately (including their ε-action side-effects).  This keeps `step()`
simple.

### 4.2 Stepping

```
fn step(edge) -> StepResult { actions, alive, ended, consumed }
```

For each active state, check every transition.  If the transition's edge
matches (or is `Edge::Any`), follow it, add the target's ε-closure to
`next_states`, and collect actions.  `consumed = true` if any transition
fired.  `alive = !next_states.is_empty()`.

### 4.3 Re-arming

After a full reset, if modifier keys are still physically held (e.g. Super),
the machine is stepped through those modifiers' press events so it is
positioned at the state it would be in had those keys been pressed from idle.
This allows a second pattern session to start without re-pressing the modifier.

**Critical:** re-arm must use `pattern_held_keys` (updated *before*
`process_patterns` is called), **not** `self.modifiers` (updated *after*).
Using `self.modifiers` during a reset triggered by a key release causes the
just-released key to still appear held, producing ghost pattern sessions.

---

## 5. Event handler integration

`process_patterns(edge)` is called for every key press/release (not repeat).
It returns `true` if the event was consumed by the pattern machine.

### 5.1 Two-phase model: recognition vs committed

The frame stack (`pattern_frame_stack`) is the boundary:

- **Empty stack = recognition phase.**  The machine is stepping speculatively.
  Non-modifier consumed events are buffered in `pattern_speculative_buffer`.
  If the pattern ultimately fails, the buffer is replayed to the virtual
  device so no events are lost.

- **Non-empty stack = committed.**  The first time a step produces real
  actions (not just NFA/meta actions) while in recognition phase, the current
  machine states are saved as a frame and the buffer is discarded (committed).

### 5.2 Frame stack

Each frame is `(nfa_states: HashSet<usize>, anchor_keys: HashSet<Key>)`.

- `nfa_states` is the machine's `current` at the moment the frame was pushed.
  Used to restore the machine on partial reset.
- `anchor_keys` is `pattern_held_keys` at push time — the set of keys that
  must still be held for this frame to remain valid.

On failure (`!alive`):

1. Walk the stack from the top.
2. If the top frame's `anchor_keys ⊆ pattern_held_keys`, restore
   `machine.current` from that frame and return (partial reset).
3. Otherwise pop it and continue.
4. If the stack is exhausted, do a full reset.

On `end` action: skip the stack walk, go straight to full reset.

### 5.3 PushFrame vs PushFrameAt

`PushFrame` (no args): pushes a copy of the current machine states as a new
restore frame.  The machine keeps running in the same state space.  Used to
implement "sticky restart" — if a pattern segment fails, fall back to the
loop rather than full-resetting.

`PushFrameAt(sub_start)` (from `push_frame(expr)`): does **not** push any
restore frame.  It only switches `machine.current` to the ε-closure of
`sub_start`.  The existing commit frame (anchor = keys held at D press time)
already handles restoration.  After the anchor key (D) is released, that
frame's anchor becomes invalid, so any failure in the sub-NFA causes a full
reset rather than falling back to the main loop.

This distinction is the fix for the "j fires swap_next after D released" bug:
adding a new restore frame with anchor = `{Super}` (keys held at D-release
time) would keep the main loop accessible even without D held.

### 5.4 Key repeat suppression

`value == 2` (repeat) events bypass `process_patterns` entirely (only
press/release drive the NFA).  Without extra tracking, a key consumed on
press would have its repeat events fall through to the virtual device.

`pattern_suppressed_keys: HashSet<Key>` tracks non-modifier keys whose press
was consumed.  Repeat events for suppressed keys are dropped.  The key is
removed from the set when its release is processed.

---

## 6. Signal system

Actions fire **signals** (`Emit(name, kind)`).  Signals are named event
channels defined in the config:

```yaml
signals:
  wm.swap_next:
    actions:
      - { run: ["hyprctl", "dispatch", "swapnext"] }
    repeat: false
```

`SignalKind`: `Fire` (one-shot), `StartRepeat`, `StopRepeat`.  Repeating
signals use a `TimerFd` to re-fire at a configured interval.

The signal layer decouples the pattern DSL from the actual actions, allowing
the same signal to be fired from multiple patterns or from keymap rules.

---

## 7. State socket

xremap can broadcast its internal pattern-machine state over a Unix domain
socket as newline-delimited JSON.  This lets external programs (status bars,
monitors) observe mode transitions without polling.

### 7.1 Configuration

```yaml
state_socket: /tmp/xremap-state.sock
```

### 7.2 Wire format

Each state change emits one JSON line:

```jsonc
// Idle — no pattern active
{"type":"idle"}

// Active — a pattern has committed
{
  "type": "active",
  "pattern": "WinMgmt",       // identified from nfa.pattern_ranges
  "frame": 1,                 // pattern_frame_stack.len()
  "anchor_keys": ["KEY_D", "KEY_LEFTMETA"],  // top frame's anchor
  "held_keys":   ["KEY_LEFTMETA"],           // pattern_held_keys now
  "available":   ["KEY_C", "KEY_J", "KEY_K", "KEY_D!", "any"]
                              // edges accepted from machine.current
}
```

`available` is computed by walking all transitions from the current active
states and collecting non-epsilon edge labels.  Format:
- `"KEY_J"` — press edge
- `"KEY_D!"` — release edge
- `"any"` — wildcard `Edge::Any`

### 7.3 Implementation

`StateBroadcaster` runs an acceptor thread that loops on a non-blocking
`UnixListener`, appending new connections to a `Arc<Mutex<Vec<UnixStream>>>`.
`broadcast()` acquires the lock, writes the JSON line to every client, and
silently removes any that have disconnected.

`broadcast_state()` in `EventHandler` is called at the end of every
`process_patterns()` invocation.  It computes `current_state_event()`,
compares it to `last_broadcast_state`, and only calls `broadcaster.broadcast()`
on change.  `last_broadcast_state` is a public field so tests can inspect the
computed state without a real socket.

Pattern identification: check which entry in `nfa.pattern_ranges` has a
non-empty intersection with `machine.current`.  Because patterns occupy
contiguous state ranges and only one pattern can be "active" (committed) at a
time, this is unambiguous.

---

## 8. Config reference

```yaml
# Existing xremap socket for Hyprland IPC actions
socket_path: /tmp/xremap-nav.sock

# State broadcast socket (new)
state_socket: /tmp/xremap-state.sock

signals:
  signal.name:
    repeat: false          # or true
    interval_ms: 16        # only meaningful if repeat: true
    actions:
      - { run: [...] }

patterns:
  PatternName: "DSL string"
```

A pattern string is parsed, compiled to a sub-NFA, and fused with all other
patterns into a single machine.

---

## 9. Bug history / correctness notes

These are subtle issues we hit; worth preserving so they're not re-introduced.

### 9.1 Modifier keys must be forwarded even when consumed

When a modifier (Super, Ctrl, …) is consumed by the pattern, it must still
call `update_modifier()` and `send_key()` to the virtual device.  Compositors
use the physical modifier state for other gestures (Super+click to drag
windows).  Skipping the forward causes the compositor to lose track of modifier
state.

### 9.2 PushFrameAt must not push a restore frame

If `push_frame(expr)` pushes the current loop states as a restore frame with
anchor = held keys at D-release time (`{Super}`), then when a key fails in the
sub-NFA, the stack walk finds that frame (Super still held → valid) and
restores the loop.  This makes loop actions fire even without D held.

The fix: `PushFrameAt` only switches `machine.current`; it does not push.  The
commit frame (anchor `{Super, D}`) already on the stack becomes invalid after D
is released, so sub-NFA failures cause a full reset.

### 9.3 Re-arm must use pattern_held_keys, not self.modifiers

`update_modifier(key, value)` runs *after* `process_patterns` returns.  If a
full reset is triggered *inside* `process_patterns` (e.g. `end_on` on Super
release), and we re-arm using `self.modifiers`, the just-released Super is
still present.  The machine re-arms as if Super is held; the next bare D press
then matches Super+D and commits a ghost session.

Fix: re-arm from `pattern_held_keys`, which is updated *before*
`process_patterns` is called.

### 9.4 Key repeats bypass process_patterns

`value == 2` (OS key repeat) never drives the NFA.  Without
`pattern_suppressed_keys`, a key consumed on press would have its repeats fall
through to the virtual device.  Track consumed non-modifier keys; drop their
repeats; remove on release.

### 9.5 Speculative buffer replay on pattern failure

In recognition phase, non-modifier consumed events are buffered.  On failure
the buffer is replayed so those keystrokes reach the application.  Without this,
typing `gx` while not in Nav mode would silently swallow `g` if the Nav pattern
starts with `g`.

---

## 10. File map

```
src/pattern/
  ast.rs          Node, Edge, ActionSpec, NodeKind
  parser.rs       Recursive-descent parser → AST
  nfa.rs          Builder, Nfa, Machine, compile_to_nfa, compile_fused_nfa
  mod.rs          build_fused_nfa (entry point for config)

src/
  event_handler.rs   process_patterns(), frame stack, broadcast_state()
  state_broadcaster.rs  StateEvent, StateBroadcaster
  signal.rs          SignalDispatcher, repeat timers
  socket_worker.rs   Hyprland IPC socket worker (unrelated to state socket)
  config/mod.rs      Config struct, load_configs, pattern compilation

external/
  config.yml      Live test config (gitignored in main repo, tracked in
                  its own git repo at external/)
```
