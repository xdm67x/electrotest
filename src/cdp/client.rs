//! CDP (Chrome DevTools Protocol) client for Electron automation

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock, oneshot};
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::cdp::messages::{CdpRequest, CdpResponse, EvaluateResult, MessageId, TargetInfo};

/// Type for WebSocket writer
pub type WsWriter =
    futures_util::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

/// Type alias for the WebSocket writer stored in CdpClient
/// This makes the CdpClient struct definition cleaner and more readable
type CdpWriter = Arc<Mutex<Option<WsWriter>>>;

/// Connection state for CDP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

/// High-level CDP client for Electron automation
pub struct CdpClient {
    port: u16,
    state: Arc<RwLock<ConnectionState>>,
    writer: CdpWriter,
    pending_requests: Arc<Mutex<HashMap<MessageId, oneshot::Sender<Result<CdpResponse>>>>>,
    next_id: Arc<Mutex<MessageId>>,
}

impl CdpClient {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            writer: Arc::new(Mutex::new(None)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    /// Connect to CDP by discovering targets and connecting to the first available one
    pub async fn connect(&mut self) -> Result<()> {
        // Check if already connected
        {
            let state = self.state.read().await;
            if *state == ConnectionState::Connected {
                return Ok(());
            }
        }

        // Set connecting state
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Connecting;
        }

        // List available targets
        let targets = Self::list_targets(self.port).await?;

        let target = targets
            .into_iter()
            .find(|t| t.target_type == "page" || t.target_type == "window")
            .ok_or_else(|| anyhow!("No CDP targets available on port {}", self.port))?;

        let ws_url = target
            .websocket_url
            .ok_or_else(|| anyhow!("Target {} has no WebSocket debugger URL", target.id))?;

        // Establish WebSocket connection
        let (ws_stream, _) = connect_async(&ws_url)
            .await
            .map_err(|e| anyhow!("WebSocket connection failed: {}", e))?;

        let (write, mut read) = ws_stream.split();

        // Store writer
        {
            let mut writer = self.writer.lock().await;
            *writer = Some(write);
        }

        // Set connected state
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Connected;
        }

        // Spawn reader task
        let pending = Arc::clone(&self.pending_requests);
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        // Parse response and send to waiting channel if found
                        if let Ok(response) = serde_json::from_str::<CdpResponse>(&text)
                            && let Some(id) = response.id
                        {
                            let mut pending = pending.lock().await;
                            if let Some(tx) = pending.remove(&id) {
                                let _ = tx.send(Ok(response));
                            }
                        }
                    }
                    Ok(Message::Close(_)) | Err(_) => break,
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// Send a CDP request and await response
    async fn send_request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<CdpResponse> {
        // Check state
        {
            let state = self.state.read().await;
            if *state != ConnectionState::Connected {
                bail!("Not connected to CDP endpoint");
            }
        }

        // Generate message ID
        let id = {
            let mut id = self.next_id.lock().await;
            let current = *id;
            *id += 1;
            current
        };

        let request = CdpRequest {
            id,
            method: method.to_string(),
            params,
        };

        // Create response channel
        let (tx, rx) = oneshot::channel();

        // Store pending request
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(id, tx);
        }

        // Serialize and send
        let json = serde_json::to_string(&request)?;

        {
            let mut writer = self.writer.lock().await;
            if let Some(ref mut w) = *writer {
                w.send(Message::Text(json.into())).await?;
            } else {
                bail!("WebSocket writer not initialized");
            }
        }

        // Wait for response with timeout
        match timeout(Duration::from_secs(30), rx).await {
            Ok(Ok(Ok(response))) => Ok(response),
            Ok(Ok(Err(e))) => Err(e),
            Ok(Err(_)) => bail!("Response channel closed"),
            Err(_) => {
                let mut pending = self.pending_requests.lock().await;
                pending.remove(&id);
                bail!("Request timed out")
            }
        }
    }

    /// Evaluate JavaScript in the Electron context
    pub async fn evaluate(&self, expression: &str) -> Result<String> {
        let params = json!({
            "expression": expression,
            "returnByValue": true,
            "awaitPromise": true,
        });

        let response = self.send_request("Runtime.evaluate", Some(params)).await?;

        if let Some(error) = response.error {
            bail!("CDP error {}: {}", error.code, error.message);
        }

        let result = response
            .result
            .ok_or_else(|| anyhow!("No result in CDP response"))?;

        let eval_result: EvaluateResult = serde_json::from_value(result)?;

        if let Some(exception) = eval_result.exception_details {
            bail!(
                "JavaScript exception: {} at line {}",
                exception.text,
                exception.line_number
            );
        }

        // Format result
        match eval_result.result.value {
            Some(value) => {
                if value.is_null() {
                    Ok("null".to_string())
                } else if let Some(s) = value.as_str() {
                    Ok(s.to_string())
                } else {
                    Ok(value.to_string())
                }
            }
            None => Ok(eval_result
                .result
                .description
                .unwrap_or_else(|| "undefined".to_string())),
        }
    }

    /// Take a screenshot and save to file
    pub async fn screenshot(&self, path: &Path) -> Result<()> {
        // Enable required domains
        self.send_request("Page.enable", None).await?;

        let params = json!({
            "format": "png",
            "fromSurface": true,
        });

        let response = self
            .send_request("Page.captureScreenshot", Some(params))
            .await?;

        if let Some(error) = response.error {
            bail!("CDP error {}: {}", error.code, error.message);
        }

        let result = response
            .result
            .ok_or_else(|| anyhow!("No result in CDP response"))?;

        let data = result
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("No screenshot data in response"))?;

        let decoded = base64::engine::general_purpose::STANDARD
            .decode(data)
            .map_err(|e| anyhow!("Failed to decode base64 screenshot: {}", e))?;

        tokio::fs::write(path, decoded).await?;

        Ok(())
    }

    /// Navigate to a URL
    pub async fn navigate(&self, url: &str) -> Result<()> {
        let params = json!({
            "url": url,
        });

        let response = self.send_request("Page.navigate", Some(params)).await?;

        if let Some(error) = response.error {
            bail!("Navigation error: {}", error.message);
        }

        Ok(())
    }

    /// Get the page title
    pub async fn get_title(&self) -> Result<String> {
        self.evaluate("document.title").await
    }

    /// List available CDP targets
    pub async fn list_targets(port: u16) -> Result<Vec<TargetInfo>> {
        let url = format!("http://127.0.0.1:{}/json/list", port);
        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            bail!("Failed to list CDP targets: HTTP {}", response.status());
        }

        let targets: Vec<TargetInfo> = response.json().await?;
        Ok(targets)
    }

    /// Disconnect from CDP
    pub async fn disconnect(&mut self) -> Result<()> {
        // Close WebSocket
        {
            let mut writer = self.writer.lock().await;
            if let Some(mut w) = writer.take() {
                let _ = w.close().await;
            }
        }

        // Clear pending requests
        {
            let mut pending = self.pending_requests.lock().await;
            pending.clear();
        }

        // Set disconnected state
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Disconnected;
        }

        Ok(())
    }
}
