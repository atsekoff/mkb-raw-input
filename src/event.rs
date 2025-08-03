//! Event types for RawInput library (keyboard and mouse)

use crate::keyboard::RawKeyboardEvent;
use crate::mouse::RawMouseEvent;

/// Represents a raw input event received from the Windows Raw Input API.
///
/// This enum wraps either a keyboard or mouse event, parsed into ergonomic Rust types.
#[derive(Debug, Clone)]
pub enum RawInputEvent {
    /// A keyboard event (key press or release)
    Keyboard(RawKeyboardEvent),
    /// A mouse event (movement, button, or wheel)
    Mouse(RawMouseEvent),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyboard::{KeyEventMessage, VirtualKey};
    use crate::mouse::{MouseButtonAction, MouseMoveMode};
    use windows::Win32::UI::Input::{RAWKEYBOARD, RAWMOUSE};

    #[test]
    fn test_keyboard_event_from_raw() {
        let raw = RAWKEYBOARD {
            MakeCode: 42u16,
            Flags: 2u16, // Key down, extended (E0)
            Reserved: 0u16,
            VKey: 65u16,     // 'A' key
            Message: 256u32, // WM_KEYDOWN
            ExtraInformation: 123u32,
        };

        let event = RawKeyboardEvent::from(&raw);
        assert_eq!(event.make_code, 42);
        assert_eq!(event.key_up, false);
        assert_eq!(event.extended, true);
        assert_eq!(event.message, KeyEventMessage::KeyDown);
        assert_eq!(event.vkey, VirtualKey::A);
        assert_eq!(event.extra_information, 123);
    }

    #[test]
    fn test_mouse_event_from_raw() {
        let raw = RAWMOUSE {
            usFlags: windows::Win32::UI::Input::MOUSE_STATE(0x01u16), // MOUSE_MOVE_ABSOLUTE
            ulRawButtons: 0x8000u32,
            lLastX: 10i32,
            lLastY: -20i32,
            ulExtraInformation: 0xDEADBEEFu32,
            Anonymous: unsafe {
                let mut anon = std::mem::zeroed::<windows::Win32::UI::Input::RAWMOUSE_0>();
                anon.Anonymous.usButtonFlags = 0x0001; // Left button down
                anon.Anonymous.usButtonData = 120;
                anon
            },
        };

        let event = RawMouseEvent::from(&raw);
        assert_eq!(event.move_mode, MouseMoveMode::Absolute);
        assert_eq!(event.button_action, MouseButtonAction::LeftDown);
        assert_eq!(event.raw_buttons, 0x8000);
        assert_eq!(event.last_x, 10);
        assert_eq!(event.last_y, -20);
        assert_eq!(event.extra_information, 0xDEADBEEF);
    }
}
