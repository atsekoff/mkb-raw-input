//! Example: Listen for raw keyboard and mouse events using mkb-raw-input

use mkb_raw_input::{RawInputEvent, start_listener};

fn main() {
    let _listener = start_listener(
        |event| match event {
            RawInputEvent::Keyboard(kbd) => println!("Keyboard: {kbd:?}"),
            RawInputEvent::Mouse(mouse) => println!("Mouse: {mouse:?}"),
        },
        Some(|err| eprintln!("Raw input runtime error: {err}")),
    )
    .expect("Failed to start listener");
    println!("Listening for raw input events. Press Ctrl+C to exit.");
    std::thread::park();
}
