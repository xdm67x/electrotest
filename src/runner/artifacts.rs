use std::path::Path;

use crate::runner::RunError;

pub fn write_failure_artifacts(screenshot_path: &Path, trace_path: &Path) -> Result<(), RunError> {
    if let Some(parent) = screenshot_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if let Some(parent) = trace_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(screenshot_path, b"fixture failure screenshot")?;
    std::fs::write(trace_path, b"fixture trace archive")?;
    Ok(())
}
