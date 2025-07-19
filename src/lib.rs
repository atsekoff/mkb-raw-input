//! # mkb-raw-input
//!
//! Safe, ergonomic, and minimal Rust library for capturing raw keyboard and mouse input on Windows
//! using the Windows Raw Input API. Provides global input capture (even when your app is not focused),
//! with a simple callback-based API and no Win32 boilerplate required.
//!
//! ## Features
//! - Global keyboard and mouse input capture (focus not required)
//! - Minimal, ergonomic API: just call [`start_listener`] with a closure
//! - Optional runtime error callback for robust error handling
//! - No Win32 message loop or HWND management required
//! - Singleton enforcement to prevent multiple listeners
//! - Automatic resource cleanup on drop
//!
//! ## Basic Example
//! ```rust,no_run
//! use mkb_raw_input::{start_listener, RawInputEvent};
//!
//! let _listener = start_listener(
//!     |event| {
//!         match event {
//!             RawInputEvent::Keyboard(kbd) => println!("Keyboard: {kbd:?}"),
//!             RawInputEvent::Mouse(mouse) => println!("Mouse: {mouse:?}"),
//!         }
//!     },
//!     Some(|err| eprintln!("Raw input runtime error: {err}")),
//! ).expect("Failed to start listener");
//! println!("Listening for raw input events. Press Ctrl+C to exit.");
//! std::thread::park();
//! ```
//!
//! ## Handling Keyboard Events
//! ```rust,no_run
//! use mkb_raw_input::{start_listener, RawInputEvent, RawKeyboardEvent, RawInputError};
//!
//! let _listener = start_listener(
//!     |event| {
//!         if let RawInputEvent::Keyboard(kbd) = event {
//!             // Check if it's a key press (not a key release)
//!             let is_key_up = (kbd.flags & 0x0001) != 0;
//!             if !is_key_up {
//!                 match kbd.vkey {
//!                     // Virtual key codes (VK_*) from Windows
//!                     0x1B => println!("ESC pressed"),  // VK_ESCAPE
//!                     0x41 => println!("A pressed"),    // VK_A
//!                     0x20 => println!("Space pressed"), // VK_SPACE
//!                     _ => println!("Key pressed: {}", kbd.vkey),
//!                 }
//!             }
//!         }
//!     },
//!     None::<fn(RawInputError)>,
//! ).expect("Failed to start listener");
//! std::thread::park();
//! ```
//!
//! ## Handling Mouse Events
//! ```rust,no_run
//! use mkb_raw_input::{start_listener, RawInputEvent, RawMouseEvent, RawInputError};
//!
//! let _listener = start_listener(
//!     |event| {
//!         if let RawInputEvent::Mouse(mouse) = event {
//!             // Check for mouse movement
//!             if mouse.last_x != 0 || mouse.last_y != 0 {
//!                 println!("Mouse moved: ({}, {})", mouse.last_x, mouse.last_y);
//!             }
//!
//!             // Check for button presses
//!             match mouse.button_flags {
//!                 0x0001 => println!("Left button down"),
//!                 0x0002 => println!("Left button up"),
//!                 0x0004 => println!("Right button down"),
//!                 0x0008 => println!("Right button up"),
//!                 0x0010 => println!("Middle button down"),
//!                 0x0020 => println!("Middle button up"),
//!                 0x0400 => {
//!                     // Mouse wheel vertical
//!                     let wheel_delta = mouse.button_data as i16;
//!                     println!("Mouse wheel: {}", wheel_delta);
//!                 },
//!                 _ => {}, // Other button combinations
//!             }
//!         }
//!     },
//!     None::<fn(RawInputError)>,
//! ).expect("Failed to start listener");
//! std::thread::park();
//! ```
//!
//! ## Error Handling
//! ```rust,no_run
//! use mkb_raw_input::{start_listener, RawInputEvent, RawInputError};
//!
//! let _listener = start_listener(
//!     |event| {
//!         // Handle events
//!     },
//!     Some(|err| {
//!         match err {
//!             RawInputError::WinApiError(msg) => {
//!                 eprintln!("Windows API error: {}", msg);
//!                 // You might want to log this or take specific action
//!             },
//!             RawInputError::Other(msg) => {
//!                 eprintln!("Other error: {}", msg);
//!             },
//!         }
//!     }),
//! ).expect("Failed to start listener");
//! std::thread::park();
//! ```
//!
//! ## Stopping the Listener
//! The listener will automatically stop and clean up resources when the `ListenerHandle` is dropped:
//! ```rust,no_run
//! use mkb_raw_input::{start_listener, RawInputEvent, RawInputError};
//! use std::time::Duration;
//! use std::thread;
//!
//! // Start the listener
//! let listener = start_listener(
//!     |event| {
//!         // Process events
//!     },
//!     None::<fn(RawInputError)>,
//! ).expect("Failed to start listener");
//!
//! // Listen for 5 seconds
//! thread::sleep(Duration::from_secs(5));
//!
//! // Stop the listener by dropping the handle
//! drop(listener);
//! println!("Listener stopped");
//! ```
//!
//! ## API
//! - [`start_listener`] - Start a background listener for raw input events; provide an event callback and optional error callback.
//! - [`RawInputEvent`] - Enum for keyboard and mouse events.
//! - [`RawKeyboardEvent`] / [`RawMouseEvent`] - Ergonomic Rust structs for event data.
//! - [`ListenerHandle`] - Handle to the running listener; dropping this stops the listener.
//!
//! ## Platform
//! - Windows only
//!
//! ## Implementation
//!
//! This library is built on top of the [`windows`](https://crates.io/crates/windows) crate for
//! safe and idiomatic access to the Win32 API from Rust.

mod event;
mod ffi;

pub use event::{RawInputEvent, RawKeyboardEvent, RawMouseEvent};
use windows::Win32::UI::Input::RAWINPUT;

/// Registers the library to receive raw input from keyboard and mouse devices.
///
/// # Arguments
/// * `hwnd` - Optional window handle. If provided, registers for input on that window. If None, attempts process-wide registration (generally requires at least one window).
///
/// # Returns
/// Ok(()) on success, or an error if registration fails.
pub(crate) fn register_raw_input(
    hwnd: Option<windows::Win32::Foundation::HWND>,
) -> Result<(), RawInputError> {
    ffi::register_keyboard_mouse(hwnd).map_err(|e| RawInputError::WinApiError(format!("{e}")))
}

/// Reads and parses a raw input event from a WM_INPUT message LPARAM.
/// Returns a boxed RAWINPUT structure on success, or an error.
pub(crate) fn read_raw_input_event_from_lparam(
    lparam: windows::Win32::Foundation::LPARAM,
) -> Result<RAWINPUT, RawInputError> {
    ffi::read_raw_input_event(lparam).map_err(|e| RawInputError::WinApiError(format!("{e}")))
}

/// Parses a RAWINPUT struct into a high-level RawInputEvent (keyboard or mouse).
/// Returns None if the event type is not supported.
pub(crate) fn parse_rawinput_event(raw: &RAWINPUT) -> Option<RawInputEvent> {
    use windows::Win32::UI::Input::{RIM_TYPEKEYBOARD, RIM_TYPEMOUSE};
    unsafe {
        match raw.header.dwType {
            dwtype if dwtype == RIM_TYPEKEYBOARD.0 => {
                let kbd = &raw.data.keyboard;
                Some(RawInputEvent::Keyboard(RawKeyboardEvent::from(kbd)))
            }
            dwtype if dwtype == RIM_TYPEMOUSE.0 => {
                let mouse = &raw.data.mouse;
                Some(RawInputEvent::Mouse(RawMouseEvent::from(mouse)))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::zeroed;
    use windows::Win32::UI::Input::{RIM_TYPEKEYBOARD, RIM_TYPEMOUSE};
    use windows::core::Error;

    #[test]
    fn test_error_conversion_winapi() {
        // Test Windows API error conversion
        let win_error = Error::from_win32();
        let raw_error = RawInputError::WinApiError(format!("{win_error}"));

        // Check that the error message contains the Windows error information
        let error_msg = format!("{raw_error}");
        assert!(
            error_msg.contains("Windows API error"),
            "Error message should contain 'Windows API error', got: {error_msg}"
        );
    }

    #[test]
    fn test_error_conversion_other() {
        // Test Other error conversion
        let message = "Test error message";
        let raw_error = RawInputError::Other(message.to_string());

        // Check that the error message contains our custom message
        let error_msg = format!("{raw_error}");
        assert!(
            error_msg.contains(message),
            "Error message should contain the custom message, got: {error_msg}"
        );
    }

    #[test]
    fn test_parse_rawinput_event_keyboard() {
        // Create a keyboard RAWINPUT structure
        let mut raw_input: RAWINPUT = unsafe { zeroed() };
        raw_input.header.dwType = RIM_TYPEKEYBOARD.0;

        // Set keyboard data
        unsafe {
            let kbd = &mut raw_input.data.keyboard;
            kbd.MakeCode = 30; // 'A' key
            kbd.Flags = 0; // Key down
            kbd.VKey = 65; // VK_A
            kbd.Message = 256; // WM_KEYDOWN
            kbd.ExtraInformation = 123;
            // Need to set Reserved field for Windows API
            kbd.Reserved = 0;
        }

        // Parse the event
        let event = parse_rawinput_event(&raw_input);

        // Verify the result
        assert!(event.is_some(), "Should parse keyboard event");
        match event {
            Some(RawInputEvent::Keyboard(kbd)) => {
                assert_eq!(kbd.make_code, 30);
                assert_eq!(kbd.flags, 0);
                assert_eq!(kbd.vkey, 65);
                assert_eq!(kbd.message, 256);
                assert_eq!(kbd.extra_information, 123);
            }
            _ => panic!("Expected keyboard event"),
        }
    }

    #[test]
    fn test_parse_rawinput_event_mouse() {
        // Create a mouse RAWINPUT structure
        let mut raw_input: RAWINPUT = unsafe { zeroed() };
        raw_input.header.dwType = RIM_TYPEMOUSE.0;

        // Set mouse data
        unsafe {
            let mouse = &mut raw_input.data.mouse;
            mouse.usFlags = windows::Win32::UI::Input::MOUSE_STATE(0x01); // MOUSE_MOVE_ABSOLUTE
            mouse.ulRawButtons = 0;
            mouse.lLastX = 100;
            mouse.lLastY = 200;
            mouse.ulExtraInformation = 456;

            // We need to initialize the union field carefully
            // This is a bit tricky with the windows crate unions
            let buttons = &mut mouse.Anonymous;
            // Access the union field directly - this is unsafe but necessary for testing
            std::ptr::write(&mut buttons.Anonymous.usButtonFlags as *mut _, 0x0400u16);
            std::ptr::write(&mut buttons.Anonymous.usButtonData as *mut _, 120u16);
        }

        // Parse the event
        let event = parse_rawinput_event(&raw_input);

        // Verify the result
        assert!(event.is_some(), "Should parse mouse event");
        match event {
            Some(RawInputEvent::Mouse(mouse)) => {
                assert_eq!(mouse.flags, 0x01);
                assert_eq!(mouse.button_flags, 0x0400);
                assert_eq!(mouse.button_data, 120);
                assert_eq!(mouse.raw_buttons, 0);
                assert_eq!(mouse.last_x, 100);
                assert_eq!(mouse.last_y, 200);
                assert_eq!(mouse.extra_information, 456);
            }
            _ => panic!("Expected mouse event"),
        }
    }

    #[test]
    fn test_parse_rawinput_event_unsupported() {
        // Create a RAWINPUT with unsupported type
        let mut raw_input: RAWINPUT = unsafe { zeroed() };
        raw_input.header.dwType = 3; // Not keyboard (1) or mouse (2)

        // Parse the event
        let event = parse_rawinput_event(&raw_input);

        // Verify it returns None for unsupported types
        assert!(
            event.is_none(),
            "Should return None for unsupported input types"
        );
    }
}

mod listener;
pub use listener::{ListenerHandle, start_listener};

/// Error type for RawInput operations.
#[derive(Debug, thiserror::Error)]
pub enum RawInputError {
    #[error("Windows API error: {0}")]
    WinApiError(String),
    #[error("Other error: {0}")]
    Other(String),
}
