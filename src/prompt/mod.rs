use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::electron::{Electron, ElectronProcess, list_processes};

const POLL_INTERVAL: Duration = Duration::from_millis(200);
const MAX_HISTORY: usize = 100;
const LOG_LIMIT: usize = 200;

pub type App = Prompt;

#[derive(Debug, Clone)]
pub struct Prompt {
    mode: AppMode,
}

#[derive(Debug, Clone)]
enum AppMode {
    Picker(ProcessPicker),
    Console(ConsolePrompt),
}

#[derive(Debug, Clone)]
struct ProcessPicker {
    processes: Vec<ElectronProcess>,
    selected: usize,
    status: String,
}

#[derive(Debug, Clone)]
struct ConsolePrompt {
    electron: Electron,
    selected_process: ElectronProcess,
    input: String,
    history: Vec<String>,
    history_index: Option<usize>,
    logs: Vec<String>,
    should_exit: bool,
}

#[derive(Debug, Clone)]
enum PickerAction {
    Continue,
    OpenProcess(ElectronProcess),
    Quit,
}

#[derive(Debug, Clone)]
enum ConsoleAction {
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

impl Default for Prompt {
    fn default() -> Self {
        match Self::new() {
            Ok(prompt) => prompt,
            Err(_) => Self {
                mode: AppMode::Picker(ProcessPicker::empty(
                    "Unable to initialize process picker".to_owned(),
                )),
            },
        }
    }
}

impl Prompt {
    pub fn new() -> Result<Self> {
        Ok(Self {
            mode: AppMode::Picker(ProcessPicker::new()?),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = TerminalSession::enter()?;

        loop {
            terminal.terminal.draw(|frame| self.render(frame))?;

            if !event::poll(POLL_INTERVAL)? {
                if let AppMode::Console(console) = &mut self.mode {
                    if !console.electron.is_alive() {
                        console.push_log(format!(
                            "Electron process {} has been killed",
                            console.electron.pid()
                        ));
                        break;
                    }
                }
                continue;
            }

            let event = event::read()?;
            if !self.handle_event(event)? {
                break;
            }
        }

        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        match &mut self.mode {
            AppMode::Picker(picker) => picker.render(frame),
            AppMode::Console(console) => console.render(frame),
        }
    }

    fn handle_event(&mut self, event: Event) -> Result<bool> {
        let Event::Key(key_event) = event else {
            return Ok(true);
        };

        if key_event.kind != KeyEventKind::Press {
            return Ok(true);
        }

        if key_event.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key_event.code, KeyCode::Char('c') | KeyCode::Char('C'))
        {
            return Ok(false);
        }

        match &mut self.mode {
            AppMode::Picker(picker) => match picker.handle_key_event(key_event)? {
                PickerAction::Continue => Ok(true),
                PickerAction::Quit => Ok(false),
                PickerAction::OpenProcess(process) => {
                    self.mode = AppMode::Console(ConsolePrompt::new(process));
                    Ok(true)
                }
            },
            AppMode::Console(console) => match console.handle_key_event(key_event)? {
                ConsoleAction::Continue => Ok(true),
                ConsoleAction::Quit => Ok(false),
                ConsoleAction::BackToPicker => {
                    self.mode = AppMode::Picker(ProcessPicker::new()?);
                    Ok(true)
                }
            },
        }
    }
}

impl ProcessPicker {
    fn new() -> Result<Self> {
        let mut picker = Self {
            processes: Vec::new(),
            selected: 0,
            status: String::new(),
        };
        picker.refresh()?;
        Ok(picker)
    }

    fn empty(status: String) -> Self {
        Self {
            processes: Vec::new(),
            selected: 0,
            status,
        }
    }

    fn refresh(&mut self) -> Result<()> {
        self.processes = list_processes()?;
        if self.processes.is_empty() {
            self.selected = 0;
            self.status =
                "No running Electron process found. Press r to refresh or q to quit.".to_owned();
        } else {
            if self.selected >= self.processes.len() {
                self.selected = self.processes.len() - 1;
            }
            self.status = format!(
                "{} Electron process(es) found. Use ↑/↓ to select, Enter to attach.",
                self.processes.len()
            );
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<PickerAction> {
        match key_event.code {
            KeyCode::Up => {
                if !self.processes.is_empty() {
                    self.selected = self.selected.saturating_sub(1);
                }
                Ok(PickerAction::Continue)
            }
            KeyCode::Down => {
                if !self.processes.is_empty() && self.selected + 1 < self.processes.len() {
                    self.selected += 1;
                }
                Ok(PickerAction::Continue)
            }
            KeyCode::Enter => {
                if let Some(process) = self.processes.get(self.selected).cloned() {
                    Ok(PickerAction::OpenProcess(process))
                } else {
                    Ok(PickerAction::Continue)
                }
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.refresh()?;
                Ok(PickerAction::Continue)
            }
            KeyCode::Char('q') | KeyCode::Esc => Ok(PickerAction::Quit),
            _ => Ok(PickerAction::Continue),
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(3),
            ])
            .split(area);

        let title = Paragraph::new("Electron Process Picker")
            .block(Block::default().borders(Borders::ALL).title("Electrotest"));
        frame.render_widget(title, chunks[0]);

        let items = if self.processes.is_empty() {
            vec![ListItem::new("No Electron process is currently running")]
        } else {
            self.processes
                .iter()
                .map(|process| {
                    let command = process.command_line();
                    let detail = if command.is_empty() {
                        format!("PID {:<8} {}", process.pid(), process.name())
                    } else {
                        format!("PID {:<8} {} — {}", process.pid(), process.name(), command)
                    };

                    ListItem::new(detail)
                })
                .collect()
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Running Electron processes"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">> ");

        let mut state = ListState::default();
        if !self.processes.is_empty() {
            state.select(Some(self.selected));
        }
        frame.render_stateful_widget(list, chunks[1], &mut state);

        let footer = Paragraph::new(self.status.as_str())
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .wrap(Wrap { trim: true });
        frame.render_widget(footer, chunks[2]);
    }
}

impl ConsolePrompt {
    fn new(selected_process: ElectronProcess) -> Self {
        let electron = Electron::from_process(&selected_process);
        let mut prompt = Self {
            electron,
            selected_process,
            input: String::new(),
            history: Vec::new(),
            history_index: None,
            logs: Vec::new(),
            should_exit: false,
        };

        prompt.push_log(format!(
            "Attached to Electron PID {}",
            prompt.electron.pid()
        ));
        prompt.push_log("Type `help` to list commands.".to_owned());
        prompt.push_log("Press Tab to return to the process picker.".to_owned());

        prompt
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ConsoleAction> {
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

    fn push_log(&mut self, line: String) {
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

    fn render(&mut self, frame: &mut Frame) {
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
}

struct TerminalSession {
    terminal: ratatui::Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalSession {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = ratatui::Terminal::new(backend)?;

        Ok(Self { terminal })
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
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
