#[cfg(any(feature = "hypr", feature = "wlroots"))]
use wayland_client::{protocol::wl_registry, Connection, Dispatch, Proxy, QueueHandle};
#[cfg(any(feature = "hypr", feature = "wlroots"))]
use wayland_protocols_hyprland::lock_notify::v1::client::{
    hyprland_lock_notification_v1, hyprland_lock_notifier_v1,
};

#[cfg(any(feature = "hypr", feature = "wlroots"))]
use std::sync::{Arc, Mutex};
#[cfg(any(feature = "hypr", feature = "wlroots"))]
use std::thread;

#[cfg(any(feature = "hypr", feature = "wlroots"))]
pub struct HyprlandLockWatcher {
    pub is_locked: Arc<Mutex<bool>>,
}

#[cfg(any(feature = "hypr", feature = "wlroots"))]
struct LockState {
    notifier: Option<hyprland_lock_notifier_v1::HyprlandLockNotifierV1>,
    notification: Option<hyprland_lock_notification_v1::HyprlandLockNotificationV1>,
    is_locked: Arc<Mutex<bool>>,
}

#[cfg(any(feature = "hypr", feature = "wlroots"))]
impl Dispatch<wl_registry::WlRegistry, ()> for LockState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _udata: &(),
        _conn: &Connection,
        qh: &QueueHandle<LockState>,
    ) {
        if let wl_registry::Event::Global { name, interface, version } = event {
            if interface == hyprland_lock_notifier_v1::HyprlandLockNotifierV1::interface().name {
                let notifier: hyprland_lock_notifier_v1::HyprlandLockNotifierV1 =
                    registry.bind(name, version.min(1), qh, ());
                let notif = notifier.get_lock_notification(qh, ());
                state.notifier = Some(notifier);
                state.notification = Some(notif);
            }
        }
    }
}

#[cfg(any(feature = "hypr", feature = "wlroots"))]
impl Dispatch<hyprland_lock_notification_v1::HyprlandLockNotificationV1, ()> for LockState {
    fn event(
        state: &mut Self,
        _obj: &hyprland_lock_notification_v1::HyprlandLockNotificationV1,
        event: hyprland_lock_notification_v1::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<LockState>,
    ) {
        match event {
            hyprland_lock_notification_v1::Event::Locked => {
                println!("Hyprland screen locked - disabling key mappings");
                if let Ok(mut locked) = state.is_locked.lock() {
                    *locked = true;
                }
            }
            hyprland_lock_notification_v1::Event::Unlocked => {
                println!("Hyprland screen unlocked - enabling key mappings");
                if let Ok(mut locked) = state.is_locked.lock() {
                    *locked = false;
                }
            }
            _ => {}
        }
    }
}

#[cfg(any(feature = "hypr", feature = "wlroots"))]
wayland_client::delegate_noop!(LockState: hyprland_lock_notifier_v1::HyprlandLockNotifierV1);

#[cfg(any(feature = "hypr", feature = "wlroots"))]
impl HyprlandLockWatcher {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let is_locked = Arc::new(Mutex::new(false));
        let is_locked_clone = Arc::clone(&is_locked);

        // Spawn background thread to monitor lock state
        thread::spawn(move || {
            if let Err(e) = Self::run_watcher(is_locked_clone) {
                eprintln!("Hyprland lock watcher error: {}", e);
            }
        });

        Ok(HyprlandLockWatcher { is_locked })
    }

    fn run_watcher(is_locked: Arc<Mutex<bool>>) -> Result<(), Box<dyn std::error::Error>> {
        let conn = Connection::connect_to_env()?;
        let display = conn.display();
        let mut event_queue = conn.new_event_queue();
        let qh = event_queue.handle();

        let _registry = display.get_registry(&qh, ());
        let mut state = LockState {
            notifier: None,
            notification: None,
            is_locked,
        };

        // Process initial globals
        event_queue.roundtrip(&mut state)?;

        if state.notification.is_none() {
            eprintln!("Warning: hyprland_lock_notifier_v1 not available on this compositor");
            return Ok(());
        }

        // Process events forever
        loop {
            event_queue.blocking_dispatch(&mut state)?;
        }
    }

    pub fn is_locked(&self) -> bool {
        match self.is_locked.lock() {
            Ok(guard) => *guard,
            Err(_) => {
                // If mutex is poisoned, assume unlocked
                eprintln!("Warning: Hyprland lock state mutex poisoned, assuming unlocked");
                false
            }
        }
    }
}

#[cfg(not(any(feature = "hypr", feature = "wlroots")))]
pub struct HyprlandLockWatcher;

#[cfg(not(any(feature = "hypr", feature = "wlroots")))]
impl HyprlandLockWatcher {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(HyprlandLockWatcher)
    }

    pub fn is_locked(&self) -> bool {
        false
    }
}