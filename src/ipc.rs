//! IPC module for single instance application support
//! Uses Unix Domain Socket for inter-process communication

use std::io::{Write, BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use log::{info, debug, error, warn};

/// Get the socket path for IPC (~/.saten.sock)
pub fn get_socket_path() -> PathBuf {
    let home = dirs::home_dir().expect("Failed to get home directory");
    home.join(".saten.sock")
}

/// Result of trying to become the primary instance
pub enum InstanceResult {
    /// This is the primary instance, receiver for incoming file paths is returned
    Primary {
        receiver: mpsc::Receiver<PathBuf>,
        socket_path: PathBuf,
    },
    /// Another instance is already running, file path was sent
    Secondary,
}

/// Try to connect to existing instance or become the primary instance
///
/// If another instance is running:
///   - Send the file path to it (if provided)
///   - Return InstanceResult::Secondary
///
/// If no other instance:
///   - Create socket listener
///   - Return InstanceResult::Primary with the receiver
pub fn try_become_primary(file_path: Option<PathBuf>) -> InstanceResult {
    let socket_path = get_socket_path();

    // Try to connect to existing instance
    if let Ok(mut stream) = UnixStream::connect(&socket_path) {
        debug!("Connected to existing instance");

        // Send file path if provided
        if let Some(path) = file_path {
            let path_str = path.to_string_lossy();
            if let Err(e) = writeln!(stream, "{}", path_str) {
                error!("Failed to send file path to primary instance: {}", e);
            } else {
                info!("Sent file path to primary instance: {}", path_str);
            }
        }

        return InstanceResult::Secondary;
    }

    // No existing instance, become primary
    // First, remove stale socket file if exists
    if socket_path.exists() {
        if let Err(e) = std::fs::remove_file(&socket_path) {
            warn!("Failed to remove stale socket file: {}", e);
        }
    }

    // Create socket listener
    match UnixListener::bind(&socket_path) {
        Ok(listener) => {
            info!("Listening on socket: {:?}", socket_path);

            // Set non-blocking for the listener
            if let Err(e) = listener.set_nonblocking(true) {
                warn!("Failed to set non-blocking mode: {}", e);
            }

            let (sender, receiver) = mpsc::channel();
            let path_for_cleanup = socket_path.clone();

            // Spawn listener thread
            thread::spawn(move || {
                listener_loop(listener, sender);
            });

            InstanceResult::Primary {
                receiver,
                socket_path: path_for_cleanup,
            }
        }
        Err(e) => {
            error!("Failed to create socket listener: {}", e);
            // Fall through and run anyway (degraded mode without IPC)
            let (_sender, receiver) = mpsc::channel();
            InstanceResult::Primary {
                receiver,
                socket_path,
            }
        }
    }
}

/// Main listener loop (runs in separate thread)
fn listener_loop(listener: UnixListener, sender: mpsc::Sender<PathBuf>) {
    loop {
        match listener.accept() {
            Ok((stream, _addr)) => {
                debug!("Accepted connection from secondary instance");
                handle_connection(stream, &sender);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Non-blocking, sleep a bit and retry
                thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => {
                error!("Error accepting connection: {}", e);
                thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}

/// Handle a connection from a secondary instance
fn handle_connection(stream: UnixStream, sender: &mpsc::Sender<PathBuf>) {
    let reader = BufReader::new(stream);

    for line in reader.lines() {
        match line {
            Ok(path_str) => {
                if !path_str.is_empty() {
                    let path = PathBuf::from(path_str);
                    info!("Received file path from secondary instance: {:?}", path);
                    if let Err(e) = sender.send(path) {
                        error!("Failed to send path to main thread: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Error reading from connection: {}", e);
                break;
            }
        }
    }
}

/// Clean up socket file on application exit
pub fn cleanup_socket(socket_path: &PathBuf) {
    if socket_path.exists() {
        if let Err(e) = std::fs::remove_file(socket_path) {
            warn!("Failed to remove socket file on cleanup: {}", e);
        } else {
            debug!("Removed socket file: {:?}", socket_path);
        }
    }
}

/// Type alias for the IPC receiver
type IpcReceiverType = std::sync::Arc<std::sync::Mutex<Option<mpsc::Receiver<PathBuf>>>>;

/// Wrapper for hashing the receiver (uses pointer address)
#[derive(Clone)]
struct IpcReceiverWrapper(IpcReceiverType);

impl std::hash::Hash for IpcReceiverWrapper {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::sync::Arc::as_ptr(&self.0).hash(state);
    }
}

/// Create an iced subscription for IPC file reception
pub fn ipc_subscription(
    receiver: IpcReceiverType
) -> iced::Subscription<PathBuf> {
    iced::Subscription::run_with(
        IpcReceiverWrapper(receiver),
        |wrapper: &IpcReceiverWrapper| {
            let receiver = wrapper.0.clone();
            futures::stream::unfold(receiver, |receiver: IpcReceiverType| async move {
                loop {
                    // Check for new file paths
                    let path: Option<PathBuf> = {
                        let guard = receiver.lock().unwrap();
                        if let Some(ref rx) = *guard {
                            rx.try_recv().ok()
                        } else {
                            None
                        }
                    };

                    if let Some(path) = path {
                        return Some((path, receiver));
                    }

                    // Yield to prevent busy loop
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            })
        }
    )
}
