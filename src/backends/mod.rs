mod claude;
mod codex;

use crate::cli::{BackendKind, StreamArgs};
use anyhow::{anyhow, Result};
use std::process::Command;

pub use claude::ClaudeBackend;
pub use codex::CodexBackend;

pub enum Event {
    /// A chunk of model output text. May contain partial words, newlines, etc.
    TextDelta(String),
    /// The model is done producing text for this turn.
    MessageStop,
}

pub trait Backend {
    /// Build the subprocess that produces a streaming response.
    fn build_command(&self, args: &StreamArgs) -> Command;

    /// Parse one line from the backend's stdout into zero or more normalized events.
    fn parse_line(&mut self, line: &str) -> Vec<Event>;

    /// Human-readable name (for error messages).
    fn name(&self) -> &'static str;
}

/// Resolve the user's `--backend` choice into a concrete backend. For `auto`,
/// pick whichever CLI is on PATH (claude is preferred when both exist).
pub fn resolve(kind: BackendKind) -> Result<Box<dyn Backend>> {
    match kind {
        BackendKind::Claude => Ok(Box::new(ClaudeBackend::new())),
        BackendKind::Codex => Ok(Box::new(CodexBackend::new())),
        BackendKind::Auto => {
            if which("claude") {
                Ok(Box::new(ClaudeBackend::new()))
            } else if which("codex") {
                Ok(Box::new(CodexBackend::new()))
            } else {
                Err(anyhow!(
                    "no backend CLI found on PATH (looked for: claude, codex)"
                ))
            }
        }
    }
}

fn which(name: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&paths).any(|p| {
        let candidate = p.join(name);
        candidate.is_file()
    })
}
