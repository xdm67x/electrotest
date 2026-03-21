use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio::process::{ChildStdin, ChildStdout, Command};
use tokio_util::codec::{FramedRead, LinesCodec, LinesCodecError};

const SHUTDOWN_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(100);

pub struct WorkerProcess {
    child: tokio::process::Child,
    stdin: ChildStdin,
    stdout: FramedRead<ChildStdout, LinesCodec>,
}

#[derive(Debug, thiserror::Error)]
pub enum WorkerProcessError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("worker stdout closed before a response was received")]
    UnexpectedEof,
    #[error("worker emitted malformed response: {0}")]
    MalformedResponse(String),
    #[error(transparent)]
    Lines(#[from] LinesCodecError),
}

impl WorkerProcess {
    pub fn from_command(mut command: Command) -> Result<Self, WorkerProcessError> {
        command
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit());

        let mut child = command.spawn()?;
        let stdin = child.stdin.take().ok_or(WorkerProcessError::UnexpectedEof)?;
        let stdout = child.stdout.take().ok_or(WorkerProcessError::UnexpectedEof)?;

        Ok(Self {
            child,
            stdin,
            stdout: FramedRead::new(stdout, LinesCodec::new()),
        })
    }

    pub async fn request(
        &mut self,
        request: &crate::engine::protocol::Request,
    ) -> Result<crate::engine::protocol::Response, WorkerProcessError> {
        self.send(request).await?;
        self.read_response().await
    }

    pub async fn send(
        &mut self,
        request: &crate::engine::protocol::Request,
    ) -> Result<(), WorkerProcessError> {
        let payload = serde_json::to_string(request)?;
        self.stdin.write_all(payload.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }

    pub async fn read_response(
        &mut self,
    ) -> Result<crate::engine::protocol::Response, WorkerProcessError> {
        let Some(line) = self.stdout.next().await.transpose()? else {
            return Err(WorkerProcessError::UnexpectedEof);
        };

        serde_json::from_str(&line).map_err(|_| WorkerProcessError::MalformedResponse(line))
    }

    pub async fn shutdown(&mut self) -> Result<(), WorkerProcessError> {
        self.stdin.shutdown().await?;

        if tokio::time::timeout(SHUTDOWN_TIMEOUT, self.child.wait())
            .await
            .is_err()
        {
            self.child.kill().await?;
            let _ = self.child.wait().await?;
        }

        Ok(())
    }
}
