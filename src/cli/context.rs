use crate::cdp::CdpClient;
use anyhow::Result;
use std::path::PathBuf;

/// Context shared between step handlers
pub struct Context {
    /// The CDP client for communicating with Electron
    pub cdp_client: CdpClient,
    /// Output directory for screenshots
    pub output_dir: PathBuf,
    /// Current viewport size (width, height)
    pub window_size: Option<(u32, u32)>,
}

impl Context {
    /// Create a new context with a CDP client
    pub fn new(cdp_client: CdpClient, output_dir: PathBuf) -> Self {
        Self {
            cdp_client,
            output_dir,
            window_size: None,
        }
    }

    /// Get the full path for a screenshot file
    pub fn screenshot_path(&self, filename: &str) -> PathBuf {
        self.output_dir.join(filename)
    }
}
