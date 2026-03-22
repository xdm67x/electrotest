use crate::{cmd::Command, config};

pub struct RunCommand;

impl Command for RunCommand {
    fn execute(self) -> anyhow::Result<()> {
        let config = config::parse()?;
        println!("Package manager: {:?}", config.package_manager);
        Ok(())
    }
}
