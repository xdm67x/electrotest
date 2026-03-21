pub struct PlaywrightEngine {
    worker: crate::engine::process::WorkerProcess,
}

impl PlaywrightEngine {
    pub fn new(worker: crate::engine::process::WorkerProcess) -> Self {
        Self { worker }
    }

    pub fn worker(&mut self) -> &mut crate::engine::process::WorkerProcess {
        &mut self.worker
    }
}
