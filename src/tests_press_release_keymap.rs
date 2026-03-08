use crate::action::Action;
use crate::event::Event;
use crate::event::{KeyEvent, KeyValue};
use crate::tests::{assert_actions, get_input_device_info};
use evdev::KeyCode as Key;
use indoc::indoc;

#[test]
fn test_press_release_keymap_triggers_release_on_modifier_up() {
    assert_actions(
        indoc! {"
        keymap:
          - remap:
              SUPER-G:
                on_press:
                  - { set_mode: resize }
                  - { launch: [\"press\"] }
                on_release:
                  - { launch: [\"release\"] }
                  - { set_mode: default }
          - mode: [resize]
            remap:
              K: { launch: [\"mode\"] }
        "},
        vec![
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTMETA, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_G, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_K, KeyValue::Press)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_K, KeyValue::Release)),
            Event::KeyEvent(get_input_device_info(), KeyEvent::new(Key::KEY_LEFTMETA, KeyValue::Release)),
        ],
        vec![
            Action::Command(vec!["press".into()]),
            Action::Command(vec!["mode".into()]),
            Action::Command(vec!["release".into()]),
        ],
    )
}
