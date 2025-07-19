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
use mkb_raw_input::{start_listener, RawInputEvent, RawInputError};

let _listener = start_listener(
    |event| {
        if let RawInputEvent::Keyboard(kbd) = event {
            // Check if it's a key press (not a key release)
            let is_key_up = (kbd.flags & 0x0001) != 0;
            if !is_key_up {
                match kbd.vkey {
                    // Virtual key codes (VK_*) from Windows
                    0x1B => println!("ESC pressed"),  // VK_ESCAPE
                    0x41 => println!("A pressed"),    // VK_A
                    0x20 => println!("Space pressed"), // VK_SPACE
                    _ => println!("Key pressed: {}", kbd.vkey),
                }
            }
        }
    },
    None::<fn(RawInputError)>,
).expect("Failed to start listener");
```

## Handling Mouse Events

```rust
use mkb_raw_input::{start_listener, RawInputEvent, RawInputError};

let _listener = start_listener(
    |event| {
        if let RawInputEvent::Mouse(mouse) = event {
            // Check for mouse movement
            if mouse.last_x != 0 || mouse.last_y != 0 {
                println!("Mouse moved: ({}, {})", mouse.last_x, mouse.last_y);
            }

            // Check for button presses
            match mouse.button_flags {
                0x0001 => println!("Left button down"),
                0x0002 => println!("Left button up"),
                0x0004 => println!("Right button down"),
                0x0008 => println!("Right button up"),
                0x0010 => println!("Middle button down"),
                0x0020 => println!("Middle button up"),
                0x0400 => {
                    // Mouse wheel vertical
                    let wheel_delta = mouse.button_data as i16;
                    println!("Mouse wheel: {}", wheel_delta);
                },
                _ => {}, // Other button combinations
            }
        }
    },
    None::<fn(RawInputError)>,
).expect("Failed to start listener");
```

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
