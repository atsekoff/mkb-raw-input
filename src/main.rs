//! Test platform for mkb-raw-input library development
//!
//! This binary will allow live validation of keyboard and mouse event capture.

use mkb_raw_input::{MouseButtonAction, RawInputEvent, start_listener};

fn main() {
    println!("=== mkb-raw-input Test Application ===");
    println!("This application demonstrates the enhanced keyboard and mouse event handling.");
    println!("Try pressing various keys including:");
    println!("- Regular keys (letters, numbers)");
    println!("- Function keys (F1-F24)");
    println!("- Numpad keys");
    println!("- Media keys (volume, playback)");
    println!("- Mouse buttons and movements");
    println!("Press Ctrl+C to exit.");
    println!("======================================\n");

    let _listener = start_listener(
        |event| match event {
            RawInputEvent::Keyboard(kbd) => {
                // Create a more descriptive state string
                let state = if kbd.key_up { "UP" } else { "DOWN" };

                println!(
                    "Keyboard: {:?} | State: {} | Message: {:?}",
                    kbd.vkey, state, kbd.message
                );
            }
            RawInputEvent::Mouse(mouse) => {
                let action_str = match mouse.button_action {
                    MouseButtonAction::None => "Movement".to_string(),
                    MouseButtonAction::WheelUp(lines) => format!("Wheel Up: {} lines", lines),
                    MouseButtonAction::WheelDown(lines) => format!("Wheel Down: {} lines", lines),
                    MouseButtonAction::WheelRight(lines) => format!("Wheel Right: {} lines", lines),
                    MouseButtonAction::WheelLeft(lines) => format!("Wheel Left: {} lines", lines),
                    other => format!("{:?}", other),
                };

                println!(
                    "Mouse: {} | Position: ({}, {}) | Mode: {:?}",
                    action_str, mouse.last_x, mouse.last_y, mouse.move_mode
                );
            }
        },
        Some(|err| eprintln!("Error: {}", err)),
    )
    .expect("Failed to start listener");

    // Keep the process alive
    std::thread::park();
}
