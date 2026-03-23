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
        // Create tokio runtime BEFORE entering raw mode
        // This avoids "Device not configured" error on macOS
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        let mut terminal = TerminalSession::enter()?;

        loop {
            terminal.terminal_mut().draw(|frame| self.render(frame))?;

            if !event::poll(POLL_INTERVAL)? {
                // Run async tick operations
                if !rt.block_on(self.tick_async()) {
                    break;
                }
                continue;
            }

            let event = event::read()?;
            if !rt.block_on(self.handle_event_async(event))? {
                break;
            }
        }

        Ok(())
    }
}
