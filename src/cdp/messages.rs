//! CDP (Chrome DevTools Protocol) message types

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// CDP message ID for request/response correlation
pub type MessageId = u64;

/// Base CDP message structure sent to browser
#[derive(Debug, Clone, Serialize)]
pub struct CdpRequest {
    pub id: MessageId,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// CDP response structure from browser
/// Some fields may not be used directly but are part of the CDP protocol
#[derive(Debug, Clone, Deserialize)]
pub struct CdpResponse {
    pub id: Option<MessageId>,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<CdpError>,
    // These fields are part of the CDP spec but not currently used
    #[serde(default)]
    #[allow(dead_code)]
    pub method: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub params: Option<Value>,
}

/// CDP error structure
#[derive(Debug, Clone, Deserialize)]
pub struct CdpError {
    pub code: i32,
    pub message: String,
    // Reserved for future CDP protocol extensions
    #[serde(default)]
    #[allow(dead_code)]
    pub data: Option<Value>,
}

/// Target info from /json/list endpoint
/// Represents an available debugging target (page, window, etc.)
#[derive(Debug, Clone, Deserialize)]
pub struct TargetInfo {
    pub id: String,
    // Human-readable title of the target (available for logging/debugging)
    #[allow(dead_code)]
    pub title: String,
    // URL of the target page (available for logging/debugging)
    #[allow(dead_code)]
    pub url: String,
    #[serde(rename = "type")]
    pub target_type: String,
    #[serde(rename = "webSocketDebuggerUrl")]
    pub websocket_url: Option<String>,
}

/// Result from Runtime.evaluate response
#[derive(Debug, Clone, Deserialize)]
pub struct EvaluateResult {
    pub result: RemoteObject,
    #[serde(default)]
    pub exception_details: Option<ExceptionDetails>,
}

/// Remote object returned from JavaScript evaluation
#[derive(Debug, Clone, Deserialize)]
pub struct RemoteObject {
    // Type of the object (e.g., "string", "number", "object")
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub object_type: String,
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Details about a JavaScript exception
#[derive(Debug, Clone, Deserialize)]
pub struct ExceptionDetails {
    // Unique identifier for this exception
    #[allow(dead_code)]
    pub exception_id: i32,
    pub text: String,
    pub line_number: i32,
    // Column where the exception occurred (not always available)
    #[allow(dead_code)]
    pub column_number: i32,
}

/// Window bounds for Browser.setWindowBounds
/// Note: Not currently used but kept for future CDP feature support
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct WindowBounds {
    pub width: i32,
    pub height: i32,
}

/// Window state for Browser.setWindowBounds
/// Note: Not currently used but kept for future CDP feature support
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
    Fullscreen,
}
