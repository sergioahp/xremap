use std::collections::HashMap;
use std::fs;
use std::os::unix::net::UnixDatagram;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct SocketMessage {
    action: String,
    id: String,
    command: Vec<String>,
}

struct Controller {
    command: Vec<String>,
    velocity: f32,
    next_tick: Instant,
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
                                map.insert(
                                    msg.id.clone(),
                                    Controller {
                                        command: msg.command,
                                        velocity: 0.0,
                                        next_tick: now,
                                        last_heartbeat: now,
                                    },
                                );
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
                let mut to_run: Vec<Vec<String>> = vec![];
                {
                    let mut map = controllers.lock().unwrap();
                    let now = Instant::now();
                    map.retain(|_, ctrl| now.duration_since(ctrl.last_heartbeat) < Duration::from_millis(200));
                    for ctrl in map.values_mut() {
                        if now >= ctrl.next_tick {
                            ctrl.velocity = (ctrl.velocity + 0.2).min(1.0);
                            let interval = Duration::from_millis((80.0 - 60.0 * ctrl.velocity) as u64);
                            ctrl.next_tick = now + interval;
                            to_run.push(ctrl.command.clone());
                        }
                    }
                }
                for cmd in to_run {
                    let _ = std::process::Command::new(&cmd[0]).args(&cmd[1..]).spawn();
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

