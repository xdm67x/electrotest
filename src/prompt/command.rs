#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptCommand {
    Help,
    Status,
    Pid,
    History,
    Clear,
    Exit,
    Empty,
    Unknown(String),
}

impl PromptCommand {
    pub fn parse(input: &str) -> Self {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Self::Empty;
        }

        match trimmed {
            "help" | "h" | "?" => Self::Help,
            "status" => Self::Status,
            "pid" => Self::Pid,
            "history" => Self::History,
            "clear" | "cls" => Self::Clear,
            "exit" | "quit" | "q" => Self::Exit,
            other => Self::Unknown(other.to_owned()),
        }
    }

    pub fn help_text() -> &'static str {
        "\
Available commands:
  help, h, ?   Show this help
  status       Show Electron process status
  pid          Show the tracked Electron PID
  history      Show entered command history
  clear, cls   Clear the terminal
  exit, quit   Exit the prompt"
    }
}
