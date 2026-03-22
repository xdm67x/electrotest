use anyhow::{Result, bail};
use sysinfo::{Pid, ProcessesToUpdate, System};

const ELECTRON_PROCESS_NAME: &str = "Electron";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Electron {
    pid: u32,
}

impl Electron {
    pub fn attach(pid: u32) -> Result<Self> {
        let mut system = System::new_all();
        system.refresh_processes(ProcessesToUpdate::All, true);

        match system.process(Pid::from_u32(pid)) {
            Some(process) if process.name() == ELECTRON_PROCESS_NAME => Ok(Self {
                pid: process.pid().as_u32(),
            }),
            _ => bail!("no electron process found with that pid: {pid}"),
        }
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn is_alive(&self) -> bool {
        let mut system = System::new_all();
        system.refresh_processes(ProcessesToUpdate::All, true);

        system
            .process(Pid::from_u32(self.pid))
            .is_some_and(|process| process.name() == ELECTRON_PROCESS_NAME)
    }
}
