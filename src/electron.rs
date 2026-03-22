use anyhow::{Result, bail};
use sysinfo::{Pid, ProcessesToUpdate, System};

const ELECTRON_PROCESS_NAME: &str = "Electron";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronProcess {
    pid: u32,
    name: String,
    command: Vec<String>,
}

impl ElectronProcess {
    pub fn new(pid: u32, name: String, command: Vec<String>) -> Self {
        Self { pid, name, command }
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn command_line(&self) -> String {
        if self.command.is_empty() {
            String::new()
        } else {
            self.command.join(" ")
        }
    }

    pub fn is_electron_name(name: &str) -> bool {
        name == ELECTRON_PROCESS_NAME
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Electron {
    pid: u32,
}

impl Electron {
    pub fn from_process(process: &ElectronProcess) -> Self {
        Self { pid: process.pid() }
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn is_alive(&self) -> bool {
        let mut system = System::new_all();
        system.refresh_processes(ProcessesToUpdate::All, true);

        system
            .process(Pid::from_u32(self.pid))
            .is_some_and(|process| ElectronProcess::is_electron_name(&process_name(process)))
    }

    pub fn refresh(&self) -> Result<ElectronProcess> {
        let mut system = System::new_all();
        system.refresh_processes(ProcessesToUpdate::All, true);

        let Some(process) = system.process(Pid::from_u32(self.pid)) else {
            bail!("electron process {} is no longer running", self.pid);
        };

        let name = process_name(process);
        if !ElectronProcess::is_electron_name(&name) {
            bail!("process {} is no longer an Electron process", self.pid);
        }

        Ok(ElectronProcess::new(
            process.pid().as_u32(),
            name,
            process_command(process),
        ))
    }
}

pub fn list_processes() -> Result<Vec<ElectronProcess>> {
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);

    let mut processes = system
        .processes()
        .values()
        .filter_map(|process| {
            let name = process_name(process);

            if !ElectronProcess::is_electron_name(&name) {
                return None;
            }

            Some(ElectronProcess::new(
                process.pid().as_u32(),
                name,
                process_command(process),
            ))
        })
        .collect::<Vec<_>>();

    processes.sort_by_key(ElectronProcess::pid);

    Ok(processes)
}

fn process_name(process: &sysinfo::Process) -> String {
    process.name().to_string_lossy().into_owned()
}

fn process_command(process: &sysinfo::Process) -> Vec<String> {
    process
        .cmd()
        .iter()
        .map(|part| part.to_string_lossy().into_owned())
        .collect()
}
