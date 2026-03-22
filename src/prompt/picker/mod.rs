use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::electron::{ElectronProcess, list_processes};

#[derive(Debug, Clone)]
pub struct ProcessPicker {
    processes: Vec<ElectronProcess>,
    selected: usize,
    status: String,
}

#[derive(Debug, Clone)]
pub enum PickerAction {
    Continue,
    OpenProcess(ElectronProcess),
    Quit,
}

impl ProcessPicker {
    pub fn new() -> Result<Self> {
        let mut picker = Self {
            processes: Vec::new(),
            selected: 0,
            status: String::new(),
        };
        picker.refresh()?;
        Ok(picker)
    }

    pub fn empty(status: String) -> Self {
        Self {
            processes: Vec::new(),
            selected: 0,
            status,
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
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

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<PickerAction> {
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

    pub fn render(&mut self, frame: &mut Frame) {
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
