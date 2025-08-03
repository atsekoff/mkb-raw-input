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
//! use mkb_raw_input::{start_listener, RawInputEvent, RawKeyboardEvent, RawInputError, VirtualKey};
//!
//! let _listener = start_listener(
//!     |event| {
//!         if let RawInputEvent::Keyboard(kbd) = event {
//!             // Check if it's a key press (not a key release)
//!             if !kbd.key_up {
//!                 match kbd.vkey {
//!                     // Use the VirtualKey enum for better type safety
//!                     VirtualKey::Escape => println!("ESC pressed"),
//!                     VirtualKey::A => println!("A pressed"),
//!                     VirtualKey::Space => println!("Space pressed"),
//!                     other => println!("Key pressed: {:?}", other),
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
//! use mkb_raw_input::{start_listener, RawInputEvent, RawMouseEvent, RawInputError, MouseButtonAction};
//!
//! let _listener = start_listener(
//!     |event| {
//!         if let RawInputEvent::Mouse(mouse) = event {
//!             // Check for mouse movement
//!             if mouse.last_x != 0 || mouse.last_y != 0 {
//!                 println!("Mouse moved: ({}, {})", mouse.last_x, mouse.last_y);
//!             }
//!
//!             // Check for button actions using the enhanced enum
//!             match mouse.button_action {
//!                 MouseButtonAction::LeftDown => println!("Left button down"),
//!                 MouseButtonAction::LeftUp => println!("Left button up"),
//!                 MouseButtonAction::RightDown => println!("Right button down"),
//!                 MouseButtonAction::RightUp => println!("Right button up"),
//!                 MouseButtonAction::MiddleDown => println!("Middle button down"),
//!                 MouseButtonAction::MiddleUp => println!("Middle button up"),
//!                 MouseButtonAction::XButton1Down => println!("X1 button down"),
//!                 MouseButtonAction::XButton1Up => println!("X1 button up"),
//!                 MouseButtonAction::XButton2Down => println!("X2 button down"),
//!                 MouseButtonAction::XButton2Up => println!("X2 button up"),
//!                 MouseButtonAction::WheelUp(lines) => println!("Mouse wheel up: {} lines", lines),
//!                 MouseButtonAction::WheelDown(lines) => println!("Mouse wheel down: {} lines", lines),
//!                 MouseButtonAction::WheelRight(lines) => println!("Mouse wheel right: {} lines", lines),
//!                 MouseButtonAction::WheelLeft(lines) => println!("Mouse wheel left: {} lines", lines),
//!                 MouseButtonAction::None => {}, // No button action
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
mod keyboard;
mod mouse;

pub use event::RawInputEvent;
pub use keyboard::RawKeyboardEvent;
pub use mouse::RawMouseEvent;
// Re-export key and mouse related enums for easier access
pub use keyboard::{KeyEventMessage, KeyFlags, VirtualKey};
pub use mouse::{MouseButtonAction, MouseMoveMode};
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
            kbd.Flags = 0; // Key down (no RI_KEY_BREAK)
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
                assert_eq!(kbd.key_up, false);
                assert_eq!(kbd.extended, false);
                assert_eq!(kbd.vkey, VirtualKey::A);
                assert_eq!(kbd.message, KeyEventMessage::KeyDown);
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
        use crate::mouse::{MOUSE_BUTTON_WHEEL_VERTICAL, MOUSE_MOVE_VIRTUAL_DESKTOP, WHEEL_DELTA};
        unsafe {
            let mouse = &mut raw_input.data.mouse;
            mouse.usFlags = windows::Win32::UI::Input::MOUSE_STATE(MOUSE_MOVE_VIRTUAL_DESKTOP); // MOUSE_MOVE_ABSOLUTE | VIRTUAL_DESKTOP
            mouse.ulRawButtons = 0;
            mouse.lLastX = 100;
            mouse.lLastY = 200;
            mouse.ulExtraInformation = 456;

            // We need to initialize the union field carefully
            // This is a bit tricky with the windows crate unions
            let anon = &mut mouse.Anonymous;
            let inner = &mut anon.Anonymous;
            inner.usButtonFlags = MOUSE_BUTTON_WHEEL_VERTICAL; // Wheel vertical
            inner.usButtonData = WHEEL_DELTA as u16;
        }

        // Parse the event
        let event = parse_rawinput_event(&raw_input);

        // Verify the result
        assert!(event.is_some(), "Should parse mouse event");
        match event {
            Some(RawInputEvent::Mouse(mouse)) => {
                assert_eq!(mouse.move_mode, MouseMoveMode::Absolute);
                if let MouseButtonAction::WheelUp(lines_scrolled) = mouse.button_action {
                    let lines = crate::mouse::get_wheel_scroll_lines()
                        .unwrap_or(crate::mouse::WHEEL_SCROLL_LINES_DEFAULT);
                    assert_eq!(lines_scrolled, lines);
                } else {
                    panic!("Expected WheelUp mouse action");
                }
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
