use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "quik",
    version,
    about = "Inline AI prompt for your terminal",
    long_about = None,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Stream a one-shot prompt through a backend CLI and render the response inline.
    Stream(StreamArgs),
}

#[derive(Parser, Debug)]
pub struct StreamArgs {
    /// The prompt to send to the model.
    pub query: String,

    /// Which backend CLI to invoke.
    #[arg(short, long, value_enum, default_value_t = BackendKind::Auto)]
    pub backend: BackendKind,

    /// Session identifier for backends that support it (claude --session-id / --resume).
    #[arg(long)]
    pub session_id: Option<String>,

    /// Resume an existing session instead of creating it (claude --resume).
    #[arg(long)]
    pub resume: bool,

    /// Hard wrap column for rendered output. Defaults to a sensible terminal-aware value.
    #[arg(short, long, default_value_t = 90)]
    pub width: usize,

    /// PID of an external "thinking" indicator to SIGTERM the moment a token arrives.
    #[arg(long)]
    pub shimmer_pid: Option<i32>,
}

#[derive(Copy, Clone, ValueEnum, Debug, PartialEq, Eq)]
pub enum BackendKind {
    /// Anthropic claude CLI.
    Claude,
    /// OpenAI codex CLI.
    Codex,
    /// Pick whichever CLI is on PATH (claude wins ties).
    Auto,
}
