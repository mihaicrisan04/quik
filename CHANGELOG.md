# Changelog

All notable changes are listed here. This project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] — first cut

- zsh plugin: `Cmd+Enter` opens an inline prompt; pure-zsh input bar handles
  multi-row wrapping correctly.
- Per-shell session memory via `claude --session-id` / `--resume`.
- Recent shell commands (with exit codes, light secret redaction) forwarded
  as context on each ask.
- Streaming markdown → ANSI renderer in rust: **bold**, *italic*, `inline
  code`, ```fenced```, and `# heading`.
- Backends: `claude` (stable), `codex` (experimental).
- Animated braille "thinking..." shimmer that the binary cancels the moment
  the first token arrives.
