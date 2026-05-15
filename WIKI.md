# wiki

The README is the pitch. This is how quik actually works.

## architecture

```
keybind (Cmd+Enter)
   → _quik_input        pure-zsh input bar, renders to /dev/tty
   → quik-ask           zsh, mints session id + forwards recent commands
   → quik stream …      rust binary
        → spawns backend (claude | codex)
        → parses NDJSON stream
        → markdown → ANSI, word-wrapped, in place
```

`shell/quik.plugin.zsh` owns integration: keybind, hooks, input bar, session state.
`quik` (rust) owns parsing + rendering. One static binary, no Python or Node.

## per-shell memory

Two things ride along between asks in the same zsh process:

- **session uuid** — minted on first ask, reused via `claude --resume` so follow-ups remember earlier turns.
- **recent commands ring** — last ~30 commands + exit codes, captured via `preexec`/`precmd` hooks. Forwarded once on the next ask, then cleared. Token / key / password / secret / bearer values are masked first.

Both reset when a new shell starts.

## input widget

`_quik_input` is a pure-zsh single-line editor that:

- renders to `/dev/tty`, doesn't clear on submit (no blank frame before the answer)
- handles multi-row wrapping, including the auto-wrap-at-column-boundary quirk
- supports arrows, Home, End, word delete, Ctrl-U / Ctrl-A / Ctrl-E

## stream rendering

The rust binary reads NDJSON line-by-line, normalizes events (`TextDelta`, `MessageStop`), and feeds deltas char-by-char through a small markdown state machine. ANSI toggles for `**bold**`, `*italic*` / `_italic_`, `` `code` ``, ` ```fenced``` `, `# heading`. Word-wrap at the configured column.

The shimmer runs as a zsh background process. When the first token arrives, the rust binary SIGTERMs it and prints `↳` before streaming.

## backends

| backend | status | notes |
| --- | --- | --- |
| claude | stable | `claude -p --output-format stream-json …` |
| codex  | experimental | `codex exec --json …`, schema may drift |

Invocation details in [docs/configuration.md](docs/configuration.md#backends).

## reference

- [docs/install.md](docs/install.md) — plugin managers, source build
- [docs/keybinds.md](docs/keybinds.md) — per-terminal `Cmd+Enter` setup
- [docs/configuration.md](docs/configuration.md) — env vars
