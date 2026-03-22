use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::electron::ElectronProcess;
use crate::prompt::console::{ConsoleAction, ConsolePrompt};
use crate::prompt::picker::{PickerAction, ProcessPicker};

#[derive(Debug, Clone)]
pub struct App {
    mode: AppMode,
}

#[derive(Debug, Clone)]
enum AppMode {
    Picker(ProcessPicker),
    Console(ConsolePrompt),
}

impl Default for App {
    fn default() -> Self {
        match Self::new() {
            Ok(app) => app,
            Err(_) => Self {
                mode: AppMode::Picker(ProcessPicker::empty(
                    "Unable to initialize process picker".to_owned(),
                )),
            },
        }
    }
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            mode: AppMode::Picker(ProcessPicker::new()?),
        })
    }

    pub fn tick(&mut self) -> bool {
        match &mut self.mode {
            AppMode::Picker(_) => true,
            AppMode::Console(console) => {
                if console.is_alive() {
                    true
                } else {
                    console.push_log(format!(
                        "Electron process {} has been killed",
                        console.electron_pid()
                    ));
                    false
                }
            }
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Result<bool> {
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
                    self.open_process(process);
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

    pub fn render(&mut self, frame: &mut ratatui::Frame) {
        match &mut self.mode {
            AppMode::Picker(picker) => picker.render(frame),
            AppMode::Console(console) => console.render(frame),
        }
    }

    fn open_process(&mut self, process: ElectronProcess) {
        self.mode = AppMode::Console(ConsolePrompt::new(process));
    }
}
