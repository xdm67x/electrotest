use camino::Utf8PathBuf;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub paths: PathsConfig,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AppMode {
    Launch,
    Attach,
}

impl AppMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Launch => "launch",
            Self::Attach => "attach",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub mode: AppMode,
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    pub endpoint: Option<String>,
    pub endpoint_file: Option<Utf8PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct PathsConfig {
    pub features: Vec<Utf8PathBuf>,
    pub steps: Vec<Utf8PathBuf>,
    pub artifacts: Utf8PathBuf,
}
