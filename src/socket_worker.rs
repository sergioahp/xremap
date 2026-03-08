use std::collections::HashMap;
use std::fs;
use std::os::unix::net::UnixDatagram;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;
use serde_json;

#[derive(Debug, Deserialize)]
struct SocketMessage {
    action: String,
    id: String,
    command: Vec<String>,
}

struct Controller {
    command: Vec<String>,
    rate: f32,          // commands per second (fixed target cadence)
    acc: f32,           // fractional accumulator for sends
    scale: f32,         // current step scale
    target_scale: f32,  // desired step scale
    last_tick: Instant, // time of last tick update
    last_heartbeat: Instant,
}

pub struct SocketWorker {
    pub path: String,
}

impl SocketWorker {
    pub fn start(path: String) -> anyhow::Result<Self> {
        if Path::new(&path).exists() {
            let _ = fs::remove_file(&path);
        }
        let sock = UnixDatagram::bind(&path)?;
        // Make socket world-writeable so external helpers can send without perms issues.
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o666));
        let controllers: Arc<Mutex<HashMap<String, Controller>>> = Arc::new(Mutex::new(HashMap::new()));

        // Receiver thread
        {
            let sock = sock.try_clone()?;
            let controllers = controllers.clone();
            thread::spawn(move || loop {
                let mut buf = [0u8; 1024];
                if let Ok((len, _)) = sock.recv_from(&mut buf) {
                    if let Ok(msg) = serde_json::from_slice::<SocketMessage>(&buf[..len]) {
                        let mut map = controllers.lock().unwrap();
                        match msg.action.as_str() {
                            "start" => {
                                let now = Instant::now();
                                if let Some(ctrl) = map.get_mut(&msg.id) {
                                    ctrl.last_heartbeat = now;
                                } else {
                                    // Fire once immediately for responsiveness
                                    send_scaled_cmd(&msg.command, 1.0);
                                    map.insert(
                                        msg.id.clone(),
                                        Controller {
                                            command: msg.command,
                                            rate: 60.0,        // fixed cadence
                                            acc: 0.0,
                                            scale: 1.0,
                                            target_scale: 3.0, // accelerate by increasing step size
                                            last_tick: now,
                                            last_heartbeat: now,
                                        },
                                    );
                                }
                            }
                            "stop" => {
                                map.remove(&msg.id);
                            }
                            "heartbeat" => {
                                if let Some(ctrl) = map.get_mut(&msg.id) {
                                    ctrl.last_heartbeat = Instant::now();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            });
        }

        // Ticker thread for executing commands with easing
        {
            let controllers = controllers.clone();
            thread::spawn(move || loop {
                thread::sleep(Duration::from_millis(10));
                let mut to_run: Vec<(Vec<String>, f32)> = vec![];
                {
                    let mut map = controllers.lock().unwrap();
                    let now = Instant::now();
                    map.retain(|_, ctrl| now.duration_since(ctrl.last_heartbeat) < Duration::from_millis(200));
                    for ctrl in map.values_mut() {
                        let dt = now.duration_since(ctrl.last_tick).as_secs_f32();
                        ctrl.last_tick = now;
                        // Exponential easing toward target scale; rate stays fixed (60 cps)
                        ctrl.scale += (ctrl.target_scale - ctrl.scale) * 0.18;
                        ctrl.acc += ctrl.rate * dt;
                        while ctrl.acc >= 1.0 {
                            ctrl.acc -= 1.0;
                            to_run.push((ctrl.command.clone(), ctrl.scale));
                        }
                    }
                }
                for (cmd, scale) in to_run {
                    send_scaled_cmd(&cmd, scale);
                }
            });
        }

        Ok(SocketWorker { path })
    }
}

pub fn send_to_socket(path: &str, msg: &str) {
    if let Ok(sock) = UnixDatagram::unbound() {
        let _ = sock.send_to(msg.as_bytes(), path);
    }
}

fn send_scaled_cmd(cmd: &Vec<String>, scale: f32) {
    if cmd.is_empty() {
        return;
    }
    // Scale any numeric arguments (useful for moveactive/resizeactive)
    let mut scaled: Vec<String> = Vec::with_capacity(cmd.len());
    scaled.push(cmd[0].clone());
    for arg in cmd.iter().skip(1) {
        if let Ok(n) = arg.parse::<f32>() {
            let val = (n * scale).round() as i32;
            // Avoid collapsing to zero; preserve sign
            let val = if val == 0 { if n >= 0.0 { 1 } else { -1 } } else { val };
            scaled.push(val.to_string());
        } else {
            scaled.push(arg.clone());
        }
    }
    let _ = std::process::Command::new(&scaled[0]).args(&scaled[1..]).spawn();
}
