use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    Ping,
    LaunchApp {
        command: String,
        args: Vec<String>,
    },
    AttachApp {
        endpoint: String,
    },
    SwitchWindow {
        target: WindowTarget,
    },
    CurrentWindowTitle,
    Click {
        window_id: String,
        locator: Vec<LocatorPayload>,
    },
    Screenshot {
        window_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    Pong,
    AppLaunched {
        window_id: String,
    },
    AppAttached {
        window_id: String,
    },
    WindowSwitched {
        window_id: String,
        description: String,
    },
    CurrentWindowTitle {
        title: String,
    },
    Clicked,
    ScreenshotTaken {
        path: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WindowTarget {
    Title { value: String },
    Index { value: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LocatorPayload {
    Explicit { selector: String },
    TestId { value: String },
    RoleName { role: String, name: String },
    Text { value: String },
}

impl From<crate::steps::Locator> for LocatorPayload {
    fn from(value: crate::steps::Locator) -> Self {
        match value {
            crate::steps::Locator::Explicit(selector) => Self::Explicit { selector },
            crate::steps::Locator::TestId(value) => Self::TestId { value },
            crate::steps::Locator::RoleName { role, name } => Self::RoleName { role, name },
            crate::steps::Locator::Text(value) => Self::Text { value },
        }
    }
}

impl From<crate::steps::WindowTarget> for WindowTarget {
    fn from(value: crate::steps::WindowTarget) -> Self {
        match value {
            crate::steps::WindowTarget::Title(value) => Self::Title { value },
            crate::steps::WindowTarget::Index(value) => Self::Index { value },
        }
    }
}
