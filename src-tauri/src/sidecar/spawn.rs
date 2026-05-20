// SidecarSpec/SidecarHandle/spawn used by phase-05+ handlers; suppress dead_code until then.
#![allow(dead_code)]

use std::{path::PathBuf, process::Stdio, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::{mpsc, oneshot},
    time::timeout,
};
use tracing::{debug, warn};

use crate::domain::error::AppError;
use super::progress::{parse_line, SidecarMsg};

/// How the sidecar's stdout lines should be interpreted.
#[derive(Debug, Clone)]
pub enum ProgressParser {
    /// Each line is either a JSON object `{percent, message}` or plain text.
    JsonLines,
    /// Pass every line through as a raw log message.
    Stderr,
}

/// Parameters for spawning a sidecar process.
pub struct SidecarSpec {
    pub operation_id: String,
    pub bin: String,
    pub args: Vec<String>,
    pub workdir: PathBuf,
    /// Wall-clock timeout; defaults to 5 minutes.
    pub timeout_duration: Duration,
    pub progress_parser: ProgressParser,
}

impl Default for SidecarSpec {
    fn default() -> Self {
        SidecarSpec {
            operation_id: String::new(),
            bin: String::new(),
            args: Vec::new(),
            workdir: PathBuf::from("."),
            timeout_duration: Duration::from_secs(300),
            progress_parser: ProgressParser::JsonLines,
        }
    }
}

/// Live handle to a spawned sidecar process.
pub struct SidecarHandle {
    pub op_id: String,
    /// Receive parsed messages from the sidecar's stdout.
    pub stdout_rx: mpsc::Receiver<SidecarMsg>,
    /// Fires once the process exits; carries the OS exit code.
    pub exit_rx: oneshot::Receiver<i32>,
}

/// Spawn the sidecar process described by `spec`.
///
/// Stdout is captured and read line-by-line in a dedicated tokio task.
/// The process is killed automatically if `timeout_duration` elapses.
/// Returns `Err(AppError::SidecarFailed)` if the binary cannot be launched.
pub async fn spawn(spec: SidecarSpec) -> Result<SidecarHandle, AppError> {
    debug!(op_id = %spec.operation_id, bin = %spec.bin, "spawning sidecar");

    let mut cmd = Command::new(&spec.bin);
    cmd.args(&spec.args)
        .current_dir(&spec.workdir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let mut child = cmd.spawn().map_err(|e| {
        AppError::SidecarFailed(format!("failed to spawn '{}': {}", spec.bin, e))
    })?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::SidecarFailed("stdout not captured".into()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::SidecarFailed("stderr not captured".into()))?;

    // Capacity 512: stderr-merged logs from chatty tools (AssetRipper, IL2CPP
    // dumper) can briefly exceed the original 64 before consumers drain.
    let (msg_tx, msg_rx) = mpsc::channel::<SidecarMsg>(512);
    let (exit_tx, exit_rx) = oneshot::channel::<i32>();

    let op_id = spec.operation_id.clone();
    let parser = spec.progress_parser.clone();
    let deadline = spec.timeout_duration;

    // Stderr reader task — forwards every line as a tagged log message so the
    // diagnostic narrative isn't lost when sidecars write errors to stderr.
    let stderr_tx = msg_tx.clone();
    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if stderr_tx
                .send(SidecarMsg::Log(format!("[stderr] {line}")))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // Stdout reader task — one task per process.
    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let result = timeout(deadline, async {
            while let Ok(Some(line)) = lines.next_line().await {
                let msg = match parser {
                    ProgressParser::JsonLines => parse_line(&line),
                    ProgressParser::Stderr => SidecarMsg::Log(line),
                };
                if msg_tx.send(msg).await.is_err() {
                    break; // Receiver dropped — no one is listening
                }
            }
            child.wait().await
        })
        .await;

        let code = match result {
            Ok(Ok(status)) => status.code().unwrap_or(-1),
            Ok(Err(e)) => {
                warn!(op_id = %op_id, "sidecar wait error: {e}");
                -1
            }
            Err(_) => {
                warn!(op_id = %op_id, "sidecar timed out, killing");
                let _ = child.kill().await;
                -2 // sentinel: timeout
            }
        };

        let _ = msg_tx.send(if code == 0 {
            SidecarMsg::Done
        } else {
            SidecarMsg::Error(format!("exit code {code}"))
        }).await;

        let _ = exit_tx.send(code);
    });

    Ok(SidecarHandle {
        op_id: spec.operation_id,
        stdout_rx: msg_rx,
        exit_rx,
    })
}
