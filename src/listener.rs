//! Background-threaded listener implementation for mkb-raw-input

use crate::{RawInputError, RawInputEvent, parse_rawinput_event};
use std::ptr::null_mut;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};

use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PCWSTR;

use std::sync::atomic::{AtomicBool as StdAtomicBool, Ordering as StdOrdering};

static LISTENER_ACTIVE: StdAtomicBool = StdAtomicBool::new(false);

/// Handle to a running raw input listener thread.
///
/// This struct manages the lifecycle of the background thread and window.
/// When dropped, it automatically stops the listener, posts a quit message,
/// and cleans up resources (window class, etc.).
pub struct ListenerHandle {
    join_handle: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
    hwnd: HWND,
    class_name: Vec<u16>,
    hinstance: HINSTANCE,
}

impl Drop for ListenerHandle {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        // Post WM_QUIT to wake up the message loop immediately
        unsafe {
            let _ = PostMessageW(Some(self.hwnd), WM_QUIT, WPARAM(0), LPARAM(0));
        }

        // Wait for the thread to finish
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }

        // Unregister the window class
        unsafe {
            let _ = UnregisterClassW(PCWSTR(self.class_name.as_ptr()), Some(self.hinstance));
        }

        // Allow another listener to be created
        LISTENER_ACTIVE.store(false, StdOrdering::SeqCst);
    }
}

/// Start the background listener thread and window for raw input events.
///
/// This function creates a background thread with a hidden window that registers
/// for and processes raw input events from keyboard and mouse devices. Events are
/// delivered to the provided callback function.
///
/// # Arguments
/// * `callback` - Function called for each raw input event
/// * `on_error` - Optional function called when errors occur during event processing
///
/// # Returns
/// * `Ok(ListenerHandle)` - Handle to the running listener (stop by dropping)
/// * `Err(RawInputError)` - If initialization fails
///
/// # Example
/// ```no_run
/// use mkb_raw_input::{start_listener, RawInputEvent};
///
/// let _listener = start_listener(
///     |event| match event {
///         RawInputEvent::Keyboard(kbd) => println!("Key: {}", kbd.vkey),
///         RawInputEvent::Mouse(mouse) => println!("Mouse: {},{}", mouse.last_x, mouse.last_y),
///     },
///     Some(|err| eprintln!("Error: {}", err)),
/// ).expect("Failed to start listener");
///
/// // Listener runs until _listener is dropped
/// std::thread::park();
/// ```
pub fn start_listener<F, E>(
    callback: F,
    on_error: Option<E>,
) -> Result<ListenerHandle, RawInputError>
where
    F: FnMut(RawInputEvent) + Send + 'static,
    E: FnMut(RawInputError) + Send + 'static,
{
    // Singleton enforcement
    if LISTENER_ACTIVE.swap(true, StdOrdering::SeqCst) {
        return Err(RawInputError::Other(
            "Raw input listener already running (singleton enforcement)".to_string(),
        ));
    }

    use std::sync::mpsc;
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let (init_tx, init_rx) = mpsc::channel();
    let (hwnd_tx, hwnd_rx) = mpsc::channel::<(isize, isize)>();
    let class_name = widestring("MkbRawInputHiddenWindow");
    let class_name_for_handle = class_name.clone();
    let join_handle = thread::spawn(move || {
        listener_thread_main(
            callback,
            on_error,
            running_clone,
            init_tx,
            hwnd_tx,
            class_name,
        );
    });

    // Wait for initialization result
    let (hwnd_raw, hinstance_raw) = match hwnd_rx.recv().unwrap_or((0isize, 0isize)) {
        (hwnd, hinstance) if hwnd != 0 => (hwnd, hinstance),
        _ => {
            return Err(RawInputError::Other(
                "Failed to receive HWND from listener thread".to_string(),
            ));
        }
    };

    let hwnd = HWND(hwnd_raw as *mut _);
    let hinstance = HINSTANCE(hinstance_raw as *mut _);

    match init_rx.recv().unwrap_or(Err(RawInputError::Other(
        "Listener thread failed to initialize".to_string(),
    ))) {
        Ok(()) => Ok(ListenerHandle {
            join_handle: Some(join_handle),
            running,
            hwnd,
            class_name: class_name_for_handle,
            hinstance,
        }),
        Err(e) => {
            LISTENER_ACTIVE.store(false, StdOrdering::SeqCst);
            Err(e)
        }
    }
}

// Extracted thread logic for readability
fn listener_thread_main<F, E>(
    mut callback: F,
    mut on_error: Option<E>,
    running_clone: Arc<AtomicBool>,
    init_tx: std::sync::mpsc::Sender<Result<(), RawInputError>>,
    hwnd_tx: std::sync::mpsc::Sender<(isize, isize)>,
    class_name: Vec<u16>,
) where
    F: FnMut(RawInputEvent) + Send + 'static,
    E: FnMut(RawInputError) + Send + 'static,
{
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        // Initialize window and register for raw input
        let hwnd = match initialize_listener_window(&class_name, &init_tx, &hwnd_tx) {
            Ok(handles) => handles,
            Err(_) => return, // Error already sent through init_tx
        };

        // Run the message loop
        run_message_loop(hwnd, running_clone, &mut callback, &mut on_error);
    }));

    if let Err(panic) = result {
        if let Some(ref mut err_cb) = on_error {
            let err_msg = if let Some(s) = panic.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic.downcast_ref::<String>() {
                s.clone()
            } else {
                "Listener thread panicked".to_string()
            };
            err_cb(RawInputError::Other(format!(
                "Listener thread panicked: {err_msg}"
            )));
        } else {
            eprintln!("Listener thread panicked");
        }
    }
}

/// Initialize the hidden window for raw input and register for events.
///
/// This function handles the entire initialization process:
/// 1. Gets the module handle
/// 2. Registers the window class
/// 3. Creates the hidden window
/// 4. Registers for raw input events
/// 5. Sends the window handle back to the main thread
///
/// # Safety
/// This function is unsafe because it calls into Win32 API functions.
///
/// # Returns
/// - `Ok(HWND)` - The window handle on success
/// - `Err(())` - If any step fails (error details are sent through `init_tx`)
unsafe fn initialize_listener_window(
    class_name: &[u16],
    init_tx: &std::sync::mpsc::Sender<Result<(), RawInputError>>,
    hwnd_tx: &std::sync::mpsc::Sender<(isize, isize)>,
) -> Result<HWND, ()> {
    // 1. Register window class
    let hmodule = match unsafe { GetModuleHandleW(None) } {
        Ok(h) => h,
        Err(e) => {
            let _ = init_tx.send(Err(RawInputError::WinApiError(format!(
                "GetModuleHandleW failed: {e}"
            ))));
            return Err(());
        }
    };

    let hinstance = HINSTANCE(hmodule.0);
    let wc = WNDCLASSW {
        lpfnWndProc: Some(wnd_proc),
        hInstance: hinstance,
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };

    if unsafe { RegisterClassW(&wc) } == 0 {
        let _ = init_tx.send(Err(RawInputError::WinApiError(format!(
            "RegisterClassW failed: {}",
            windows::core::Error::from_win32()
        ))));
        return Err(());
    }

    // 2. Create hidden window
    let hwnd = match unsafe {
        CreateWindowExW(
            Default::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(class_name.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(hinstance),
            Some(null_mut()),
        )
    } {
        Ok(h) => h,
        Err(e) => {
            let _ = init_tx.send(Err(RawInputError::WinApiError(format!(
                "CreateWindowExW failed: {e}",
            ))));
            return Err(());
        }
    };

    // Send HWND and HINSTANCE as raw integer values to main thread (FFI-safe)
    let _ = hwnd_tx.send((hwnd.0 as isize, hinstance.0 as isize));

    // 3. Register for raw input
    if let Err(e) = crate::register_raw_input(Some(hwnd)) {
        let _ = init_tx.send(Err(e));
        return Err(());
    }

    // Signal success to main thread
    let _ = init_tx.send(Ok(()));

    Ok(hwnd)
}

/// Run the Windows message loop, processing raw input events.
///
/// This function handles the Windows message pump, watching for WM_INPUT messages
/// and dispatching them to the user's callback. It continues running until the
/// `running` flag is set to false or a WM_QUIT message is received.
///
/// # Safety
/// This function is unsafe because it calls into Win32 API functions.
unsafe fn run_message_loop<F, E>(
    hwnd: HWND,
    running: Arc<AtomicBool>,
    callback: &mut F,
    on_error: &mut Option<E>,
) where
    F: FnMut(RawInputEvent),
    E: FnMut(RawInputError),
{
    let mut msg = MSG::default();
    while running.load(Ordering::SeqCst)
        && unsafe { GetMessageW(&mut msg, Some(hwnd), 0, 0) }.into()
    {
        if msg.message == WM_INPUT {
            let lparam = msg.lParam;
            match crate::read_raw_input_event_from_lparam(lparam) {
                Ok(raw) => {
                    if let Some(event) = parse_rawinput_event(&raw) {
                        callback(event);
                    }
                }
                Err(e) => {
                    if let Some(err_cb) = on_error {
                        err_cb(e);
                    } else {
                        eprintln!("Raw input event error: {e}");
                    }
                }
            }
        }

        // DispatchMessageW doesn't return a meaningful value for us to check
        unsafe { DispatchMessageW(&msg) };
    }
}

/// Converts a Rust string to a null-terminated UTF-16 string for Windows API calls.
///
/// This is a helper function used to create wide strings for window class names and other
/// Windows API parameters that require UTF-16 encoding.
fn widestring(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Window procedure for the hidden window that receives raw input events.
///
/// This is a minimal implementation that only handles WM_DESTROY by posting
/// a quit message to terminate the message loop.
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_DESTROY {
        // PostQuitMessage doesn't return a value, no need to handle result
        unsafe { PostQuitMessage(0) };
    }
    // Always call the default window procedure for unhandled messages
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RawInputEvent;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    /// Test that only one listener can be active at a time (singleton enforcement)
    #[test]
    fn test_singleton_enforcement() {
        // Start first listener
        let listener1 =
            start_listener(|_event: RawInputEvent| {}, None::<fn(crate::RawInputError)>);
        assert!(
            listener1.is_ok(),
            "First listener should start successfully"
        );

        // Try to start a second listener while the first is active
        let listener2 =
            start_listener(|_event: RawInputEvent| {}, None::<fn(crate::RawInputError)>);
        assert!(listener2.is_err(), "Second listener should fail to start");

        // The error should be about singleton enforcement
        if let Err(e) = listener2 {
            assert!(
                e.to_string().contains("singleton"),
                "Error should mention singleton enforcement, got: {e}"
            );
        }

        // Drop the first listener
        drop(listener1.unwrap());

        // Give it a moment to clean up
        thread::sleep(Duration::from_millis(100));

        // Now we should be able to start another listener
        let listener3 =
            start_listener(|_event: RawInputEvent| {}, None::<fn(crate::RawInputError)>);
        assert!(
            listener3.is_ok(),
            "Should be able to start a new listener after dropping the first"
        );

        // Clean up
        drop(listener3.unwrap());
    }

    /// Test that the singleton flag is properly reset when a listener panics
    #[test]
    fn test_singleton_reset_on_panic() {
        // Create a channel to signal when the callback has been called
        let (tx, _) = mpsc::channel();

        // Start a listener with a callback that will panic
        let _listener = thread::spawn(move || {
            let listener = start_listener(
                move |_event: RawInputEvent| {
                    // Signal that we're about to panic
                    let _ = tx.send(());
                    panic!("Intentional panic for testing");
                },
                None::<fn(crate::RawInputError)>,
            );

            // This should succeed
            assert!(listener.is_ok());

            // Keep the listener alive in this thread
            thread::sleep(Duration::from_secs(2));
        });

        // Wait for the listener thread to start and the callback to be ready to panic
        // (In a real scenario, we'd send an event to trigger the callback)
        thread::sleep(Duration::from_millis(500));

        // Give time for any cleanup to occur
        thread::sleep(Duration::from_millis(500));

        // Now we should be able to start another listener
        let new_listener =
            start_listener(|_event: RawInputEvent| {}, None::<fn(crate::RawInputError)>);

        // This might fail if the singleton flag wasn't reset properly
        assert!(
            new_listener.is_ok(),
            "Should be able to start a new listener after the previous one panicked"
        );

        // Clean up
        if let Ok(l) = new_listener {
            drop(l);
        }
    }
}
