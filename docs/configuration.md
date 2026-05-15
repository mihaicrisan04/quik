# Configuration

`quik` is configured via environment variables, read once when the zsh plugin is sourced.

## Variables

| Variable | Default | What it does |
| --- | --- | --- |
| `QUIK_BACKEND` | `auto` | Which backend CLI to invoke. `claude`, `codex`, or `auto` (claude wins ties). |
| `QUIK_KEYBIND` | `^_` | `bindkey` escape sequence the widget binds to. See [keybinds.md](keybinds.md). |
| `QUIK_BIN` | `quik` | Path to the binary. Override for development checkouts. |
| `QUIK_WIDTH` | `min(COLUMNS, 90)` | Hard wrap column for the rendered response. |

## Per-shell state

The plugin maintains two pieces of state inside each running zsh process:

- **Session id** — a UUID minted on first ask in a shell. Reused via
  `claude --resume` on subsequent asks so the model remembers earlier turns.
  Resets when you open a new terminal.
- **Recent commands buffer** — a rolling ring of up to 30 entries shaped like
  `[exit_code] command`. Forwarded as context on the next ask, then cleared.
  Values for tokens, keys, passwords, secrets, bearer tokens, etc are masked
  before they leave the shell.

To disable command forwarding entirely:

```sh
precmd_functions=(${precmd_functions[@]:#_quik_post})
preexec_functions=(${preexec_functions[@]:#_quik_pre})
```

## Backends

### `claude`

Stable. Invokes:

```
claude -p --output-format stream-json --verbose --include-partial-messages \
       [--session-id <uuid> | --resume <uuid>] \
       --append-system-prompt "<terse-system-prompt>" \
       "<query>"
```

You need an authenticated claude CLI on `PATH`. Follow the [claude docs](https://docs.claude.com/en/docs/claude-code) for auth.

### `codex`

Experimental. Invokes:

```
codex exec --json "<query>"
```

Codex session continuity isn't wired yet — each call is independent. The codex CLI's NDJSON schema is still moving; if you see no output, please open an issue with a sample of:

```sh
codex exec --json "hello" 2>&1 | head -50
```
