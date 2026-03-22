use crate::{cmd::Command, config};

pub struct InitCommand;

impl Command for InitCommand {
    fn execute(self) -> anyhow::Result<()> {
        config::create()?;
        Ok(())
    }
}
