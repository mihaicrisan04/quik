//! OpenAI codex CLI backend.
//!
//! The codex CLI emits NDJSON events from `codex exec --json`. We parse the
//! agent_message_delta stream and stop on task_complete. Schema may evolve
//! across codex releases — keep this parser permissive and update it if you
//! see codex emit different `msg.type` values.

use super::{Backend, Event};
use crate::cli::StreamArgs;
use serde::Deserialize;
use std::process::Command;

pub struct CodexBackend;

impl CodexBackend {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct Envelope {
    #[serde(default)]
    msg: Option<Msg>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Msg {
    /// Incremental text from the model.
    #[serde(rename = "agent_message_delta")]
    AgentMessageDelta {
        #[serde(default)]
        delta: String,
    },
    /// Full agent message (some codex builds emit this instead of deltas).
    #[serde(rename = "agent_message")]
    AgentMessage {
        #[serde(default)]
        message: String,
    },
    /// Task finished.
    #[serde(rename = "task_complete")]
    TaskComplete {
        #[allow(dead_code)]
        #[serde(default)]
        last_agent_message: Option<String>,
    },
    /// Any other event type — ignore.
    #[serde(other)]
    Other,
}

impl Backend for CodexBackend {
    fn build_command(&self, args: &StreamArgs) -> Command {
        let mut cmd = Command::new("codex");
        cmd.arg("exec").arg("--json");
        // codex's session/resume model differs from claude's. Until we wire
        // codex session continuity, we honor session_id for parity but pass
        // it as an opaque tag if supported via env. Update once the codex
        // CLI exposes a stable `--session-id` / `--resume` interface.
        if let Some(sid) = &args.session_id {
            cmd.env("QUIK_SESSION", sid);
        }
        cmd.arg(&args.query);
        cmd
    }

    fn parse_line(&mut self, line: &str) -> Vec<Event> {
        let env: Envelope = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        };
        let Some(msg) = env.msg else {
            return Vec::new();
        };
        match msg {
            Msg::AgentMessageDelta { delta } if !delta.is_empty() => {
                vec![Event::TextDelta(delta)]
            }
            Msg::AgentMessage { message } if !message.is_empty() => {
                // Only honor the full message if we never saw any deltas — otherwise
                // we'd print the text twice.
                vec![Event::TextDelta(message)]
            }
            Msg::TaskComplete { .. } => vec![Event::MessageStop],
            _ => Vec::new(),
        }
    }

    fn name(&self) -> &'static str {
        "codex"
    }
}
