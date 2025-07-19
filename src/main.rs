//! Test platform for mkb-raw-input library development
//!
//! This binary will allow live validation of keyboard and mouse event capture.

// use mkb_raw_input::ffi;
use mkb_raw_input::start_listener;

fn main() {
    let _listener = start_listener(
        |event| {
            println!("RawInputEvent: {event:?}");
        },
        Some(|err| eprintln!("Raw input runtime error: {err}")),
    )
    .expect("Failed to start listener");
    println!("Listening for raw input events. Press Ctrl+C to exit.");
    // Keep the process alive
    std::thread::park();
}
