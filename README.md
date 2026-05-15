# quik

inline AI prompt for zsh. `Cmd+Enter`, ask, streams in place.

sorry Warp, i like my terminal

<img width="1040" height="720" alt="quik_final" src="https://github.com/user-attachments/assets/349fa2dc-60c8-434b-9424-11f5332c58f0" />

## install

```sh
brew install mihaicrisan04/tap/quik
```

Add to `~/.zshrc`:

```sh
source "$(brew --prefix)/share/quik/quik.plugin.zsh"
```

Needs [`claude`](https://docs.claude.com/en/docs/claude-code) or [`codex`](https://github.com/openai/codex) on `PATH`.

## keybind

`quik` listens for byte `0x1f`. Map `Cmd+Enter` → it in your terminal — see [docs/keybinds.md](docs/keybinds.md).

## more

[wiki](WIKI.md) · [roadmap](ROADMAP.md) · [changelog](CHANGELOG.md)

## license

MIT
