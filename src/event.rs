//! Event types for RawInput library (keyboard and mouse)

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

use windows::Win32::UI::Input::{RAWKEYBOARD, RAWMOUSE};

/// Data for a raw keyboard event, parsed from the Windows RAWKEYBOARD struct.
///
/// All fields are direct representations of the Windows API fields, but with Rust naming.
///
/// - `make_code`: Scan code of the key (hardware-dependent, see [Scan Code](https://docs.microsoft.com/en-us/windows/win32/inputdev/about-keyboard-input#scan-codes)).
/// - `flags`: Bitfield indicating event type and extended key (see below).
///   - Bit 0: Key is an extended key (e.g., right ALT/CTRL, arrow keys)
///   - Bit 1: Reserved
///   - Bit 2: Key was released (1 = key up, 0 = key down)
///   - Bit 3: Key is a scan code prefixed with E0 (1 = E0, 0 = E1 or none)
///   - Bits 4-15: Reserved
/// - `vkey`: Virtual-key code (see [Virtual-Key Codes](https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes)).
/// - `message`: Corresponding Windows message (e.g., WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP).
/// - `extra_information`: Additional info from the driver or hardware.
#[derive(Debug, Clone)]
pub struct RawKeyboardEvent {
    /// Scan code of the key (hardware-dependent)
    pub make_code: u16,
    /// Bitfield indicating event type and extended key
    pub flags: u16,
    /// Virtual-key code (see MSDN)
    pub vkey: u16,
    /// Windows message type (WM_KEYDOWN, WM_KEYUP, etc.)
    pub message: u32,
    /// Additional driver/hardware info
    pub extra_information: u32,
}

impl From<&RAWKEYBOARD> for RawKeyboardEvent {
    fn from(kbd: &RAWKEYBOARD) -> Self {
        Self {
            make_code: kbd.MakeCode,
            flags: kbd.Flags,
            vkey: kbd.VKey,
            message: kbd.Message,
            extra_information: kbd.ExtraInformation,
        }
    }
}

/// Data for a raw mouse event, parsed from the Windows RAWMOUSE struct.
///
/// All fields are direct representations of the Windows API fields, but with Rust naming.
///
/// - `flags`: Bitfield indicating mouse event type (e.g., relative/absolute movement, virtual desktop)
///   - Bit 0: MOUSE_MOVE_RELATIVE (0) or MOUSE_MOVE_ABSOLUTE (1)
///   - Bit 1: MOUSE_VIRTUAL_DESKTOP (1 = virtual desktop)
///   - Bit 2: MOUSE_ATTRIBUTES_CHANGED (1 = device attributes changed)
///   - Bit 3: MOUSE_MOVE_NOCOALESCE (1 = no coalescing)
/// - `button_flags`: Bitfield for button events (see [Button Flags](https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawmouse#members))
///   - 0x0001: Left button down
///   - 0x0002: Left button up
///   - 0x0004: Right button down
///   - 0x0008: Right button up
///   - 0x0010: Middle button down
///   - 0x0020: Middle button up
///   - 0x0040: Button 4 down
///   - 0x0080: Button 4 up
///   - 0x0100: Button 5 down
///   - 0x0200: Button 5 up
///   - 0x0400: Mouse wheel (vertical)
///   - 0x0800: Mouse wheel (horizontal)
/// - `button_data`: Wheel delta or button data (e.g., wheel movement amount)
/// - `raw_buttons`: Raw button bitmask (rarely used)
/// - `last_x`: Relative or absolute X movement (see `flags`)
/// - `last_y`: Relative or absolute Y movement (see `flags`)
/// - `extra_information`: Additional info from the driver or hardware
#[derive(Debug, Clone)]
pub struct RawMouseEvent {
    /// Mouse event type flags (see above)
    pub flags: u16,
    /// Button event flags (see above)
    pub button_flags: u16,
    /// Wheel delta or button data
    pub button_data: u16,
    /// Raw button bitmask
    pub raw_buttons: u32,
    /// X movement (relative or absolute)
    pub last_x: i32,
    /// Y movement (relative or absolute)
    pub last_y: i32,
    /// Additional driver/hardware info
    pub extra_information: u32,
}

impl From<&RAWMOUSE> for RawMouseEvent {
    fn from(mouse: &RAWMOUSE) -> Self {
        // SAFETY: Union layout per Windows API docs
        let (button_flags, button_data) = unsafe {
            let anon = mouse.Anonymous.Anonymous;
            (anon.usButtonFlags, anon.usButtonData)
        };
        Self {
            flags: mouse.usFlags.0,
            button_flags,
            button_data,
            raw_buttons: mouse.ulRawButtons,
            last_x: mouse.lLastX,
            last_y: mouse.lLastY,
            extra_information: mouse.ulExtraInformation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Input::{RAWKEYBOARD, RAWMOUSE};

    #[test]
    fn test_keyboard_event_from_raw() {
        let raw = RAWKEYBOARD {
            MakeCode: 42,
            Flags: 1,
            Reserved: 0,
            VKey: 65,
            Message: 256,
            ExtraInformation: 123,
        };

        let event = RawKeyboardEvent::from(&raw);
        assert_eq!(event.make_code, 42);
        assert_eq!(event.flags, 1);
        assert_eq!(event.vkey, 65);
        assert_eq!(event.message, 256);
        assert_eq!(event.extra_information, 123);
    }

    #[test]
    fn test_mouse_event_from_raw() {
        let raw = RAWMOUSE {
            usFlags: windows::Win32::UI::Input::MOUSE_STATE(0x01u16),
            ulRawButtons: 0x8000,
            lLastX: 10,
            lLastY: -20,
            ulExtraInformation: 0xDEADBEEF,
            Anonymous: unsafe {
                let mut anon = std::mem::zeroed::<windows::Win32::UI::Input::RAWMOUSE_0>();
                anon.Anonymous.usButtonFlags = 0x0001;
                anon.Anonymous.usButtonData = 120;
                anon
            },
        };

        let event = RawMouseEvent::from(&raw);
        assert_eq!(event.flags, 0x01);
        assert_eq!(event.button_flags, 0x0001);
        assert_eq!(event.button_data, 120);
        assert_eq!(event.raw_buttons, 0x8000);
        assert_eq!(event.last_x, 10);
        assert_eq!(event.last_y, -20);
        assert_eq!(event.extra_information, 0xDEADBEEF);
    }
}
