# roadmap

Short bullets, flesh out when picked up.

## next

- codex: validate schema against current `codex exec --json` output, wire session continuity
- `update-formula.yml` action to bump the brew tap automatically on every tag push
- bash + fish ports of the shell plugin
- `quik history` — browse past asks in the current shell

## maybe

- ollama backend (local models)
- gemini-cli backend
- config file (`~/.config/quik/config.toml`) as alt to env vars
- streaming markdown: tables, links, bullet lists
- multi-line input mode (Alt+Enter for newline inside the bar)
- subtle "done" sound (opt-in)
- prompt history persisted across shells (opt-in, file-backed)

## known issues

- column-boundary auto-wrap quirk handled with a sentinel space — if you see a one-cell glitch at terminal-width boundaries, file an issue with your terminal name
- codex backend may emit no output if the schema has drifted; paste a sample of `codex exec --json "hi"` in an issue
