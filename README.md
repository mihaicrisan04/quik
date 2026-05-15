# quik

Inline AI prompt for zsh. `Cmd+Enter`, ask, streams in place.

<!-- demo gif goes here -->

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
