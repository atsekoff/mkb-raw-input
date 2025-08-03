//! Keyboard event types and conversions for the Raw Input API

use windows::Win32::UI::Input::RAWKEYBOARD;
use windows::Win32::UI::WindowsAndMessaging::{WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP};

/// Represents a keyboard event message based on legacy Windows keyboard input notification messages.
///
/// This enum is derived from the `Message` field in the Windows `RAWKEYBOARD` structure,
/// which contains standard Windows messages like `WM_KEYDOWN`, `WM_KEYUP`, etc.
/// These are the legacy keyboard input notification messages that would normally be
/// generated for this key event in a standard Windows application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum KeyEventMessage {
    /// Key was pressed down (corresponds to WM_KEYDOWN)
    KeyDown = WM_KEYDOWN,
    /// Key was released (corresponds to WM_KEYUP)
    KeyUp = WM_KEYUP,
    /// System key was pressed down (e.g., Alt key combinations) (corresponds to WM_SYSKEYDOWN)
    SysKeyDown = WM_SYSKEYDOWN,
    /// System key was released (corresponds to WM_SYSKEYUP)
    SysKeyUp = WM_SYSKEYUP,
    /// Unknown or unsupported message type
    Unknown(u32),
}

impl From<u32> for KeyEventMessage {
    fn from(message: u32) -> Self {
        match message {
            WM_KEYDOWN => Self::KeyDown,
            WM_KEYUP => Self::KeyUp,
            WM_SYSKEYDOWN => Self::SysKeyDown,
            WM_SYSKEYUP => Self::SysKeyUp,
            other => Self::Unknown(other),
        }
    }
}

/// Flags for a keyboard event from the raw keyboard controller data.
///
/// These flags represent the low-level hardware state reported by the keyboard controller.
/// Note that there is some redundancy between these flags and the `KeyEventType` (derived from
/// the Windows message) - for example, both `flags.key_up` and `event_type` indicate whether
/// a key was pressed or released. This is because Raw Input provides both the raw hardware data
/// and the Windows message interpretation.
///
/// For most applications, using `event_type` is more intuitive, but the raw flags provide
/// additional hardware-level information that can be useful for advanced keyboard handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyFlags {
    /// Whether the key is an extended key (e.g., right ALT/CTRL, arrow keys)
    pub e0_prefix: bool,
    /// Whether the key was released (true) or pressed (false)
    /// This is the raw hardware state and corresponds to the RI_KEY_BREAK flag
    pub key_up: bool,
    /// Whether the key has an E0 prefix
    /// The E0 prefix is used to distinguish between duplicate keys on modern keyboards
    /// (e.g., right vs left Ctrl, numeric keypad vs. arrow keys)
    pub e1_prefix: bool,
}

pub(crate) const RI_KEY_BREAK: u16 = 0x01;
pub(crate) const RI_KEY_E0: u16 = 0x02;
pub(crate) const RI_KEY_E1: u16 = 0x04;

impl From<u16> for KeyFlags {
    fn from(flags: u16) -> Self {
        Self {
            // RI_KEY_MAKE (0): Key is down (no flag)
            // RI_KEY_BREAK (1): Key is up
            key_up: (flags & RI_KEY_BREAK) != 0,
            // RI_KEY_E0 (2): The scan code has the E0 prefix
            e0_prefix: (flags & RI_KEY_E0) != 0,
            // RI_KEY_E1 (4): The scan code has the E1 prefix
            e1_prefix: (flags & RI_KEY_E1) != 0,
        }
    }
}

/// Common virtual key codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum VirtualKey {
    // Control keys
    Backspace = 0x08,
    Tab = 0x09,
    Enter = 0x0D,
    Shift = 0x10,
    Control = 0x11,
    Alt = 0x12,
    Pause = 0x13,
    CapsLock = 0x14,
    Escape = 0x1B,
    Space = 0x20,
    PageUp = 0x21,
    PageDown = 0x22,
    End = 0x23,
    Home = 0x24,

    // Arrow keys
    Left = 0x25,
    Up = 0x26,
    Right = 0x27,
    Down = 0x28,

    // Special keys
    PrintScreen = 0x2C,
    Insert = 0x2D,
    Delete = 0x2E,

    // Number keys (0-9)
    Key0 = 0x30,
    Key1 = 0x31,
    Key2 = 0x32,
    Key3 = 0x33,
    Key4 = 0x34,
    Key5 = 0x35,
    Key6 = 0x36,
    Key7 = 0x37,
    Key8 = 0x38,
    Key9 = 0x39,

    // Letter keys (A-Z)
    A = 0x41,
    B = 0x42,
    C = 0x43,
    D = 0x44,
    E = 0x45,
    F = 0x46,
    G = 0x47,
    H = 0x48,
    I = 0x49,
    J = 0x4A,
    K = 0x4B,
    L = 0x4C,
    M = 0x4D,
    N = 0x4E,
    O = 0x4F,
    P = 0x50,
    Q = 0x51,
    R = 0x52,
    S = 0x53,
    T = 0x54,
    U = 0x55,
    V = 0x56,
    W = 0x57,
    X = 0x58,
    Y = 0x59,
    Z = 0x5A,

    // Function keys
    F1 = 0x70,
    F2 = 0x71,
    F3 = 0x72,
    F4 = 0x73,
    F5 = 0x74,
    F6 = 0x75,
    F7 = 0x76,
    F8 = 0x77,
    F9 = 0x78,
    F10 = 0x79,
    F11 = 0x7A,
    F12 = 0x7B,
    F13 = 0x7C,
    F14 = 0x7D,
    F15 = 0x7E,
    F16 = 0x7F,
    F17 = 0x80,
    F18 = 0x81,
    F19 = 0x82,
    F20 = 0x83,
    F21 = 0x84,
    F22 = 0x85,
    F23 = 0x86,
    F24 = 0x87,

    // Numpad keys
    Numpad0 = 0x60,
    Numpad1 = 0x61,
    Numpad2 = 0x62,
    Numpad3 = 0x63,
    Numpad4 = 0x64,
    Numpad5 = 0x65,
    Numpad6 = 0x66,
    Numpad7 = 0x67,
    Numpad8 = 0x68,
    Numpad9 = 0x69,
    NumpadMultiply = 0x6A,
    NumpadAdd = 0x6B,
    NumpadSeparator = 0x6C,
    NumpadSubtract = 0x6D,
    NumpadDecimal = 0x6E,
    NumpadDivide = 0x6F,

    // Other common keys
    NumLock = 0x90,
    ScrollLock = 0x91,
    LeftShift = 0xA0,
    RightShift = 0xA1,
    LeftControl = 0xA2,
    RightControl = 0xA3,
    LeftAlt = 0xA4,
    RightAlt = 0xA5,
    BrowserBack = 0xA6,
    BrowserForward = 0xA7,
    BrowserRefresh = 0xA8,
    BrowserStop = 0xA9,
    BrowserSearch = 0xAA,
    BrowserFavorites = 0xAB,
    BrowserHome = 0xAC,
    VolumeMute = 0xAD,
    VolumeDown = 0xAE,
    VolumeUp = 0xAF,
    MediaNextTrack = 0xB0,
    MediaPrevTrack = 0xB1,
    MediaStop = 0xB2,
    MediaPlayPause = 0xB3,
    LaunchMail = 0xB4,
    LaunchMediaSelect = 0xB5,
    LaunchApp1 = 0xB6,
    LaunchApp2 = 0xB7,
    /// ';:' for US
    OEM1 = 0xBA,
    /// '+' any country
    OEMPlus = 0xBB,
    /// ',' any country
    OEMComma = 0xBC,
    /// '-' any country
    OEMMinus = 0xBD,
    /// '.' any country
    OEMPeriod = 0xBE,
    /// '/?' for US
    OEM2 = 0xBF,
    /// '`~' for US
    OEM3 = 0xC0,
    /// '[{' for US
    OEM4 = 0xDB,
    /// '\|' for US
    OEM5 = 0xDC,
    /// ']}' for US
    OEM6 = 0xDD,
    /// '\'"' for US
    OEM8 = 0xDF,

    // Windows keys
    LWin = 0x5B,
    RWin = 0x5C,
    Apps = 0x5D, // Context menu key

    // Other keys are represented as Unknown
    Unknown(u16),
}

impl From<u16> for VirtualKey {
    fn from(vkey: u16) -> Self {
        match vkey {
            0x08 => Self::Backspace,
            0x09 => Self::Tab,
            0x0D => Self::Enter,
            0x10 => Self::Shift,
            0x11 => Self::Control,
            0x12 => Self::Alt,
            0x13 => Self::Pause,
            0x14 => Self::CapsLock,
            0x1B => Self::Escape,
            0x20 => Self::Space,
            0x21 => Self::PageUp,
            0x22 => Self::PageDown,
            0x23 => Self::End,
            0x24 => Self::Home,
            0x25 => Self::Left,
            0x26 => Self::Up,
            0x27 => Self::Right,
            0x28 => Self::Down,
            0x2C => Self::PrintScreen,
            0x2D => Self::Insert,
            0x2E => Self::Delete,
            0x30 => Self::Key0,
            0x31 => Self::Key1,
            0x32 => Self::Key2,
            0x33 => Self::Key3,
            0x34 => Self::Key4,
            0x35 => Self::Key5,
            0x36 => Self::Key6,
            0x37 => Self::Key7,
            0x38 => Self::Key8,
            0x39 => Self::Key9,
            0x41 => Self::A,
            0x42 => Self::B,
            0x43 => Self::C,
            0x44 => Self::D,
            0x45 => Self::E,
            0x46 => Self::F,
            0x47 => Self::G,
            0x48 => Self::H,
            0x49 => Self::I,
            0x4A => Self::J,
            0x4B => Self::K,
            0x4C => Self::L,
            0x4D => Self::M,
            0x4E => Self::N,
            0x4F => Self::O,
            0x50 => Self::P,
            0x51 => Self::Q,
            0x52 => Self::R,
            0x53 => Self::S,
            0x54 => Self::T,
            0x55 => Self::U,
            0x56 => Self::V,
            0x57 => Self::W,
            0x58 => Self::X,
            0x59 => Self::Y,
            0x5A => Self::Z,
            0x5B => Self::LWin,
            0x5C => Self::RWin,
            0x5D => Self::Apps,
            0x60 => Self::Numpad0,
            0x61 => Self::Numpad1,
            0x62 => Self::Numpad2,
            0x63 => Self::Numpad3,
            0x64 => Self::Numpad4,
            0x65 => Self::Numpad5,
            0x66 => Self::Numpad6,
            0x67 => Self::Numpad7,
            0x68 => Self::Numpad8,
            0x69 => Self::Numpad9,
            0x6A => Self::NumpadMultiply,
            0x6B => Self::NumpadAdd,
            0x6C => Self::NumpadSeparator,
            0x6D => Self::NumpadSubtract,
            0x6E => Self::NumpadDecimal,
            0x6F => Self::NumpadDivide,
            0x70 => Self::F1,
            0x71 => Self::F2,
            0x72 => Self::F3,
            0x73 => Self::F4,
            0x74 => Self::F5,
            0x75 => Self::F6,
            0x76 => Self::F7,
            0x77 => Self::F8,
            0x78 => Self::F9,
            0x79 => Self::F10,
            0x7A => Self::F11,
            0x7B => Self::F12,
            0x7C => Self::F13,
            0x7D => Self::F14,
            0x7E => Self::F15,
            0x7F => Self::F16,
            0x80 => Self::F17,
            0x81 => Self::F18,
            0x82 => Self::F19,
            0x83 => Self::F20,
            0x84 => Self::F21,
            0x85 => Self::F22,
            0x86 => Self::F23,
            0x87 => Self::F24,
            0x90 => Self::NumLock,
            0x91 => Self::ScrollLock,
            0xA0 => Self::LeftShift,
            0xA1 => Self::RightShift,
            0xA2 => Self::LeftControl,
            0xA3 => Self::RightControl,
            0xA4 => Self::LeftAlt,
            0xA5 => Self::RightAlt,
            0xA6 => Self::BrowserBack,
            0xA7 => Self::BrowserForward,
            0xA8 => Self::BrowserRefresh,
            0xA9 => Self::BrowserStop,
            0xAA => Self::BrowserSearch,
            0xAB => Self::BrowserFavorites,
            0xAC => Self::BrowserHome,
            0xAD => Self::VolumeMute,
            0xAE => Self::VolumeDown,
            0xAF => Self::VolumeUp,
            0xB0 => Self::MediaNextTrack,
            0xB1 => Self::MediaPrevTrack,
            0xB2 => Self::MediaStop,
            0xB3 => Self::MediaPlayPause,
            0xB4 => Self::LaunchMail,
            0xB5 => Self::LaunchMediaSelect,
            0xB6 => Self::LaunchApp1,
            0xB7 => Self::LaunchApp2,
            0xBA => Self::OEM1,
            0xBB => Self::OEMPlus,
            0xBC => Self::OEMComma,
            0xBD => Self::OEMMinus,
            0xBE => Self::OEMPeriod,
            0xBF => Self::OEM2,
            0xC0 => Self::OEM3,
            0xDB => Self::OEM4,
            0xDC => Self::OEM5,
            0xDD => Self::OEM6,
            0xDF => Self::OEM8,
            other => Self::Unknown(other),
        }
    }
}

/// Data for a raw keyboard event, parsed from the Windows RAWKEYBOARD struct.
///
/// This struct provides a more ergonomic interface compared to the raw Windows API,
/// with enums for event types and key codes.
///
/// ## Handling Key Press/Release Events
///
/// There are two ways to determine if a key was pressed or released:
///
/// 1. Using `message`: Check if it's `KeyDown`/`SysKeyDown` or `KeyUp`/`SysKeyUp`.
///    ```rust,no_run
///    use mkb_raw_input::{RawKeyboardEvent, KeyEventMessage};
///    
///    fn process_keyboard_event(event: &RawKeyboardEvent) {
///        if event.message == KeyEventMessage::KeyDown || event.message == KeyEventMessage::SysKeyDown {
///            println!("Key was pressed: {:?}", event.vkey);
///        }
///    }
///    ```
///
/// 2. Using `key_up`: Check the raw hardware flag.
///    ```rust,no_run
///    use mkb_raw_input::RawKeyboardEvent;
///    
///    fn process_keyboard_event(event: &RawKeyboardEvent) {
///        if !event.key_up {
///            println!("Key was pressed: {:?}", event.vkey);
///        }
///    }
///    ```
///
/// For most applications, using `message` is recommended as it follows Windows conventions,
/// but the raw `key_up` flag can be useful for hardware-specific handling or when you need to
/// distinguish between physical keys that generate the same virtual key code.
/// Data for a raw keyboard event, parsed from the Windows RAWKEYBOARD struct.
///
/// This struct provides ergonomic and minimal information for keyboard events.
///
/// - `key_up` is `true` if the key was released, `false` if pressed (from the raw flags)
/// - `extended` is `true` if the key has the E0 or E1 prefix (extended key)
/// - `message` is the raw Windows message (e.g., WM_KEYDOWN, WM_KEYUP, etc.)
/// - `vkey` is the Windows virtual key code mapped to a Rust enum
/// - `make_code` is the hardware scan code
/// - `extra_information` is additional driver/hardware info
#[derive(Debug, Clone)]
pub struct RawKeyboardEvent {
    /// Scan code of the key (hardware-dependent)
    pub make_code: u16,
    /// Whether the key was released (true) or pressed (false), from the raw flags
    pub key_up: bool,
    /// Whether the key is extended (E0 or E1 prefix)
    pub extended: bool,
    /// The raw Windows message (WM_KEYDOWN, WM_KEYUP, etc.)
    pub message: KeyEventMessage,
    /// Virtual key code (Windows virtual key code mapped to a Rust enum)
    pub vkey: VirtualKey,
    /// Additional driver/hardware info
    pub extra_information: u32,
}

impl From<&RAWKEYBOARD> for RawKeyboardEvent {
    fn from(kbd: &RAWKEYBOARD) -> Self {
        let flags = kbd.Flags;
        let key_up = (flags & 0x01) != 0;
        let e0 = (flags & 0x02) != 0;
        let e1 = (flags & 0x04) != 0;
        let extended = e0 || e1;
        Self {
            make_code: kbd.MakeCode,
            key_up,
            extended,
            message: KeyEventMessage::from(kbd.Message),
            vkey: VirtualKey::from(kbd.VKey),
            extra_information: kbd.ExtraInformation,
        }
    }
}
