use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::cdp::client::{CdpClient, ConnectionState};
use crate::electron::{Electron, ElectronProcess};

const MAX_HISTORY: usize = 100;
const LOG_LIMIT: usize = 200;
const ALIVE_CHECK_INTERVAL: Duration = Duration::from_secs(2);

pub struct ConsolePrompt {
    electron: Electron,
    selected_process: ElectronProcess,
    input: String,
    history: Vec<String>,
    history_index: Option<usize>,
    logs: Vec<String>,
    should_exit: bool,
    last_alive_check: Instant,
    // CDP-related fields
    cdp_client: Option<CdpClient>,
    cdp_status: String,
}

pub enum ConsoleAction {
    Continue,
    BackToPicker,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PromptCommand {
    Help,
    Status,
    Pid,
    History,
    Clear,
    Exit,
    Refresh,
    Empty,
    // CDP commands
    Connect,
    Disconnect,
    Evaluate(String),
    Screenshot(String),
    Navigate(String),
    CdpStatus,
    Unknown(String),
}

impl PromptCommand {
    fn parse(input: &str) -> Self {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Self::Empty;
        }

        let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
        let cmd = parts[0];
        let args = parts.get(1).map(|s| s.to_string());

        match cmd {
            "help" | "h" | "?" => Self::Help,
            "status" => Self::Status,
            "pid" => Self::Pid,
            "history" => Self::History,
            "clear" | "cls" => Self::Clear,
            "refresh" | "reload" => Self::Refresh,
            "exit" | "quit" | "q" => Self::Exit,
            // CDP commands
            "connect" => Self::Connect,
            "disconnect" => Self::Disconnect,
            "evaluate" | "eval" | "js" => {
                if let Some(expr) = args {
                    Self::Evaluate(expr)
                } else {
                    Self::Unknown("evaluate: missing expression".to_string())
                }
            }
            "screenshot" | "shot" | "ss" => {
                Self::Screenshot(args.unwrap_or_else(|| "screenshot.png".to_string()))
            }
            "navigate" | "goto" | "url" => {
                if let Some(url) = args {
                    Self::Navigate(url)
                } else {
                    Self::Unknown("navigate: missing URL".to_string())
                }
            }
            "cdp-status" => Self::CdpStatus,
            other => Self::Unknown(other.to_owned()),
        }
    }

    fn help_text() -> &'static str {
        "\
Available commands:
  help, h, ?       Show this help
  status           Show Electron process status
  pid              Show the tracked Electron PID
  history          Show entered command history
  clear, cls       Clear the prompt output
  refresh, reload  Refresh attached process metadata
  connect          Connect to CDP (uses detected port or 9222)
  disconnect       Disconnect from CDP
  evaluate <js>    Evaluate JavaScript in Electron
  screenshot [path] Take a screenshot (default: screenshot.png)
  navigate <url>   Navigate to URL
  cdp-status       Show CDP connection status
  exit, quit, q    Exit the application

CDP commands require Electron to be launched with --remote-debugging-port"
    }
}

impl ConsolePrompt {
    pub fn new(selected_process: ElectronProcess) -> Self {
        let electron = Electron::from_process(&selected_process);
        let cdp_port = electron.cdp_port();

        let mut prompt = Self {
            electron,
            selected_process,
            input: String::new(),
            history: Vec::new(),
            history_index: None,
            logs: Vec::new(),
            should_exit: false,
            last_alive_check: Instant::now(),
            cdp_client: None,
            cdp_status: if cdp_port.is_some() {
                format!("CDP port detected: {} (not connected)", cdp_port.unwrap())
            } else {
                "No CDP port detected".to_string()
            },
        };

        prompt.push_log(format!(
            "Attached to Electron PID {}",
            prompt.electron.pid()
        ));

        if let Some(port) = cdp_port {
            prompt.push_log(format!("CDP debugging available on port {}", port));
        } else {
            prompt.push_log("Tip: Launch Electron with --remote-debugging-port=9222".to_owned());
        }

        prompt.push_log("Type `help` to list commands.".to_owned());
        prompt.push_log("Press Tab to return to the process picker.".to_owned());

        prompt
    }

    pub fn tick(&mut self) -> bool {
        if self.last_alive_check.elapsed() >= ALIVE_CHECK_INTERVAL {
            self.last_alive_check = Instant::now();
            if !self.electron.is_alive() {
                self.push_log(format!(
                    "Electron process {} has been killed",
                    self.electron.pid()
                ));
                return false;
            }
        }
        true
    }

    /// Async tick for CDP operations
    pub async fn tick_async(&mut self) {
        // Update CDP status periodically
        if let Some(client) = &self.cdp_client {
            let state = client.state().await;
            self.cdp_status = match state {
                ConnectionState::Connected => {
                    format!("CDP: Connected (port {})", self.electron.cdp_port().unwrap_or(0))
                }
                ConnectionState::Disconnected => "CDP: Disconnected".to_string(),
                ConnectionState::Connecting => "CDP: Connecting...".to_string(),
            };
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ConsoleAction> {
        match key_event.code {
            KeyCode::Char(c) => {
                self.input.push(c);
                self.history_index = None;
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Enter => {
                self.submit_current_input()?;
            }
            KeyCode::Up => {
                self.navigate_history_up();
            }
            KeyCode::Down => {
                self.navigate_history_down();
            }
            KeyCode::Tab => {
                return Ok(ConsoleAction::BackToPicker);
            }
            KeyCode::Esc => {
                self.should_exit = true;
            }
            _ => {}
        }

        if self.should_exit {
            Ok(ConsoleAction::Quit)
        } else {
            Ok(ConsoleAction::Continue)
        }
    }

    /// Async event handler for CDP operations
    pub async fn handle_key_event_async(
        &mut self,
        key_event: KeyEvent,
    ) -> Result<ConsoleAction> {
        match key_event.code {
            KeyCode::Char(c) => {
                self.input.push(c);
                self.history_index = None;
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Enter => {
                self.submit_current_input_async().await?;
            }
            KeyCode::Up => {
                self.navigate_history_up();
            }
            KeyCode::Down => {
                self.navigate_history_down();
            }
            KeyCode::Tab => {
                // Disconnect CDP before going back
                self.disconnect_cdp().await.ok();
                return Ok(ConsoleAction::BackToPicker);
            }
            KeyCode::Esc => {
                self.should_exit = true;
            }
            _ => {}
        }

        if self.should_exit {
            Ok(ConsoleAction::Quit)
        } else {
            Ok(ConsoleAction::Continue)
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),
                Constraint::Min(8),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(area);

        let process_info = {
            let command = self.selected_process.command_line();
            let cdp_info = if let Some(port) = self.electron.cdp_port() {
                format!("CDP Port: {}\n", port)
            } else {
                String::new()
            };

            if command.is_empty() {
                format!(
                    "PID: {}\nName: {}\n{}",
                    self.selected_process.pid(),
                    self.selected_process.name(),
                    cdp_info
                )
            } else {
                format!(
                    "PID: {}\nName: {}\n{}Cmd: {}",
                    self.selected_process.pid(),
                    self.selected_process.name(),
                    cdp_info,
                    command
                )
            }
        };

        let header = Paragraph::new(process_info)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Attached process"),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(header, chunks[0]);

        let logs = self
            .logs
            .iter()
            .map(|line| ListItem::new(line.as_str()))
            .collect::<Vec<_>>();
        let logs_widget = List::new(logs).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Prompt output - {}", self.cdp_status)),
        );
        frame.render_widget(logs_widget, chunks[1]);

        let input = Paragraph::new(self.input.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command input"),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(input, chunks[2]);

        let footer = Paragraph::new(
            "Enter: submit  •  Up/Down: history  •  Tab: picker  •  Esc/Ctrl+C: quit",
        )
        .block(Block::default().borders(Borders::ALL).title("Shortcuts"))
        .wrap(Wrap { trim: true });
        frame.render_widget(footer, chunks[3]);

        let cursor_x = chunks[2]
            .x
            .saturating_add(1 + self.input.chars().count() as u16)
            .min(chunks[2].right().saturating_sub(2));
        let cursor_y = chunks[2].y.saturating_add(1);
        frame.set_cursor_position((cursor_x, cursor_y));

        if self.should_exit {
            let popup_area = centered_rect(60, 3, area);
            frame.render_widget(Clear, popup_area);
            let popup = Paragraph::new("Exiting...")
                .block(Block::default().borders(Borders::ALL).title("Status"))
                .alignment(Alignment::Center);
            frame.render_widget(popup, popup_area);
        }
    }

    fn submit_current_input(&mut self) -> Result<()> {
        let line = self.input.trim().to_string();

        if !line.is_empty() {
            self.push_history(line.clone());
            self.push_log(format!("> {line}"));
            // For sync submissions, just log that async is needed
            self.push_log("Use async mode for CDP commands".to_owned());
        }

        self.input.clear();
        self.history_index = None;
        Ok(())
    }

    async fn submit_current_input_async(&mut self) -> Result<()> {
        let line = self.input.trim().to_string();

        if !line.is_empty() {
            self.push_history(line.clone());
            self.push_log(format!("> {line}"));
            self.execute_command_async(&line).await?;
        }

        self.input.clear();
        self.history_index = None;
        Ok(())
    }

    async fn execute_command_async(&mut self, line: &str) -> Result<()> {
        match PromptCommand::parse(line) {
            PromptCommand::Help => {
                self.push_multiline(PromptCommand::help_text());
                self.push_log(
                    "Extra shortcuts: Tab to go back to picker, Ctrl+C to quit.".to_owned(),
                );
            }
            PromptCommand::Status => {
                let status = if self.electron.is_alive() {
                    "alive"
                } else {
                    "dead"
                };
                self.push_log(format!("Electron status: {status}"));
            }
            PromptCommand::Pid => {
                self.push_log(format!("Electron PID: {}", self.electron.pid()));
            }
            PromptCommand::History => {
                if self.history.is_empty() {
                    self.push_log("History is empty".to_owned());
                } else {
                    let entries = self
                        .history
                        .iter()
                        .enumerate()
                        .map(|(index, entry)| format!("{:>3}: {}", index + 1, entry))
                        .collect::<Vec<_>>();
                    for entry in entries {
                        self.push_log(entry);
                    }
                }
            }
            PromptCommand::Clear => {
                self.logs.clear();
            }
            PromptCommand::Refresh => match self.electron.refresh() {
                Ok(process) => {
                    self.selected_process = process;
                    self.push_log("Attached process metadata refreshed".to_owned());
                }
                Err(error) => {
                    self.push_log(format!("Refresh failed: {error}"));
                }
            },
            PromptCommand::Exit => {
                self.should_exit = true;
            }
            PromptCommand::Empty => {}
            // CDP commands
            PromptCommand::Connect => {
                self.connect_cdp().await?;
            }
            PromptCommand::Disconnect => {
                self.disconnect_cdp().await?;
            }
            PromptCommand::Evaluate(expr) => {
                self.evaluate_js(&expr).await?;
            }
            PromptCommand::Screenshot(path) => {
                self.take_screenshot(&path).await?;
            }
            PromptCommand::Navigate(url) => {
                self.navigate_to(&url).await?;
            }
            PromptCommand::CdpStatus => {
                self.show_cdp_status().await;
            }
            PromptCommand::Unknown(other) => {
                self.push_log(format!("Unknown command: {other}"));
                self.push_log("Type `help` to list available commands.".to_owned());
            }
        }

        Ok(())
    }

    async fn connect_cdp(&mut self) -> Result<()> {
        if self.cdp_client.is_some() {
            self.push_log("Already connected. Disconnect first to reconnect.".to_owned());
            return Ok(());
        }

        let port = self.electron.cdp_port().unwrap_or(9222);
        self.push_log(format!("Connecting to CDP on port {}...", port));

        let mut client = CdpClient::new(port);

        match client.connect().await {
            Ok(_) => {
                self.cdp_status = format!("CDP: Connected (port {})", port);
                self.push_log(format!("Connected to CDP on port {}", port));

                // Try to get page info
                match client.get_title().await {
                    Ok(title) => self.push_log(format!("Page title: {}", title)),
                    Err(e) => self.push_log(format!("Could not get title: {}", e)),
                }

                self.cdp_client = Some(client);
            }
            Err(e) => {
                self.cdp_status = format!("CDP: Error - {}", e);
                self.push_log(format!("Failed to connect: {}", e));
                self.push_log(
                    "Make sure Electron was launched with --remote-debugging-port".to_owned(),
                );
            }
        }

        Ok(())
    }

    pub async fn disconnect_cdp(&mut self) -> Result<()> {
        if let Some(mut client) = self.cdp_client.take() {
            match client.disconnect().await {
                Ok(_) => self.push_log("Disconnected from CDP".to_owned()),
                Err(e) => self.push_log(format!("Error disconnecting: {}", e)),
            }
        }
        self.cdp_status = "CDP: Disconnected".to_string();
        Ok(())
    }

    async fn evaluate_js(&mut self, expression: &str) -> Result<()> {
        if let Some(client) = &self.cdp_client {
            match client.evaluate(expression).await {
                Ok(result) => {
                    self.push_log(format!("Result: {}", result));
                }
                Err(e) => {
                    self.push_log(format!("Evaluation error: {}", e));
                }
            }
        } else {
            self.push_log("Not connected to CDP. Use 'connect' first.".to_owned());
        }
        Ok(())
    }

    async fn take_screenshot(&mut self, path: &str) -> Result<()> {
        if let Some(client) = &self.cdp_client {
            let path = Path::new(path);
            match client.screenshot(path).await {
                Ok(_) => {
                    self.push_log(format!("Screenshot saved to {}", path.display()));
                }
                Err(e) => {
                    self.push_log(format!("Screenshot error: {}", e));
                }
            }
        } else {
            self.push_log("Not connected to CDP. Use 'connect' first.".to_owned());
        }
        Ok(())
    }

    async fn navigate_to(&mut self, url: &str) -> Result<()> {
        if let Some(client) = &self.cdp_client {
            match client.navigate(url).await {
                Ok(_) => {
                    self.push_log(format!("Navigated to {}", url));
                }
                Err(e) => {
                    self.push_log(format!("Navigation error: {}", e));
                }
            }
        } else {
            self.push_log("Not connected to CDP. Use 'connect' first.".to_owned());
        }
        Ok(())
    }

    async fn show_cdp_status(&mut self) {
        let status = if let Some(client) = &self.cdp_client {
            match client.state().await {
                ConnectionState::Connected => {
                    format!("CDP: Connected on port {:?}", self.electron.cdp_port())
                }
                ConnectionState::Connecting => "CDP: Connecting...".to_string(),
                ConnectionState::Disconnected => "CDP: Disconnected".to_string(),
            }
        } else {
            format!(
                "CDP: Not connected. Port available: {:?}",
                self.electron.cdp_port()
            )
        };
        self.push_log(status);
    }

    fn navigate_history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        let next_index = match self.history_index {
            None => self.history.len().saturating_sub(1),
            Some(0) => 0,
            Some(index) => index.saturating_sub(1),
        };

        self.history_index = Some(next_index);
        self.input = self.history[next_index].clone();
    }

    fn navigate_history_down(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {}
            Some(index) if index + 1 >= self.history.len() => {
                self.history_index = None;
                self.input.clear();
            }
            Some(index) => {
                let next_index = index + 1;
                self.history_index = Some(next_index);
                self.input = self.history[next_index].clone();
            }
        }
    }

    fn push_history(&mut self, line: String) {
        self.history.push(line);
        if self.history.len() > MAX_HISTORY {
            let overflow = self.history.len() - MAX_HISTORY;
            self.history.drain(0..overflow);
        }
    }

    pub fn push_log(&mut self, line: String) {
        self.logs.push(line);
        if self.logs.len() > LOG_LIMIT {
            let overflow = self.logs.len() - LOG_LIMIT;
            self.logs.drain(0..overflow);
        }
    }

    fn push_multiline(&mut self, text: &str) {
        for line in text.lines() {
            self.push_log(line.to_owned());
        }
    }
}

fn centered_rect(width_percent: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}
