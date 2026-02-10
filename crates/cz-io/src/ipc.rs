use std::fs;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::thread;

/// A simple broadcast server using Unix Domain Sockets.
/// Pushes notifications to all connected observers (like cz-hub).
pub struct IpcServer {
    clients: Arc<Mutex<Vec<UnixStream>>>,
}

impl IpcServer {
    pub fn start(path: &str) -> std::io::Result<Self> {
        // Clean up existing socket file
        if fs::metadata(path).is_ok() {
            fs::remove_file(path)?;
        }

        let listener = UnixListener::bind(path)?;
        let clients = Arc::new(Mutex::new(Vec::new()));
        let clients_clone = clients.clone();

        // Background thread to accept connections
        thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(s) = stream {
                    let _ = s.set_nonblocking(true);
                    let mut lock = clients_clone.lock().unwrap();
                    lock.push(s);
                }
            }
        });

        Ok(Self { clients })
    }

    /// Sends a message to all connected clients.
    /// Removes clients that have disconnected.
    pub fn broadcast(&self, msg: &[u8]) {
        let mut lock = self.clients.lock().unwrap();
        lock.retain_mut(|client| {
            // We use write_all. If it fails (e.g. Broken pipe), the client is dropped.
            client.write_all(msg).is_ok()
        });
    }
}
