//! FFI bindings and safe wrappers for Windows Raw Input API (keyboard and mouse)
//!
//! This module uses the `windows` crate for Win32 API access and provides the
//! necessary types and functions for device registration and event handling.

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::{
    RAWINPUTDEVICE, RAWINPUTHEADER, RIDEV_INPUTSINK, RIDEV_NOLEGACY, RegisterRawInputDevices,
};
use windows::core::Result as WinResult;

// Usage page and usage IDs for keyboard and mouse
pub const HID_USAGE_PAGE_GENERIC: u16 = 0x01;
pub const HID_USAGE_GENERIC_MOUSE: u16 = 0x02;
pub const HID_USAGE_GENERIC_KEYBOARD: u16 = 0x06;

/// Registers for raw input from keyboard and mouse devices.
pub fn register_keyboard_mouse(hwnd: Option<HWND>) -> WinResult<()> {
    let devices = [
        RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_MOUSE,
            dwFlags: RIDEV_NOLEGACY | RIDEV_INPUTSINK,
            hwndTarget: hwnd.unwrap_or_default(),
        },
        RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_KEYBOARD,
            dwFlags: RIDEV_NOLEGACY | RIDEV_INPUTSINK,
            hwndTarget: hwnd.unwrap_or_default(),
        },
    ];
    unsafe { RegisterRawInputDevices(&devices, std::mem::size_of::<RAWINPUTDEVICE>() as u32) }
}

use std::ffi::c_void;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::UI::Input::{GetRawInputData, HRAWINPUT, RAWINPUT, RID_INPUT};

/// Reads and parses a raw input event from a WM_INPUT message.
///
/// # Arguments
/// * `lparam` - The LPARAM from the WM_INPUT message.
///
/// # Returns
/// A RAWINPUT structure on success, or an error.
pub fn read_raw_input_event(lparam: LPARAM) -> Result<RAWINPUT, windows::core::Error> {
    unsafe {
        let hrawinput = HRAWINPUT(lparam.0 as *mut c_void);
        let mut size = 0u32;
        let rc = GetRawInputData(
            hrawinput,
            RID_INPUT,
            None,
            &mut size,
            std::mem::size_of::<RAWINPUTHEADER>() as u32,
        );

        if rc == u32::MAX {
            return Err(windows::core::Error::from_win32());
        }

        let mut raw_input_data_buffer = vec![0u8; size as usize];
        let rc = GetRawInputData(
            hrawinput,
            RID_INPUT,
            Some(raw_input_data_buffer.as_mut_ptr() as *mut c_void),
            &mut size,
            std::mem::size_of::<RAWINPUTHEADER>() as u32,
        );

        if rc == u32::MAX {
            return Err(windows::core::Error::from_win32());
        }

        let raw_input_data_ptr = raw_input_data_buffer.as_ptr() as *const RAWINPUT;
        let raw_input_data = raw_input_data_ptr.read();
        Ok(raw_input_data)
    }
}

// FFI tests are not included here because they would require interaction with the actual Windows API,
// which is unreliable in a test environment. Instead, we test the error conversion logic in lib.rs.
