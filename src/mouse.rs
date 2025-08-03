//! Mouse event types and conversions for the Raw Input API

use windows::Win32::UI::Input::{MOUSE_MOVE_ABSOLUTE, MOUSE_MOVE_RELATIVE, RAWMOUSE};
use windows::Win32::UI::WindowsAndMessaging::{
    SPI_GETWHEELSCROLLLINES, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
};

/// The unit delta for one wheel notch (Microsoft standard)
pub const WHEEL_DELTA: i16 = 120;
/// The Windows default for lines to scroll per wheel notch
pub const WHEEL_SCROLL_LINES_DEFAULT: u32 = 3;

/// Represents mouse movement mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseMoveMode {
    /// Relative movement (most common)
    Relative,
    /// Absolute positioning
    Absolute,
    /// Virtual desktop positioning
    VirtualDesktop,
    /// Attribute mode
    AttributeChange,
    /// Unknown mode
    Unknown(u16),
}

pub(crate) const MOUSE_MOVE_VIRTUAL_DESKTOP: u16 = 0x01;
pub(crate) const MOUSE_MOVE_ATTRIBUTE_CHANGE: u16 = 0x04;

impl From<u16> for MouseMoveMode {
    fn from(flags: u16) -> Self {
        match flags {
            f if f == MOUSE_MOVE_RELATIVE.0 => Self::Relative,
            f if f == MOUSE_MOVE_ABSOLUTE.0 => Self::Absolute,
            f if f == (MOUSE_MOVE_ABSOLUTE.0 | MOUSE_MOVE_VIRTUAL_DESKTOP) => Self::VirtualDesktop,
            f if f == MOUSE_MOVE_ATTRIBUTE_CHANGE => Self::AttributeChange,
            other => Self::Unknown(other),
        }
    }
}

/// Mouse button action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButtonAction {
    /// Left button pressed
    LeftDown,
    /// Left button released
    LeftUp,
    /// Right button pressed
    RightDown,
    /// Right button released
    RightUp,
    /// Middle button pressed
    MiddleDown,
    /// Middle button released
    MiddleUp,
    /// X button 1 pressed
    XButton1Down,
    /// X button 1 released
    XButton1Up,
    /// X button 2 pressed
    XButton2Down,
    /// X button 2 released
    XButton2Up,
    /// Mouse wheel scrolled up by the given number of lines (system setting respected)
    WheelUp(u32),
    /// Mouse wheel scrolled down by the given number of lines (system setting respected)
    WheelDown(u32),
    /// Mouse wheel scrolled right by the given number of lines (system setting respected)
    WheelRight(u32),
    /// Mouse wheel scrolled left by the given number of lines (system setting respected)
    WheelLeft(u32),
    /// No button action
    None,
}

/// Converts button flags and data to a MouseButtonAction
pub(crate) fn get_wheel_scroll_lines() -> Result<u32, windows::core::Error> {
    let mut lines: u32 = 0;
    use std::ffi::c_void;
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETWHEELSCROLLLINES,
            0,
            Some(&mut lines as *mut u32 as *mut c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    };
    if ok.is_ok() {
        Ok(lines)
    } else {
        Err(windows::core::Error::from_win32())
    }
}

pub(crate) const MOUSE_BUTTON_LEFT_DOWN: u16 = 0x0001;
pub(crate) const MOUSE_BUTTON_LEFT_UP: u16 = 0x0002;
pub(crate) const MOUSE_BUTTON_RIGHT_DOWN: u16 = 0x0004;
pub(crate) const MOUSE_BUTTON_RIGHT_UP: u16 = 0x0008;
pub(crate) const MOUSE_BUTTON_MIDDLE_DOWN: u16 = 0x0010;
pub(crate) const MOUSE_BUTTON_MIDDLE_UP: u16 = 0x0020;
pub(crate) const MOUSE_BUTTON_XBUTTON1_DOWN: u16 = 0x0040;
pub(crate) const MOUSE_BUTTON_XBUTTON1_UP: u16 = 0x0080;
pub(crate) const MOUSE_BUTTON_XBUTTON2_DOWN: u16 = 0x0100;
pub(crate) const MOUSE_BUTTON_XBUTTON2_UP: u16 = 0x0200;
pub(crate) const MOUSE_BUTTON_WHEEL_VERTICAL: u16 = 0x0400;
pub(crate) const MOUSE_BUTTON_WHEEL_HORIZONTAL: u16 = 0x0800;

fn button_flags_to_action(button_flags: u16, button_data: u16) -> MouseButtonAction {
    match button_flags {
        MOUSE_BUTTON_LEFT_DOWN => MouseButtonAction::LeftDown,
        MOUSE_BUTTON_LEFT_UP => MouseButtonAction::LeftUp,
        MOUSE_BUTTON_RIGHT_DOWN => MouseButtonAction::RightDown,
        MOUSE_BUTTON_RIGHT_UP => MouseButtonAction::RightUp,
        MOUSE_BUTTON_MIDDLE_DOWN => MouseButtonAction::MiddleDown,
        MOUSE_BUTTON_MIDDLE_UP => MouseButtonAction::MiddleUp,
        MOUSE_BUTTON_XBUTTON1_DOWN => MouseButtonAction::XButton1Down,
        MOUSE_BUTTON_XBUTTON1_UP => MouseButtonAction::XButton1Up,
        MOUSE_BUTTON_XBUTTON2_DOWN => MouseButtonAction::XButton2Down,
        MOUSE_BUTTON_XBUTTON2_UP => MouseButtonAction::XButton2Up,
        MOUSE_BUTTON_WHEEL_VERTICAL => {
            let lines = get_wheel_scroll_lines().unwrap_or(WHEEL_SCROLL_LINES_DEFAULT);
            let notches = button_data as i16 as i32 / WHEEL_DELTA as i32;
            if notches > 0 {
                MouseButtonAction::WheelUp((notches * lines as i32) as u32)
            } else if notches < 0 {
                MouseButtonAction::WheelDown((-notches * lines as i32) as u32)
            } else {
                MouseButtonAction::None
            }
        }
        MOUSE_BUTTON_WHEEL_HORIZONTAL => {
            let lines = get_wheel_scroll_lines().unwrap_or(WHEEL_SCROLL_LINES_DEFAULT);
            let notches = button_data as i16 as i32 / WHEEL_DELTA as i32;
            if notches > 0 {
                MouseButtonAction::WheelRight((notches * lines as i32) as u32)
            } else if notches < 0 {
                MouseButtonAction::WheelLeft((-notches * lines as i32) as u32)
            } else {
                MouseButtonAction::None
            }
        }
        _ => MouseButtonAction::None,
    }
}

/// Data for a raw mouse event, parsed from the Windows RAWMOUSE struct.
///
/// This struct provides a more ergonomic interface compared to the raw Windows API,
/// with enums for movement modes and button actions.
#[derive(Debug, Clone)]
pub struct RawMouseEvent {
    /// Mouse movement mode
    pub move_mode: MouseMoveMode,
    /// Button action (if any)
    pub button_action: MouseButtonAction,
    /// Raw button state
    pub raw_buttons: u32,
    /// Movement in X direction
    pub last_x: i32,
    /// Movement in Y direction
    pub last_y: i32,
    /// Additional driver/hardware info
    pub extra_information: u32,
}

impl From<&RAWMOUSE> for RawMouseEvent {
    fn from(mouse: &RAWMOUSE) -> Self {
        // Extract button flags and data from the union
        let (button_flags, button_data) = unsafe {
            let anonymous = &mouse.Anonymous;
            let inner = &anonymous.Anonymous;
            (inner.usButtonFlags, inner.usButtonData)
        };

        Self {
            move_mode: MouseMoveMode::from(mouse.usFlags.0),
            button_action: button_flags_to_action(button_flags, button_data),
            raw_buttons: mouse.ulRawButtons,
            last_x: mouse.lLastX,
            last_y: mouse.lLastY,
            extra_information: mouse.ulExtraInformation,
        }
    }
}
