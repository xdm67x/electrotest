//! Chrome DevTools Protocol (CDP) client module
//!
//! This module provides a client for communicating with Electron's
//! Chrome DevTools Protocol to enable automation and testing.

pub mod client;
pub mod messages;

pub use client::{CdpClient, ConnectionState};
pub use messages::{
    CdpError, CdpRequest, CdpResponse, EvaluateResult, MessageId, RemoteObject, TargetInfo,
};
