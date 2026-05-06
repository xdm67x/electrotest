//! Chrome DevTools Protocol (CDP) client module
//!
//! This module provides a client for communicating with Electron's
//! Chrome DevTools Protocol to enable automation and testing.

pub mod client;
pub mod messages;

// Main export - the CDP client for Electron automation
pub use client::CdpClient;
