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
#[derive(Debug, Clone, Deserialize)]
pub struct CdpResponse {
    pub id: Option<MessageId>,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<CdpError>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CdpError {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub data: Option<Value>,
}

/// Target info from /json/list endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct TargetInfo {
    pub id: String,
    pub title: String,
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

#[derive(Debug, Clone, Deserialize)]
pub struct RemoteObject {
    #[serde(rename = "type")]
    pub object_type: String,
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExceptionDetails {
    pub exception_id: i32,
    pub text: String,
    pub line_number: i32,
    pub column_number: i32,
}

/// Window bounds for Browser.setWindowBounds
#[derive(Debug, Clone, Serialize)]
pub struct WindowBounds {
    pub width: i32,
    pub height: i32,
}

/// Window state for Browser.setWindowBounds
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
    Fullscreen,
}
