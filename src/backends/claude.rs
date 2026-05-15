use super::{Backend, Event};
use crate::cli::StreamArgs;
use serde::Deserialize;
use std::process::Command;

const SYSTEM_PROMPT: &str = "You are a concise terminal assistant. \
Answer in 1-3 short paragraphs. \
You may use **bold** for emphasis, `inline code` for commands or identifiers, \
and ```fenced code blocks``` for multi-line code. \
Skip headings unless the answer benefits from them. No filler.";

pub struct ClaudeBackend;

impl ClaudeBackend {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct Envelope {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    event: Option<StreamEvent>,
}

#[derive(Deserialize)]
struct StreamEvent {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    delta: Option<Delta>,
}

#[derive(Deserialize)]
struct Delta {
    #[serde(default)]
    text: Option<String>,
}

impl Backend for ClaudeBackend {
    fn build_command(&self, args: &StreamArgs) -> Command {
        let mut cmd = Command::new("claude");
        cmd.arg("-p")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            .arg("--include-partial-messages");

        if let Some(sid) = &args.session_id {
            if args.resume {
                cmd.arg("--resume").arg(sid);
            } else {
                cmd.arg("--session-id").arg(sid);
            }
        }

        cmd.arg("--append-system-prompt").arg(SYSTEM_PROMPT);
        cmd.arg(&args.query);
        cmd
    }

    fn parse_line(&mut self, line: &str) -> Vec<Event> {
        let env: Envelope = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        };
        if env.kind != "stream_event" {
            return Vec::new();
        }
        let Some(evt) = env.event else {
            return Vec::new();
        };
        match evt.kind.as_str() {
            "content_block_delta" => {
                if let Some(Delta { text: Some(t) }) = evt.delta {
                    if !t.is_empty() {
                        return vec![Event::TextDelta(t)];
                    }
                }
                Vec::new()
            }
            "message_stop" => vec![Event::MessageStop],
            _ => Vec::new(),
        }
    }

    fn name(&self) -> &'static str {
        "claude"
    }
}
