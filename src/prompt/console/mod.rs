use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::electron::{Electron, ElectronProcess};

const MAX_HISTORY: usize = 100;
const LOG_LIMIT: usize = 200;
const ALIVE_CHECK_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Debug, Clone)]
pub struct ConsolePrompt {
    electron: Electron,
    selected_process: ElectronProcess,
    input: String,
    history: Vec<String>,
    history_index: Option<usize>,
    logs: Vec<String>,
    should_exit: bool,
    last_alive_check: Instant,
}

#[derive(Debug, Clone)]
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
    Unknown(String),
}

impl PromptCommand {
    fn parse(input: &str) -> Self {
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
            "refresh" | "reload" => Self::Refresh,
            "exit" | "quit" | "q" => Self::Exit,
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
  exit, quit, q    Exit the application"
    }
}

impl ConsolePrompt {
    pub fn new(selected_process: ElectronProcess) -> Self {
        let electron = Electron::from_process(&selected_process);
        let mut prompt = Self {
            electron,
            selected_process,
            input: String::new(),
            history: Vec::new(),
            history_index: None,
            logs: Vec::new(),
            should_exit: false,
            last_alive_check: Instant::now(),
        };

        prompt.push_log(format!(
            "Attached to Electron PID {}",
            prompt.electron.pid()
        ));
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

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Min(8),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(area);

        let process_info = {
            let command = self.selected_process.command_line();
            if command.is_empty() {
                format!(
                    "PID: {}\nName: {}",
                    self.selected_process.pid(),
                    self.selected_process.name()
                )
            } else {
                format!(
                    "PID: {}\nName: {}\nCmd: {}",
                    self.selected_process.pid(),
                    self.selected_process.name(),
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
                .title("Prompt output"),
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
            self.execute_command(&line)?;
        }

        self.input.clear();
        self.history_index = None;
        Ok(())
    }

    fn execute_command(&mut self, line: &str) -> Result<()> {
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
            PromptCommand::Unknown(other) => {
                self.push_log(format!("Unknown command: {other}"));
                self.push_log("Type `help` to list available commands.".to_owned());
            }
        }

        Ok(())
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
