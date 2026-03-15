use std::{
    io::Write,
    os::unix::net::{UnixListener, UnixStream},
    sync::{Arc, Mutex},
    time::Duration,
};

use serde::Serialize;

/// A state event broadcast to all connected clients whenever xremap's
/// pattern-machine state changes.  Serialised as newline-delimited JSON.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StateEvent {
    /// No pattern is active; the machine is idle.
    Idle,
    /// A pattern is committed (frame stack non-empty).
    Active {
        /// Name of the matched pattern, if it could be identified.
        pattern: Option<String>,
        /// Frame depth: 1 = initial commit, 2 = sub-frame (e.g. after D release), …
        frame: usize,
        /// Keys held at the time the current top frame was pushed.
        /// These are the "anchor" keys; the frame stays valid while all are held.
        anchor_keys: Vec<String>,
        /// Keys actually held right now (may differ from anchor_keys).
        held_keys: Vec<String>,
        /// Edge labels the NFA currently accepts from its active states.
        /// Format: `"KEY_J"` for press, `"KEY_D!"` for release, `"any"` for wildcard.
        available: Vec<String>,
    },
}

/// Broadcasts [`StateEvent`]s as newline-delimited JSON to every connected
/// Unix-socket client.  Clients connect/disconnect freely; dead clients are
/// silently removed on the next broadcast.
pub struct StateBroadcaster {
    clients: Arc<Mutex<Vec<UnixStream>>>,
    /// Kept alive so the acceptor thread runs for the lifetime of this struct.
    _acceptor: std::thread::JoinHandle<()>,
}

impl StateBroadcaster {
    /// Bind a Unix-domain socket at `path`.  Removes any stale socket first.
    pub fn bind(path: &str) -> std::io::Result<Self> {
        let _ = std::fs::remove_file(path);
        let listener = UnixListener::bind(path)?;
        listener.set_nonblocking(true)?;

        let clients: Arc<Mutex<Vec<UnixStream>>> = Arc::new(Mutex::new(Vec::new()));
        let clients_tx = Arc::clone(&clients);

        let _acceptor = std::thread::Builder::new()
            .name("xremap-state-acceptor".into())
            .spawn(move || {
                loop {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            clients_tx.lock().unwrap().push(stream);
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            std::thread::sleep(Duration::from_millis(50));
                        }
                        Err(_) => break,
                    }
                }
            })?;

        Ok(StateBroadcaster { clients, _acceptor })
    }

    /// Serialize `event` to a JSON line and write it to every live client.
    pub fn broadcast(&self, event: &StateEvent) {
        let mut line = serde_json::to_string(event).expect("StateEvent serialization infallible");
        line.push('\n');
        let bytes = line.as_bytes();
        let mut clients = self.clients.lock().unwrap();
        clients.retain_mut(|c| c.write_all(bytes).is_ok());
    }
}
