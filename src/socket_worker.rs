use std::collections::HashMap;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct JsonMsg {
    action: String,
    id: String,
    #[serde(default)]
    command: Vec<String>,
}

pub enum WorkerMsg {
    Start { id: String, command: Vec<String> },
    Stop { id: String },
    Heartbeat { id: String },
}

struct Controller {
    command: Vec<String>,
    rate: f32,
    acc: f32,
    scale: f32,
    target_scale: f32,
    last_tick: Instant,
    last_heartbeat: Instant,
}

/// Handle returned to callers; send WorkerMsgs via the channel.
pub struct WorkerHandle {
    tx: mpsc::Sender<WorkerMsg>,
}

impl WorkerHandle {
    /// Parse a JSON socket_send payload (same format as the old Unix-socket messages)
    /// and enqueue it on the channel. Never blocks, never drops.
    pub fn send_json(&self, payload: &str) {
        if let Ok(msg) = serde_json::from_str::<JsonMsg>(payload) {
            let wm = match msg.action.as_str() {
                "start" => WorkerMsg::Start { id: msg.id, command: msg.command },
                "stop" => WorkerMsg::Stop { id: msg.id },
                "heartbeat" => WorkerMsg::Heartbeat { id: msg.id },
                _ => return,
            };
            let _ = self.tx.send(wm);
        }
    }
}

pub fn start_worker() -> WorkerHandle {
    let (tx, rx) = mpsc::channel::<WorkerMsg>();
    thread::spawn(move || worker_loop(rx));
    WorkerHandle { tx }
}

// ---------------------------------------------------------------------------
// Command execution: prefer direct Hyprland IPC, fall back to process spawn
// ---------------------------------------------------------------------------

enum Executor {
    HyprSocket(String),
    Spawn,
}

fn detect_executor() -> Executor {
    if let Some(path) = hyprland_socket_path() {
        if std::path::Path::new(&path).exists() {
            return Executor::HyprSocket(path);
        }
    }
    Executor::Spawn
}

fn hyprland_socket_path() -> Option<String> {
    let xdg = std::env::var("XDG_RUNTIME_DIR").ok()?;
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    Some(format!("{}/hypr/{}/.socket.sock", xdg, sig))
}

fn send_command(executor: &Executor, command: &[String], scale: f32) {
    match executor {
        Executor::HyprSocket(path) => {
            let cmd = build_hypr_cmd(command, scale);
            if !cmd.is_empty() {
                // Synchronous write: by the time this returns the bytes are in
                // Hyprland's kernel receive buffer. No lingering processes.
                if let Ok(mut stream) = UnixStream::connect(path) {
                    let _ = stream.write_all(cmd.as_bytes());
                    // Closing the stream signals end-of-message to Hyprland.
                }
            }
        }
        Executor::Spawn => {
            spawn_scaled_cmd(command, scale);
        }
    }
}

/// Build the Hyprland IPC command string from a hyprctl-style command Vec.
/// command[0] is the hyprctl binary path; command[1..] are the Hyprland args.
fn build_hypr_cmd(command: &[String], scale: f32) -> String {
    if command.len() < 2 {
        return String::new();
    }
    command[1..]
        .iter()
        .map(|arg| {
            if let Ok(n) = arg.parse::<f32>() {
                let val = (n * scale).round() as i32;
                // Preserve sign; never collapse to zero.
                let val = if val == 0 { if n >= 0.0 { 1 } else { -1 } } else { val };
                val.to_string()
            } else {
                arg.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn spawn_scaled_cmd(cmd: &[String], scale: f32) {
    if cmd.is_empty() {
        return;
    }
    let mut scaled: Vec<String> = Vec::with_capacity(cmd.len());
    scaled.push(cmd[0].clone());
    for arg in cmd.iter().skip(1) {
        if let Ok(n) = arg.parse::<f32>() {
            let val = (n * scale).round() as i32;
            let val = if val == 0 { if n >= 0.0 { 1 } else { -1 } } else { val };
            scaled.push(val.to_string());
        } else {
            scaled.push(arg.clone());
        }
    }
    let _ = std::process::Command::new(&scaled[0])
        .args(&scaled[1..])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

// ---------------------------------------------------------------------------
// Worker loop
// ---------------------------------------------------------------------------

fn worker_loop(rx: mpsc::Receiver<WorkerMsg>) {
    let executor = detect_executor();
    let mut controllers: HashMap<String, Controller> = HashMap::new();

    // How long we wait for a message before ticking.
    // Using recv_timeout instead of sleep means Stop is processed immediately
    // (no fixed sleep delay) while still ticking at ~100 Hz.
    const TICK_INTERVAL: Duration = Duration::from_millis(10);

    // Safety-net GC timeout. With a reliable channel Stop messages are never
    // dropped, so this only matters if the sender thread panics/exits.
    const HEARTBEAT_TIMEOUT: Duration = Duration::from_millis(200);

    loop {
        // Block until a message arrives OR the tick interval elapses.
        match rx.recv_timeout(TICK_INTERVAL) {
            Ok(msg) => {
                process_msg(msg, &mut controllers, &executor);
                // Drain any additional messages that arrived while we were busy.
                loop {
                    match rx.try_recv() {
                        Ok(msg) => process_msg(msg, &mut controllers, &executor),
                        Err(mpsc::TryRecvError::Empty) => break,
                        Err(mpsc::TryRecvError::Disconnected) => return,
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        }

        // GC controllers whose heartbeat has gone stale.
        let now = Instant::now();
        controllers.retain(|_, c| now.duration_since(c.last_heartbeat) < HEARTBEAT_TIMEOUT);

        // Send any commands due this tick.
        for ctrl in controllers.values_mut() {
            let dt = now.duration_since(ctrl.last_tick).as_secs_f32();
            ctrl.last_tick = now;
            ctrl.scale += (ctrl.target_scale - ctrl.scale) * 0.18;
            ctrl.acc += ctrl.rate * dt;
            while ctrl.acc >= 1.0 {
                ctrl.acc -= 1.0;
                send_command(&executor, &ctrl.command, ctrl.scale);
            }
        }
    }
}

fn process_msg(msg: WorkerMsg, controllers: &mut HashMap<String, Controller>, executor: &Executor) {
    let now = Instant::now();
    match msg {
        WorkerMsg::Start { id, command } => {
            if let Some(ctrl) = controllers.get_mut(&id) {
                ctrl.last_heartbeat = now;
            } else {
                // Fire one command immediately for responsiveness.
                send_command(executor, &command, 1.0);
                controllers.insert(
                    id,
                    Controller {
                        command,
                        rate: 60.0,
                        acc: 0.0,
                        scale: 1.0,
                        target_scale: 3.0,
                        last_tick: now,
                        last_heartbeat: now,
                    },
                );
            }
        }
        WorkerMsg::Stop { id } => {
            controllers.remove(&id);
        }
        WorkerMsg::Heartbeat { id } => {
            if let Some(ctrl) = controllers.get_mut(&id) {
                ctrl.last_heartbeat = now;
            }
        }
    }
}
