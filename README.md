# quik

> Inline AI prompt for your zsh session. Press `Cmd+Enter` mid-shell, ask anything, get a streamed answer right where you are. No new window, no context switch.

```
~/dev/personal/quik main
❯ how do i list only directories in a path?

⠹ thinking...
↳
Use `ls -d */` for the simple case, or `find . -maxdepth 1 -type d`
when you want hidden directories included.
```

## Why

You're in a terminal. You hit a wall — wrong flag, fuzzy memory, half-remembered command. Today you tab over to a chat window, paste the error, copy back a fix. `quik` removes the tab-over: a tiny prompt drops in beneath the cursor, you ask, the answer streams in place, and you keep going.

It threads in two things that make terminal-context answers good:

- **Per-shell session memory** — follow-ups in the same terminal remember what you just asked.
- **Recent command context** — your last ~30 commands and their exit codes are forwarded automatically (with values for things like `token=`, `password=` masked).

## Install

```sh
brew install mihaicrisan04/tap/quik
```

Then add this to your `.zshrc`:

```sh
source "$(brew --prefix)/share/quik/quik.plugin.zsh"
```

You also need at least one backend CLI on your `PATH`:

- [`claude`](https://docs.claude.com/en/docs/claude-code) (recommended)
- [`codex`](https://github.com/openai/codex) (OpenAI)

`quik` auto-picks whichever is installed (claude wins ties). Override with `export QUIK_BACKEND=codex`.

## Keybind

`quik` ships bound to the escape sequence `\x1f` (Ctrl+Underscore), which most modern terminals can produce on `Cmd+Enter` with a one-line config:

### Ghostty

In `~/.config/ghostty/config`:

```
keybind = cmd+enter=text:\x1f
```

### iTerm2

Preferences → Keys → Key Bindings → `+` → Cmd+Enter → Action: "Send Hex Code" → `0x1f`.

### Kitty

In `~/.config/kitty/kitty.conf`:

```
map cmd+enter send_text all \x1f
```

### Alacritty

In `~/.config/alacritty/alacritty.toml`:

```toml
[[keyboard.bindings]]
key = "Return"
mods = "Command"
chars = ""
```

### WezTerm

In `~/.wezterm.lua`:

```lua
config.keys = {
  { key = "Enter", mods = "CMD", action = wezterm.action.SendString "\x1f" },
}
```

Want a different binding? Set `export QUIK_KEYBIND='^X'` before sourcing the plugin.

## Configuration

All env vars are read once when the plugin is sourced.

| Variable | Default | Purpose |
| --- | --- | --- |
| `QUIK_BACKEND` | `auto` | `claude`, `codex`, or `auto`. |
| `QUIK_KEYBIND` | `^_` | `bindkey` escape sequence to bind. |
| `QUIK_BIN` | `quik` | Path to the `quik` binary. |
| `QUIK_WIDTH` | `min(COLUMNS, 90)` | Wrap column for streamed responses. |

## How it works

Two pieces:

- **`shell/quik.plugin.zsh`** — a zle widget bound to your keybind. It captures input with a pure-zsh prompt that handles multi-row buffers, runs `preexec`/`precmd` hooks to keep a redacted command-history buffer, and shells out to the binary.
- **`quik` binary (rust)** — spawns the chosen backend CLI, parses its streaming output, kills the shimmer animation as soon as the first token lands, and renders the response with incremental markdown → ANSI (bold, italic, inline code, fenced code, headings) and word-wrapping.

The split keeps the integration-heavy parts in zsh where they're natural, and the parsing/rendering parts in a single static binary with no Python or Node dependencies.

## Building from source

```sh
git clone https://github.com/mihaicrisan04/quik
cd quik
mise install            # installs the pinned rust toolchain
mise run install        # builds release and installs to ~/.cargo/bin
```

Then source `shell/quik.plugin.zsh` from anywhere.

## Status

`v0.1` — works locally with `claude`. `codex` backend is wired but the upstream codex CLI's streaming schema is still evolving; report mismatches with the output of `codex exec --json "hello" 2>&1 | head`.

## License

MIT — see [LICENSE](LICENSE).
