use std::os::unix::net::UnixStream;
use std::io::{BufRead, BufReader, Write};
// no shared state required currently
use std::thread;
use crate::shared::{FrontendMessage, BackendMessage, ClipboardItemPreview};

const SOCKET_PATH: &str = "/tmp/cursor-clip.sock";

/// Frontend client for communicating with the backend
pub struct FrontendClient {
    writer: UnixStream,
    _recv_handle: thread::JoinHandle<()>,
}

impl FrontendClient {
    /// Create a new client
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let stream = UnixStream::connect(SOCKET_PATH)?;
        let reader_stream = stream.try_clone()?;

        // Central receiving loop: single place to handle ALL backend messages
        let handle = thread::spawn(move || {
            let mut reader = BufReader::new(reader_stream);
            loop {
                let mut line = String::new();
                let Ok(n) = reader.read_line(&mut line) else { break; };
                if n == 0 { break; }
                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }
                let Ok(msg) = serde_json::from_str::<BackendMessage>(trimmed) else { continue; };

                // Central dispatch: directly call corresponding functions
                match &msg {
                    BackendMessage::NewItem { item } => {
                        FrontendClient::handle_new_item(item.clone());
                    }
                    BackendMessage::History { items } => {
                        FrontendClient::handle_history(items.clone());
                    }
                    BackendMessage::ClipboardSet => {
                        FrontendClient::handle_clipboard_set();
                    }
                    BackendMessage::HistoryCleared => {
                        FrontendClient::handle_history_cleared();
                    }
                    BackendMessage::Error { message } => {
                        FrontendClient::handle_error(message);
                    }
                }

            }
        });

        Ok(Self { writer: stream, _recv_handle: handle })
    }

    // ================= Direct handlers for incoming messages =================
    fn handle_new_item(item: ClipboardItemPreview) {
        println!(
            "[ipc_client] NewItem received: id={} type={} preview=\"{}\" ts={}",
            item.item_id,
            item.content_type.as_str(),
            item.content_preview,
            item.timestamp
        );
        // TODO: integrate with UI here if needed
    }

    fn handle_history(items: Vec<ClipboardItemPreview>) {
        println!("[ipc_client] History received ({} items)", items.len());
        // TODO: integrate with UI here if needed
    }

    fn handle_clipboard_set() {
        println!("[ipc_client] ClipboardSet received");
    }

    fn handle_history_cleared() {
        println!("[ipc_client] HistoryCleared received");
    }

    fn handle_error(message: &str) {
        eprintln!("[ipc_client] Error received: {}", message);
    }

    /// Send: write a message to the backend (non-blocking w.r.t. response)
    pub fn send(&mut self, message: &FrontendMessage) -> Result<(), Box<dyn std::error::Error>> {
        let message_json = serde_json::to_string(message)?;
        self.writer.write_all(message_json.as_bytes())?;
        self.writer.write_all(b"\n")?;
        Ok(())
    }

    /// Get clipboard history
    pub fn get_history(&mut self) -> Result<Vec<ClipboardItemPreview>, Box<dyn std::error::Error>> {
        // Deprecated: no longer returns data synchronously.
        // Trigger an async request; the receiver thread will handle History when it arrives.
        let _ = self.send(&FrontendMessage::GetHistory)?;
        Ok(Vec::new())
    }

    /// Set clipboard by ID 
    pub fn set_clipboard_by_id(&mut self, id: u64) -> Result<(), Box<dyn std::error::Error>> {
        self.send(&FrontendMessage::SetClipboardById { id })
    }

    /// Clear history
    pub fn clear_history(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.send(&FrontendMessage::ClearHistory)
    }
}