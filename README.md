# mkb-raw-input

A safe, ergonomic, and minimal Rust library for capturing raw keyboard and mouse input on Windows using the Windows Raw Input API. This library provides global input capture (even when your application is not focused) with a simple callback-based API and no Win32 boilerplate required.

## Features

- **Global input capture**: Receive keyboard and mouse events regardless of window focus
- **Minimal, ergonomic API**: Just call `start_listener` with a closure
- **Safe Rust abstractions**: Minimal unsafe code, properly encapsulated
- **Proper resource management**: Automatic cleanup on drop
- **Singleton enforcement**: Only one listener can be active at a time
- **Optional error callbacks**: For robust error handling
- **No Win32 message loop or HWND management required**: All handled internally

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
mkb-raw-input = { git = "https://github.com/atsekoff/mkb-raw-input" }
```

## Basic Usage

```rust
use mkb_raw_input::{start_listener, RawInputEvent};

fn main() {
    // Start the listener with a callback for events
    let _listener = start_listener(
        |event| {
            match event {
                RawInputEvent::Keyboard(kbd) => println!("Keyboard: {:?}", kbd),
                RawInputEvent::Mouse(mouse) => println!("Mouse: {:?}", mouse),
            }
        },
        // Optional error callback
        Some(|err| eprintln!("Raw input error: {}", err)),
    ).expect("Failed to start listener");

    println!("Listening for raw input events. Press Ctrl+C to exit.");
    std::thread::park(); // Keep the process alive
}
```

## Handling Keyboard Events

```rust
use mkb_raw_input::{start_listener, RawInputEvent, RawKeyboardEvent, VirtualKey};

let _listener = start_listener(
    |event| {
        if let RawInputEvent::Keyboard(kbd) = event {
            // Use the ergonomic API: key_up, vkey, message
            if !kbd.key_up {
                match kbd.vkey {
                    VirtualKey::Escape => println!("ESC pressed"),
                    VirtualKey::A => println!("A pressed"),
                    VirtualKey::Space => println!("Space pressed"),
                    other => println!("Key pressed: {:?}", other),
                }
            }
        }
    },
    None::<fn(_)> // No error callback
).expect("Failed to start listener");
```

- `kbd.key_up`: `true` if the key was released, `false` if pressed
- `kbd.vkey`: ergonomic Rust enum for virtual key codes
- `kbd.message`: the Windows message (KeyDown, KeyUp, etc.)


## Handling Mouse Events

```rust
use mkb_raw_input::{start_listener, RawInputEvent, MouseButtonAction};

let _listener = start_listener(
    |event| {
        if let RawInputEvent::Mouse(mouse) = event {
            // Check for mouse movement
            if mouse.last_x != 0 || mouse.last_y != 0 {
                println!("Mouse moved: ({}, {})", mouse.last_x, mouse.last_y);
            }

            // Check for button and wheel actions
            match mouse.button_action {
                MouseButtonAction::LeftDown => println!("Left button down"),
                MouseButtonAction::LeftUp => println!("Left button up"),
                MouseButtonAction::RightDown => println!("Right button down"),
                MouseButtonAction::RightUp => println!("Right button up"),
                MouseButtonAction::MiddleDown => println!("Middle button down"),
                MouseButtonAction::MiddleUp => println!("Middle button up"),
                MouseButtonAction::WheelUp(lines) => println!("Mouse wheel up: {} lines", lines),
                MouseButtonAction::WheelDown(lines) => println!("Mouse wheel down: {} lines", lines),
                MouseButtonAction::WheelRight(lines) => println!("Mouse wheel right: {} lines", lines),
                MouseButtonAction::WheelLeft(lines) => println!("Mouse wheel left: {} lines", lines),
                _ => {},
            }
        }
    },
    None::<fn(_)> // No error callback
).expect("Failed to start listener");
```

- Mouse wheel events are reported as lines scrolled (already multiplied by the user's system setting).
- If the system is set to "page scroll", the value will be `i32::MAX` or `i32::MIN` to indicate a page scroll direction.
- All mouse button and movement actions are reported via ergonomic enums.


## Stopping the Listener

The listener will automatically stop and clean up resources when the `ListenerHandle` is dropped:

```rust
use mkb_raw_input::{start_listener, RawInputEvent, RawInputError};
use std::time::Duration;
use std::thread;

// Start the listener
let listener = start_listener(
    |event| {
        // Process events
    },
    None::<fn(RawInputError)>,
).expect("Failed to start listener");

// Listen for 5 seconds
thread::sleep(Duration::from_secs(5));

// Stop the listener by dropping the handle
drop(listener);
println!("Listener stopped");
```

## Error Handling

```rust
use mkb_raw_input::{start_listener, RawInputEvent, RawInputError};

let _listener = start_listener(
    |event| {
        // Handle events
    },
    Some(|err| {
        match err {
            RawInputError::WinApiError(msg) => {
                eprintln!("Windows API error: {}", msg);
                // You might want to log this or take specific action
            },
            RawInputError::Other(msg) => {
                eprintln!("Other error: {}", msg);
            },
        }
    }),
).expect("Failed to start listener");
```

## Platform Support

- Windows only

## Implementation Details

This library is built on top of the [`windows`](https://crates.io/crates/windows) crate for safe and idiomatic access to the Win32 API from Rust. It creates a hidden window with a message loop running in a background thread to receive raw input events.

## License

Licensed under either of:

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
