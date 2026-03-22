mod command;

use self::command::PromptCommand;
use crate::electron::Electron;
use anyhow::Result;
use crossterm::{
    ExecutableCommand, cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{self, ClearType},
};
use std::io::{self, Write};
use std::time::Duration;

const PROMPT_PREFIX: &str = "> ";
const POLL_INTERVAL: Duration = Duration::from_millis(200);
const MAX_HISTORY: usize = 100;

#[derive(Debug, Clone)]
pub struct Prompt {
    electron: Electron,
    input: String,
    history: Vec<String>,
    history_index: Option<usize>,
    should_exit: bool,
}

impl Prompt {
    pub fn new(electron: Electron) -> Self {
        Self {
            electron,
            input: String::new(),
            history: Vec::new(),
            history_index: None,
            should_exit: false,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = TerminalSession::enter()?;

        self.print_welcome(&mut terminal.stdout)?;

        while !self.should_exit {
            if !self.electron.is_alive() {
                self.clear_current_line(&mut terminal.stdout)?;
                writeln!(terminal.stdout, "Electron process has been killed")?;
                terminal.stdout.flush()?;
                break;
            }

            self.render_prompt(&mut terminal.stdout)?;

            if !event::poll(POLL_INTERVAL)? {
                continue;
            }

            let event = event::read()?;
            self.handle_event(event, &mut terminal.stdout)?;
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event, stdout: &mut io::Stdout) -> Result<()> {
        let Event::Key(key_event) = event else {
            return Ok(());
        };

        if key_event.kind != KeyEventKind::Press {
            return Ok(());
        }

        if key_event.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key_event.code, KeyCode::Char('c') | KeyCode::Char('C'))
        {
            self.clear_current_line(stdout)?;
            writeln!(stdout, "Interrupted")?;
            stdout.flush()?;
            self.should_exit = true;
            return Ok(());
        }

        self.handle_key_event(key_event, stdout)
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, stdout: &mut io::Stdout) -> Result<()> {
        match key_event.code {
            KeyCode::Char(c) => {
                self.input.push(c);
                self.history_index = None;
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Enter => {
                self.submit_current_input(stdout)?;
            }
            KeyCode::Up => {
                self.navigate_history_up();
            }
            KeyCode::Down => {
                self.navigate_history_down();
            }
            KeyCode::Esc => {
                self.clear_current_line(stdout)?;
                stdout.flush()?;
                self.should_exit = true;
            }
            KeyCode::Tab => {}
            _ => {}
        }

        Ok(())
    }

    fn submit_current_input(&mut self, stdout: &mut io::Stdout) -> Result<()> {
        self.clear_current_line(stdout)?;

        let line = self.input.trim().to_string();

        if !line.is_empty() {
            self.push_history(line.clone());
            writeln!(stdout, "{PROMPT_PREFIX}{line}")?;
            self.execute_command(&line, stdout)?;
        }

        self.input.clear();
        self.history_index = None;
        stdout.flush()?;

        Ok(())
    }

    fn execute_command(&mut self, line: &str, stdout: &mut io::Stdout) -> Result<()> {
        match PromptCommand::parse(line) {
            PromptCommand::Help => {
                writeln!(stdout, "{}", PromptCommand::help_text())?;
            }
            PromptCommand::Status => {
                let status = if self.electron.is_alive() {
                    "alive"
                } else {
                    "dead"
                };
                writeln!(stdout, "Electron status: {status}")?;
            }
            PromptCommand::Pid => {
                writeln!(stdout, "Electron PID: {}", self.electron.pid())?;
            }
            PromptCommand::History => {
                if self.history.is_empty() {
                    writeln!(stdout, "History is empty")?;
                } else {
                    for (index, entry) in self.history.iter().enumerate() {
                        writeln!(stdout, "{:>3}: {}", index + 1, entry)?;
                    }
                }
            }
            PromptCommand::Clear => {
                stdout.execute(terminal::Clear(ClearType::All))?;
                stdout.execute(cursor::MoveTo(0, 0))?;
            }
            PromptCommand::Exit => {
                self.should_exit = true;
            }
            PromptCommand::Empty => {}
            PromptCommand::Unknown(other) => {
                writeln!(stdout, "Unknown command: {other}")?;
                writeln!(stdout, "Type `help` to list available commands.")?;
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

    fn print_welcome(&self, stdout: &mut io::Stdout) -> Result<()> {
        writeln!(stdout, "Attached to Electron PID {}", self.electron.pid())?;
        writeln!(stdout, "Type `help` to list available commands.")?;
        writeln!(stdout, "Use Up/Down for history, Ctrl+C or `exit` to quit.")?;
        stdout.flush()?;
        Ok(())
    }

    fn render_prompt(&self, stdout: &mut io::Stdout) -> Result<()> {
        stdout.execute(cursor::MoveToColumn(0))?;
        stdout.execute(terminal::Clear(ClearType::CurrentLine))?;
        write!(stdout, "{PROMPT_PREFIX}{}", self.input)?;
        stdout.flush()?;
        Ok(())
    }

    fn clear_current_line(&self, stdout: &mut io::Stdout) -> Result<()> {
        stdout.execute(cursor::MoveToColumn(0))?;
        stdout.execute(terminal::Clear(ClearType::CurrentLine))?;
        Ok(())
    }
}

struct TerminalSession {
    stdout: io::Stdout,
}

impl TerminalSession {
    fn enter() -> Result<Self> {
        terminal::enable_raw_mode()?;

        let mut stdout = io::stdout();
        stdout.execute(cursor::Show)?;

        Ok(Self { stdout })
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.stdout.execute(cursor::MoveToColumn(0));
        let _ = self.stdout.execute(terminal::Clear(ClearType::CurrentLine));
        let _ = terminal::disable_raw_mode();
    }
}
