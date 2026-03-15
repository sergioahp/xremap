use evdev::EventType;
use evdev::InputEvent;
use evdev::KeyCode as Key;
use indoc::indoc;
use nix::sys::timerfd::{ClockId, TimerFd, TimerFlags};
use std::path::Path;
use std::time::Duration;
use std::{fs, path::PathBuf};

use crate::client::{Client, WMClient};
use crate::device::InputDeviceInfo;
use crate::{
    action::Action,
    config::{keymap::build_keymap_table, Config},
    event::{Event, KeyEvent, KeyValue, RelativeEvent},
    event_handler::{make_signal_dispatcher, EventHandler},
};

struct StaticClient {
    current_application: Option<String>,
}

impl Client for StaticClient {
    fn supported(&mut self) -> bool {
        true
    }
    fn current_window(&mut self) -> Option<String> {
        None
    }

    fn current_application(&mut self) -> Option<String> {
        self.current_application.clone()
    }
}

pub fn get_input_device_info<'a>() -> InputDeviceInfo<'a> {
    InputDeviceInfo {
        name: "Some Device",
        path: Path::new("/dev/input/event0"),
        vendor: 0x1234,
        product: 0x5678,
    }
}

#[test]
fn test_basic_modmap() {
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              a: b
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_B, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
        ],
    )
}

/* Table to see which scancodes/custom key events correspond to which relative events
    Original RELATIVE event | scancode | Custom keyname if                              | Info
                            |          | positive value (+)     | negative value (-)    |
    REL_X                   |    0     | XRIGHTCURSOR       | XLEFTCURSOR       | Cursor right and left
    REL_Y                   |    1     | XDOWNCURSOR        | XUPCURSOR         | Cursor down and up
    REL_Z                   |    2     | XREL_Z_AXIS_1      | XREL_Z_AXIS_2     | Cursor... forward and backwards?
    REL_RX                  |    3     | XREL_RX_AXIS_1     | XREL_RX_AXIS_2    | Horizontally rotative cursor movement?
    REL_RY                  |    4     | XREL_RY_AXIS_1     | XREL_RY_AXIS_2    | Vertical rotative cursor movement?
    REL_RZ                  |    5     | XREL_RZ_AXIS_1     | XREL_RZ_AXIS_2    | "Whatever the third dimensional axis is called" rotative cursor movement?
    REL_HWHEEL              |    6     | XRIGHTSCROLL       | XLEFTSCROLL       | Rightscroll and leftscroll
    REL_DIAL                |    7     | XREL_DIAL_1        | XREL_DIAL_2       | ???
    REL_WHEEL               |    8     | XUPSCROLL          | XDOWNSCROLL       | Upscroll and downscroll
    REL_MISC                |    9     | XREL_MISC_1        | XREL_MISC_2       | Something?
    REL_RESERVED            |    10    | XREL_RESERVED_1    | XREL_RESERVED_2   | Something?
    REL_WHEEL_HI_RES        |    11    | XHIRES_UPSCROLL    | XHIRES_DOWNSCROLL | High resolution downscroll and upscroll, sent just after their non-high resolution version
    REL_HWHEEL_HI_RES       |    12    | XHIRES_RIGHTSCROLL | XHIRES_LEFTSCROLL | High resolution rightcroll and leftscroll, sent just after their non-high resolution version
*/

const _POSITIVE: i32 = 1;
const _NEGATIVE: i32 = -1;

const _REL_X: u16 = 0;
const _REL_Y: u16 = 1;
const _REL_Z: u16 = 2;
const _REL_RX: u16 = 3;
const _REL_RY: u16 = 4;
const _REL_RZ: u16 = 5;
const _REL_HWHEEL: u16 = 6;
const _REL_DIAL: u16 = 7;
const _REL_WHEEL: u16 = 8;
const _REL_MISC: u16 = 9;
const _REL_RESERVED: u16 = 10;
const _REL_WHEEL_HI_RES: u16 = 11;
const _REL_HWHEEL_HI_RES: u16 = 12;

#[test]
fn test_relative_events() {
    assert_actions(
        indoc! {"
        modmap:
          - remap:
              XRIGHTCURSOR: b
        "},
        vec![Event::RelativeEvent(
            get_input_device_info(),
            RelativeEvent::new_with(_REL_X, _POSITIVE),
        )],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
        ],
    )
}

#[test]
fn verify_disguised_relative_events() {
    use crate::config::DISGUISED_EVENT_OFFSETTER;
    // Verifies that the event offsetter used to "disguise" relative events into key event
    // is a bigger number than the biggest one a scancode had at the time of writing this (26 december 2022)
    const _: () = assert!(0x2e7 < DISGUISED_EVENT_OFFSETTER);
    // and that it's not big enough that one of the "disguised" events's scancode would overflow.
    // (the largest of those events is equal to DISGUISED_EVENT_OFFSETTER + 26)
    const _: () = assert!(DISGUISED_EVENT_OFFSETTER <= u16::MAX - 26);
}

#[test]
fn test_mouse_movement_event_accumulation() {
    // Tests that mouse movement events correctly get collected to be sent as one MouseMovementEventCollection,
    // which is necessary to avoid separating mouse movement events with synchronization events,
    // because such a separation would cause a bug with cursor movement.

    // Please refer to test_cursor_behavior_1 and test_cursor_behavior_2 for more information on said bug.
    assert_actions(
        indoc! {""},
        vec![
            Event::RelativeEvent(get_input_device_info(), RelativeEvent::new_with(_REL_X, _POSITIVE)),
            Event::RelativeEvent(get_input_device_info(), RelativeEvent::new_with(_REL_Y, _POSITIVE)),
        ],
        vec![Action::MouseMovementEventCollection(vec![
            RelativeEvent::new_with(_REL_X, _POSITIVE),
            RelativeEvent::new_with(_REL_Y, _POSITIVE),
        ])],
    )
}

#[test]
#[ignore]
// The OS interprets a REL_X event¹ combined with a REL_Y event² differently if they are separated by synchronization event.
// This test and test_cursor_behavior_2 are meant to be run to demonstrate that fact.

// ¹Mouse movement along the X (horizontal) axis.
// ²Mouse movement along the Y (vertical) axis.

// The only difference between test_cursor_behavior_1 and test_cursor_behavior_2 is that
// test_cursor_behavior_1 adds a synchronization event between REL_X and REL_Y events that would not normally be there.
// In other words, test_cursor_behavior_2 represents what would occur without Xremap intervention.

// Here's how to proceed :
// 1 - Move your mouse cursor to the bottom left of your screen.
// 2 - either run this test with sudo privileges or while your environnment is properly set up (https:// github.com/k0kubun/xremap#running-xremap-without-sudo),
//     so that your keyboard and/or mouse may be captured.

// 3 - Press any button (don't move the mouse).
// 4 - Note where the cursor ended up.

// 5 - Repeat steps 1 through 4 for test_cursor_behavior_2.
// 6 - Notice that the mouse cursor often ends up in a different position than when running test_cursor_behavior_1.

//
// Notes :
// - Because emitting an event automatcially adds a synchronization event afterwards (see https:// github.com/emberian/evdev/blob/1d020f11b283b0648427a2844b6b980f1a268221/src/uinput.rs#L167),
//   Mouse movement events should be batched together when emitted,
//   to avoid separating them with a synchronization event.
//
// - Because a mouse will only ever send a maximum of one REL_X and one REL_Y (and maybe one REL_Z for 3D mice?) at once,
//   the only point where a synchronization event can be added where it shouldn't by Xremap is between those events,
//   meaning this bug is exclusive to diagonal mouse movement.
//
// - The call to std::thread::sleep for five milliseconds is meant to emulate
//   the interval between events from a mouse with a frequency of ~200 Hz.
//   A lower time interval between events (which would correspond to a mouse with a higher frequency)
//   would cause the difference between test_cursor_behavior_1 and test_cursor_behavior_2 to become less noticeable.
//   Conversely, a higher time interval would make the difference more noticeable.
//
fn test_cursor_behavior_1() {
    use crate::device::InputDevice;
    use crate::device::{get_input_devices, output_device};
    // Setup to be able to send events
    let mut input_devices = match get_input_devices(&[String::from("/dev/input/event25")], &[], true, false) {
        Ok(input_devices) => input_devices,
        Err(e) => panic!("Failed to prepare input devices: {e}"),
    };
    let mut output_device =
        match output_device(input_devices.values().next().map(InputDevice::bus_type), true, 0x1234, 0x5678) {
            Ok(output_device) => output_device,
            Err(e) => panic!("Failed to prepare an output device: {e}"),
        };
    for input_device in input_devices.values_mut() {
        let _unused = input_device.fetch_events().unwrap();
    }

    // Looping 400 times amplifies the difference between test_cursor_behavior_1 and test_cursor_behavior_2 to visible levels.
    for _ in 0..400 {
        output_device
            .emit(&[
                InputEvent::new_now(EventType::RELATIVE.0, _REL_X, _POSITIVE),
                //
                // This line is the only difference between test_cursor_behavior_1 and test_cursor_behavior_2.
                InputEvent::new(EventType::SYNCHRONIZATION.0, 0, 0),
                //
                InputEvent::new_now(EventType::RELATIVE.0, _REL_Y, _NEGATIVE),
            ])
            .unwrap();

        // Creating a time interval between mouse movement events to simulate a mouse with a frequency of ~200 Hz.
        // The smaller the time interval, the smaller the difference between test_cursor_behavior_1 and test_cursor_behavior_2.
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
#[ignore]
// The OS interprets a REL_X event combined with a REL_Y event differently if they are separated by synchronization event.
// This test and test_cursor_behavior_1 are meant to be run to demonstrate that fact.
// Please refer to the comment above test_cursor_behavior_1 for information on how to run these tests.
fn test_cursor_behavior_2() {
    use crate::device::InputDevice;
    use crate::device::{get_input_devices, output_device};
    // Setup to be able to send events
    let mut input_devices = match get_input_devices(&[String::from("/dev/input/event25")], &[], true, false) {
        Ok(input_devices) => input_devices,
        Err(e) => panic!("Failed to prepare input devices: {e}"),
    };
    let mut output_device =
        match output_device(input_devices.values().next().map(InputDevice::bus_type), true, 0x1234, 0x5678) {
            Ok(output_device) => output_device,
            Err(e) => panic!("Failed to prepare an output device: {e}"),
        };
    for input_device in input_devices.values_mut() {
        let _unused = input_device.fetch_events().unwrap();
    }

    // Looping 400 times amplifies the difference between test_cursor_behavior_1 and test_cursor_behavior_2 to visible levels.
    for _ in 0..400 {
        output_device
            .emit(&[
                InputEvent::new_now(EventType::RELATIVE.0, _REL_X, _POSITIVE),
                InputEvent::new_now(EventType::RELATIVE.0, _REL_Y, _NEGATIVE),
            ])
            .unwrap();

        // Creating a time interval between mouse movement events to simulate a mouse with a frequency of ~200 Hz.
        // The smaller the time interval, the smaller the difference between test_cursor_behavior_1 and test_cursor_behavior_2.
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn test_interleave_modifiers() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              M-f: C-right
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_exact_match_true() {
    assert_actions(
        indoc! {"
        keymap:
          - exact_match: true
            remap:
              M-f: C-right
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_F, KeyValue::Press)),
        ],
    )
}

#[test]
fn test_exact_match_false() {
    assert_actions(
        indoc! {"
        keymap:
          - exact_match: false
            remap:
              M-f: C-right
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_exact_match_default() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              M-f: C-right
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_RIGHT, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTALT, KeyValue::Press)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_exact_match_true_nested() {
    assert_actions(
        indoc! {"
        keymap:
          - exact_match: true
            remap:
              C-x:
                remap:
                  h: C-a
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_H, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_H, KeyValue::Press)),
        ],
    )
}

#[test]
fn test_exact_match_false_nested() {
    assert_actions(
        indoc! {"
        keymap:
          - exact_match: false
            remap:
              C-x:
                remap:
                  h: C-a
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_H, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTSHIFT, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_application_override() {
    let config = indoc! {"
        keymap:

          - name: firefox
            application:
              only: [firefox]
            remap:
              a: C-c

          - name: generic
            remap:
              a: C-b
    "};

    assert_actions(
        config,
        vec![Event::KeyEvent(
            get_input_device_info(),
            KeyEvent::new(Key::KEY_A, KeyValue::Press),
        )],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );

    assert_actions_with_current_application(
        config,
        Some(String::from("firefox")),
        vec![Event::KeyEvent(
            get_input_device_info(),
            KeyEvent::new(Key::KEY_A, KeyValue::Press),
        )],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_device_override() {
    let config = indoc! {"
        keymap:

          - name: event1
            device:
              only: [event1]
            remap:
              a: C-c

          - name: event0
            remap:
              a: C-b
    "};

    assert_actions(
        config,
        vec![Event::KeyEvent(
            InputDeviceInfo {
                name: "Some Device",
                path: Path::new("/dev/input/event0"),
                vendor: 0x1234,
                product: 0x5678,
            },
            KeyEvent::new(Key::KEY_A, KeyValue::Press),
        )],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );

    assert_actions(
        config,
        vec![Event::KeyEvent(
            InputDeviceInfo {
                name: "Other Device",
                path: Path::new("/dev/input/event1"),
                vendor: 0x1234,
                product: 0x5678,
            },
            KeyEvent::new(Key::KEY_A, KeyValue::Press),
        )],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );
}

#[test]
fn test_merge_remaps() {
    let config = indoc! {"
        keymap:
          - remap:
              C-x:
                remap:
                  h: C-a
          - remap:
              C-x:
                remap:
                  k: C-w
    "};

    assert_actions(
        config,
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_H, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );

    assert_actions(
        config,
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_K, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_W, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_W, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_merge_remaps_with_override() {
    let config = indoc! {"
        keymap:
          - remap:
              C-x:
                remap:
                  h: C-a
          - remap:
              C-x:
                remap:
                  h: C-b
                  c: C-q
    "};

    assert_actions(
        config,
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_H, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    );

    assert_actions(
        config,
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_C, KeyValue::Press)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_X, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Q, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_Q, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_LEFTCTRL, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_mixing_keypress_and_remap_in_keymap_action() {
    // KEY_D will be emitted, and the remap will be used for next key press.
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12:
                - d
                - remap:
                    a: b
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F12, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F12, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_D, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_F12, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_mixing_no_keypress_and_remap_in_keymap_action() {
    // The first match stops the search for matches. So the last remap isn't used.
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12: []
          - remap:
              f12:
                - remap:
                    a: b
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F12, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F12, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_F12, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
        ],
    )
}

#[test]
fn test_no_keymap_action() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12: []
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F12, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F12, KeyValue::Release)),
        ],
        vec![
            //This is just release, so the key is not emitted.
            Action::KeyEvent(KeyEvent::new(Key::KEY_F12, KeyValue::Release)),
        ],
    );

    //Same test with the null keyword
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              f12: null
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F12, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_F12, KeyValue::Release)),
        ],
        vec![Action::KeyEvent(KeyEvent::new(Key::KEY_F12, KeyValue::Release))],
    )
}

#[test]
fn test_any_key() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              a: b
              ANY: null
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_C, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_C, KeyValue::Release)),
        ],
        vec![
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Press)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_B, KeyValue::Release)),
            Action::Delay(Duration::from_nanos(0)),
            Action::Delay(Duration::from_nanos(0)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_A, KeyValue::Release)),
            Action::KeyEvent(KeyEvent::new(Key::KEY_C, KeyValue::Release)),
        ],
    );
}

pub fn assert_actions(config_yaml: &str, events: Vec<Event>, actions: Vec<Action>) {
    assert_actions_with_current_application(config_yaml, None, events, actions);
}

pub fn assert_actions_with_current_application(
    config_yaml: &str,
    current_application: Option<String>,
    events: Vec<Event>,
    actions: Vec<Action>,
) {
    let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let mut config: Config = serde_yaml::from_str(config_yaml).unwrap();
    config.keymap_table = build_keymap_table(&config.keymap);
    let signal_timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let dispatcher = crate::signal::SignalDispatcher::new(std::collections::HashMap::new());
    let mut event_handler = EventHandler::new(
        timer,
        signal_timer,
        "default",
        Duration::from_micros(0),
        WMClient::new("static", Box::new(StaticClient { current_application })),
        dispatcher,
        None,
        None,
    );
    let mut actual: Vec<Action> = vec![];

    actual.append(&mut event_handler.on_events(&events, &config).unwrap());

    assert_eq!(format!("{actions:?}"), format!("{:?}", actual));
}

#[test]
fn test_pattern_emit_signal_actions() {
    let yaml = indoc! {"
    signals:
      repeat.left:
        repeat: false
        actions:
          - { press: b }
    patterns:
      Nav: \"a => emit_start(repeat.left) a! => emit_stop(repeat.left)\"
    "};
    let path = write_temp_config(yaml);
    let mut config = crate::config::load_configs(&[path.clone()]).expect("config load");
    let signal_timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let dispatcher = make_signal_dispatcher(&config);
    let mut handler = EventHandler::new(
        timer,
        signal_timer,
        "default",
        Duration::from_micros(0),
        WMClient::new("static", Box::new(StaticClient { current_application: None })),
        dispatcher,
        config.fused_nfa.clone(),
        None,
    );

    let actions = handler
        .on_events(
            &vec![Event::KeyEvent(
                get_input_device_info(),
                KeyEvent::new(Key::KEY_A, KeyValue::Press),
            )],
            &config,
        )
        .unwrap();
    assert!(actions
        .iter()
        .any(|a| matches!(a, Action::KeyEvent(k) if k.key == Key::KEY_B && k.value() == 1)));

    let actions2 = handler
        .on_events(
            &vec![Event::KeyEvent(
                get_input_device_info(),
                KeyEvent::new(Key::KEY_A, KeyValue::Release),
            )],
            &config,
        )
        .unwrap();
    assert!(actions2.is_empty());

    let _ = fs::remove_file(path);
}

#[test]
fn parse_move_pattern_from_config_sample() {
    let pat = r#"Super_L d => emit(float.enable) ( d!|ε ) ( h => emit_start(move.left) | h! => emit_stop(move.left) ε => emit(move.stop) | j => emit_start(move.down) | j! => emit_stop(move.down) ε => emit(move.stop) | k => emit_start(move.up) | k! => emit_stop(move.up) ε => emit(move.stop) | l => emit_start(move.right) | l! => emit_stop(move.right) ε => emit(move.stop) | d! => noop )* end_on(Super_L!) => emit(move.stop)"#;
    let _ = crate::pattern::parser::parse_pattern(pat).expect("pattern should parse");
}

// ── Workspace-toggle pattern tests ──────────────────────────────────────────

const WS_PREV_YAML: &str = indoc! {"
signals:
  ws.prev:
    repeat: false
    actions:
      - { press: w }
patterns:
  WorkspacePrevHold: \"Super_L leftbrace => emit(ws.prev) ( leftbrace! => emit(ws.prev) leftbrace => emit(ws.prev) )* end_on(Super_L!) => noop\"
"};

fn make_ws_handler() -> (EventHandler, crate::config::Config) {
    let path = write_temp_config(WS_PREV_YAML);
    let config = crate::config::load_configs(&[path]).expect("config load");
    let dispatcher = make_signal_dispatcher(&config);
    let signal_timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let handler = EventHandler::new(
        timer,
        signal_timer,
        "default",
        Duration::from_micros(0),
        WMClient::new("static", Box::new(StaticClient { current_application: None })),
        dispatcher,
        config.fused_nfa.clone(),
        None,
    );
    (handler, config)
}

fn kp(key: Key) -> Event<'static> {
    Event::KeyEvent(get_input_device_info(), KeyEvent::new(key, KeyValue::Press))
}

fn kr(key: Key) -> Event<'static> {
    Event::KeyEvent(get_input_device_info(), KeyEvent::new(key, KeyValue::Release))
}

fn emits_w(actions: &[Action]) -> bool {
    actions.iter().any(|a| matches!(a, Action::KeyEvent(k) if k.key == Key::KEY_W && k.value() == 1))
}

/// Super+[ emits ws.prev on press.
#[test]
fn test_ws_prev_basic() {
    let (mut h, cfg) = make_ws_handler();
    h.on_events(&vec![kp(Key::KEY_LEFTMETA)], &cfg).unwrap();
    let actions = h.on_events(&vec![kp(Key::KEY_LEFTBRACE)], &cfg).unwrap();
    assert!(emits_w(&actions), "Super+[ should emit ws.prev");
}

/// Super+J (unrelated key) kills the pattern; Super+[ while still holding Super
/// should still emit ws.prev via sticky restart.
#[test]
fn test_ws_prev_after_other_super_chord() {
    let (mut h, cfg) = make_ws_handler();
    // Press Super — pattern arms
    h.on_events(&vec![kp(Key::KEY_LEFTMETA)], &cfg).unwrap();
    // Press+release J — kills WorkspacePrevHold, but Super still held
    h.on_events(&vec![kp(Key::KEY_J)], &cfg).unwrap();
    h.on_events(&vec![kr(Key::KEY_J)], &cfg).unwrap();
    // Press [ — sticky restart should have re-armed; expect ws.prev
    let actions = h.on_events(&vec![kp(Key::KEY_LEFTBRACE)], &cfg).unwrap();
    assert!(emits_w(&actions), "Super+[ after Super+J should emit ws.prev via sticky restart");
}

// ── any-wildcard / modifier-mode tests ──────────────────────────────────────

const GUI_HOLD_YAML: &str = indoc! {"
signals:
  gui.open:
    repeat: false
    actions:
      - { press: o }
  gui.close:
    repeat: false
    actions:
      - { press: c }
patterns:
  GuiHold: \"Super_L s => noop ( o => emit(gui.open) | o! => emit(gui.close) | any => noop )* end_on(Super_L! | s!) => emit(gui.close)\"
"};

fn make_gui_handler() -> (EventHandler, crate::config::Config) {
    let path = write_temp_config(GUI_HOLD_YAML);
    let config = crate::config::load_configs(&[path]).expect("config load");
    let dispatcher = make_signal_dispatcher(&config);
    let signal_timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let handler = EventHandler::new(
        timer,
        signal_timer,
        "default",
        Duration::from_micros(0),
        WMClient::new("static", Box::new(StaticClient { current_application: None })),
        dispatcher,
        config.fused_nfa.clone(),
        None,
    );
    (handler, config)
}

fn emits_key(actions: &[Action], key: Key) -> bool {
    actions.iter().any(|a| matches!(a, Action::KeyEvent(k) if k.key == key && k.value() == 1))
}

/// After a GuiHold session ends, a fresh Super+S+O must still work.
/// Regression: machine left in stale post-Super state caused S press to miss the
/// pattern and fall through to keymap; O then fired the wrong action (volume up).
#[test]
fn test_gui_hold_second_session() {
    let (mut h, cfg) = make_gui_handler();
    // First session: enter and exit GuiHold
    h.on_events(&vec![kp(Key::KEY_LEFTMETA)], &cfg).unwrap();
    h.on_events(&vec![kp(Key::KEY_S)], &cfg).unwrap();
    h.on_events(&vec![kr(Key::KEY_S)], &cfg).unwrap(); // exit via s!
    h.on_events(&vec![kr(Key::KEY_LEFTMETA)], &cfg).unwrap();

    // Second session: fresh Super+S+O — O must open GUI, not fire keymap
    h.on_events(&vec![kp(Key::KEY_LEFTMETA)], &cfg).unwrap();
    h.on_events(&vec![kp(Key::KEY_S)], &cfg).unwrap();
    let open = h.on_events(&vec![kp(Key::KEY_O)], &cfg).unwrap();
    assert!(emits_key(&open, Key::KEY_O), "second session O press must open GUI, not fire keymap");
}

/// Releasing S while GUI is open (O held) must emit gui.close via end_on action.
#[test]
fn test_gui_hold_close_on_s_release() {
    let (mut h, cfg) = make_gui_handler();
    h.on_events(&vec![kp(Key::KEY_LEFTMETA)], &cfg).unwrap();
    h.on_events(&vec![kp(Key::KEY_S)], &cfg).unwrap();
    h.on_events(&vec![kp(Key::KEY_O)], &cfg).unwrap(); // open

    // Release S while Super+O still held — end_on(s!) must close the GUI
    let close = h.on_events(&vec![kr(Key::KEY_S)], &cfg).unwrap();
    assert!(emits_key(&close, Key::KEY_C), "releasing S must close GUI via end_on emit(gui.close)");
}

/// Super+S+O opens the GUI; releasing O closes it; pressing O again reopens.
#[test]
fn test_gui_hold_open_close_repeat() {
    let (mut h, cfg) = make_gui_handler();
    h.on_events(&vec![kp(Key::KEY_LEFTMETA)], &cfg).unwrap();
    h.on_events(&vec![kp(Key::KEY_S)], &cfg).unwrap();

    let open = h.on_events(&vec![kp(Key::KEY_O)], &cfg).unwrap();
    assert!(emits_key(&open, Key::KEY_O), "first O press should open GUI");

    let close = h.on_events(&vec![kr(Key::KEY_O)], &cfg).unwrap();
    assert!(emits_key(&close, Key::KEY_C), "O release should close GUI");

    let reopen = h.on_events(&vec![kp(Key::KEY_O)], &cfg).unwrap();
    assert!(emits_key(&reopen, Key::KEY_O), "second O press should reopen GUI");
}

/// After releasing S (or Super) from GuiHold, normal keys must not be swallowed.
#[test]
fn test_gui_hold_keys_forwarded_after_exit() {
    let (mut h, cfg) = make_gui_handler();
    // Enter GuiHold: Super+S+O (open), then release S to exit the mode
    h.on_events(&vec![kp(Key::KEY_LEFTMETA)], &cfg).unwrap();
    h.on_events(&vec![kp(Key::KEY_S)], &cfg).unwrap();
    h.on_events(&vec![kp(Key::KEY_O)], &cfg).unwrap();
    h.on_events(&vec![kr(Key::KEY_S)], &cfg).unwrap(); // exit via s!
    h.on_events(&vec![kr(Key::KEY_O)], &cfg).unwrap();
    h.on_events(&vec![kr(Key::KEY_LEFTMETA)], &cfg).unwrap();

    // Now `a` must be forwarded — machine must not still be in the swallowing loop
    let actions = h.on_events(&vec![kp(Key::KEY_A)], &cfg).unwrap();
    let a_forwarded = actions.iter().any(|a| matches!(a, Action::KeyEvent(k) if k.key == Key::KEY_A));
    assert!(a_forwarded, "KEY_A must be forwarded after GuiHold exits via s!");
}

/// While Super+S held (no O), pressing J should NOT fall through to keymap.
#[test]
fn test_gui_hold_blocks_other_keys() {
    let (mut h, cfg) = make_gui_handler();
    h.on_events(&vec![kp(Key::KEY_LEFTMETA)], &cfg).unwrap();
    h.on_events(&vec![kp(Key::KEY_S)], &cfg).unwrap();

    // J press while in modifier mode — should be consumed, not forwarded
    let actions = h.on_events(&vec![kp(Key::KEY_J)], &cfg).unwrap();
    let j_forwarded = actions.iter().any(|a| matches!(a, Action::KeyEvent(k) if k.key == Key::KEY_J));
    assert!(!j_forwarded, "J should be blocked while Super+S modifier mode is active");
}

// ── Speculative buffer + checkpoint frame tests ──────────────────────────────

/// Pattern fails in recognition phase (before first action fires): non-modifier
/// keys buffered during recognition must be replayed to the virtual device.
#[test]
fn test_chord_rollback_replays_buffered_keys() {
    // "Super_L d c => noop": recognition phase consumes Super_L (modifier) and d
    // (non-modifier, buffered). When Z is pressed instead of C, the pattern fails
    // with no commit, so D press must be replayed and Z must fall through.
    let yaml = indoc! {"
    patterns:
      Chord: \"Super_L d c => noop\"
    "};
    let path = write_temp_config(yaml);
    let config = crate::config::load_configs(&[path.clone()]).expect("config load");
    let dispatcher = make_signal_dispatcher(&config);
    let signal_timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let mut h = EventHandler::new(
        timer,
        signal_timer,
        "default",
        Duration::from_micros(0),
        WMClient::new("static", Box::new(StaticClient { current_application: None })),
        dispatcher,
        config.fused_nfa.clone(),
        None,
    );

    // Super_L press: consumed (modifier), forwarded via modifier path
    h.on_events(&vec![kp(Key::KEY_LEFTMETA)], &config).unwrap();
    // D press: consumed into recognition phase, buffered
    h.on_events(&vec![kp(Key::KEY_D)], &config).unwrap();
    // Z press: pattern fails — D buffered press must be replayed, Z falls through
    let actions = h.on_events(&vec![kp(Key::KEY_Z)], &config).unwrap();

    let d_replayed = actions.iter().any(|a| matches!(a, Action::KeyEvent(k) if k.key == Key::KEY_D && k.value() == 1));
    let z_forwarded = actions.iter().any(|a| matches!(a, Action::KeyEvent(k) if k.key == Key::KEY_Z && k.value() == 1));
    assert!(d_replayed, "D press must be replayed from speculative buffer on pattern failure; actions={actions:?}");
    assert!(z_forwarded, "Z press must fall through after pattern failure; actions={actions:?}");

    let _ = fs::remove_file(path);
}

/// After the commit point, failures with anchor keys still held should perform
/// a partial reset (restore checkpoint states) rather than a full reset.
/// The failing key falls through; subsequent loop keys are still consumed.
#[test]
fn test_partial_reset_restores_checkpoint() {
    // Pattern: Super_L s => noop ( o => noop )* end_on(Super_L!) => noop
    // Commit fires on S press (noop action). Checkpoint: anchor={Super_L, S}.
    // K press: not in loop, fails → partial reset (anchors still held) → K falls through.
    // O press: checkpoint restored, o consumed in loop (not forwarded).
    let yaml = indoc! {"
    patterns:
      Modal: \"Super_L s => noop ( o => noop )* end_on(Super_L!) => noop\"
    "};
    let path = write_temp_config(yaml);
    let config = crate::config::load_configs(&[path.clone()]).expect("config load");
    let dispatcher = make_signal_dispatcher(&config);
    let signal_timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let mut h = EventHandler::new(
        timer,
        signal_timer,
        "default",
        Duration::from_micros(0),
        WMClient::new("static", Box::new(StaticClient { current_application: None })),
        dispatcher,
        config.fused_nfa.clone(),
        None,
    );

    // Enter modal: Super_L then S (commit fires)
    h.on_events(&vec![kp(Key::KEY_LEFTMETA)], &config).unwrap();
    h.on_events(&vec![kp(Key::KEY_S)], &config).unwrap();

    // K press: not in loop — partial reset, K falls through
    let k_actions = h.on_events(&vec![kp(Key::KEY_K)], &config).unwrap();
    let k_forwarded = k_actions.iter().any(|a| matches!(a, Action::KeyEvent(k) if k.key == Key::KEY_K && k.value() == 1));
    assert!(k_forwarded, "K must fall through after partial reset; actions={k_actions:?}");

    // O press: checkpoint restored, should be consumed by loop (not forwarded)
    let o_actions = h.on_events(&vec![kp(Key::KEY_O)], &config).unwrap();
    let o_forwarded = o_actions.iter().any(|a| matches!(a, Action::KeyEvent(k) if k.key == Key::KEY_O && k.value() == 1));
    assert!(!o_forwarded, "O must be consumed by the loop after partial reset; actions={o_actions:?}");

    let _ = fs::remove_file(path);
}

/// After the commit point, if an anchor key is released, the next failure must
/// trigger a full reset (anchor check fails) rather than a partial reset.
#[test]
fn test_full_reset_when_anchor_released() {
    // Same pattern as above. Anchor at commit: {Super_L, S}.
    // Release Super_L (anchor gone). Then O press fails (machine was reset, Super_L
    // not held so re-arm doesn't reach loop state) — full reset path taken.
    // After full reset, O must fall through (not consumed by loop).
    let yaml = indoc! {"
    patterns:
      Modal: \"Super_L s => noop ( o => noop )* end_on(Super_L!) => noop\"
    "};
    let path = write_temp_config(yaml);
    let config = crate::config::load_configs(&[path.clone()]).expect("config load");
    let dispatcher = make_signal_dispatcher(&config);
    let signal_timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let mut h = EventHandler::new(
        timer,
        signal_timer,
        "default",
        Duration::from_micros(0),
        WMClient::new("static", Box::new(StaticClient { current_application: None })),
        dispatcher,
        config.fused_nfa.clone(),
        None,
    );

    // Enter modal: Super_L then S (commit fires, anchor={Super_L, S})
    h.on_events(&vec![kp(Key::KEY_LEFTMETA)], &config).unwrap();
    h.on_events(&vec![kp(Key::KEY_S)], &config).unwrap();

    // Release Super_L: anchor key released; note this is an end_on key so the
    // pattern ends via end_on, causing a full reset regardless.
    h.on_events(&vec![kr(Key::KEY_LEFTMETA)], &config).unwrap();

    // O press: pattern has fully reset (no Super_L held to re-arm), must fall through
    let o_actions = h.on_events(&vec![kp(Key::KEY_O)], &config).unwrap();
    let o_forwarded = o_actions.iter().any(|a| matches!(a, Action::KeyEvent(k) if k.key == Key::KEY_O && k.value() == 1));
    assert!(o_forwarded, "O must fall through after full reset when anchor released; actions={o_actions:?}");

    let _ = fs::remove_file(path);
}

/// Verifies that push_frame extends the active frame after the d! anchor key is released.
/// While D is held, j fires swap_next. D release fires push_frame (new frame, anchor={Super}).
/// After D is released, c fires wm.kill (frame 2 still active). Super release ends everything.
#[test]
fn test_push_frame_extends_frame_after_anchor_release() {
    let yaml = indoc! {"
    patterns:
      WinMgmt: \"Super_L d => noop ( j => emit(test.j) | c => emit(test.c) | d! => push_frame | any => noop )* end_on(Super_L!) => noop\"
    signals:
      test.j:
        actions: []
      test.c:
        actions: []
    "};
    let path = write_temp_config(yaml);
    let config = crate::config::load_configs(&[path.clone()]).expect("config load");
    let dispatcher = make_signal_dispatcher(&config);
    let signal_timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
    let mut h = EventHandler::new(
        timer,
        signal_timer,
        "default",
        Duration::from_micros(0),
        WMClient::new("static", Box::new(StaticClient { current_application: None })),
        dispatcher,
        config.fused_nfa.clone(),
        None,
    );

    // Super_L press: enters recognition phase (modifier, consumed)
    h.on_events(&vec![kp(Key::KEY_LEFTMETA)], &config).unwrap();
    // D press: commit fires (noop action), frame pushed with anchor={Super, D}
    h.on_events(&vec![kp(Key::KEY_D)], &config).unwrap();

    // J press while D held: test.j signal should fire, J not forwarded
    let j_actions = h.on_events(&vec![kp(Key::KEY_J)], &config).unwrap();
    let j_forwarded = j_actions.iter().any(|a| matches!(a, Action::KeyEvent(k) if k.key == Key::KEY_J && k.value() == 1));
    assert!(!j_forwarded, "J must be consumed by the loop while D is held; actions={j_actions:?}");

    // D release: push_frame fires — new frame pushed with anchor={Super_L} only
    h.on_events(&vec![kr(Key::KEY_D)], &config).unwrap();

    // C press: test.c signal should fire, C not forwarded (frame 2 still active, Super still held)
    let c_actions = h.on_events(&vec![kp(Key::KEY_C)], &config).unwrap();
    let c_forwarded = c_actions.iter().any(|a| matches!(a, Action::KeyEvent(k) if k.key == Key::KEY_C && k.value() == 1));
    assert!(!c_forwarded, "C must be consumed by the loop after D release (push_frame frame active); actions={c_actions:?}");

    // Super release: end_on fires, full reset
    h.on_events(&vec![kr(Key::KEY_LEFTMETA)], &config).unwrap();

    // After full reset, a new key should fall through
    let x_actions = h.on_events(&vec![kp(Key::KEY_X)], &config).unwrap();
    let x_forwarded = x_actions.iter().any(|a| matches!(a, Action::KeyEvent(k) if k.key == Key::KEY_X && k.value() == 1));
    assert!(x_forwarded, "X must fall through after full reset; actions={x_actions:?}");

    let _ = fs::remove_file(path);
}

fn write_temp_config(yaml: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "xremap-pattern-test-{}.yml",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::write(&path, yaml).expect("write temp config");
    path
}
