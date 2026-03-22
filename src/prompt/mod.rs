mod app;
mod console;
mod picker;
mod ui;

pub use app::App;

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event};

use self::ui::TerminalSession;

const POLL_INTERVAL: Duration = Duration::from_millis(200);

impl App {
    pub fn run(&mut self) -> Result<()> {
        let mut terminal = TerminalSession::enter()?;

        loop {
            terminal.terminal_mut().draw(|frame| self.render(frame))?;

            if !event::poll(POLL_INTERVAL)? {
                if !self.tick() {
                    break;
                }
                continue;
            }

            let event = event::read()?;
            if !self.handle_runtime_event(event)? {
                break;
            }
        }

        Ok(())
    }

    fn handle_runtime_event(&mut self, event: Event) -> Result<bool> {
        self.handle_event(event)
    }
}
