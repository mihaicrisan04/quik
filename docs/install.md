# Install

## Homebrew (macOS / Linuxbrew)

```sh
brew install mihaicrisan04/tap/quik
```

Add to your `~/.zshrc`:

```sh
source "$(brew --prefix)/share/quik/quik.plugin.zsh"
```

## From source

```sh
git clone https://github.com/mihaicrisan04/quik
cd quik
mise install            # installs the pinned rust toolchain
mise run install        # builds release, places binary in ~/.cargo/bin
```

Then source the plugin from anywhere (the repo path or a copy on disk):

```sh
source ~/path/to/quik/shell/quik.plugin.zsh
```

## Plugin managers

### zinit

```sh
zinit light mihaicrisan04/quik
```

### antidote

In your bundle file:

```
mihaicrisan04/quik path:shell
```

### oh-my-zsh

```sh
git clone https://github.com/mihaicrisan04/quik ~/.oh-my-zsh/custom/plugins/quik
ln -s ~/.oh-my-zsh/custom/plugins/quik/shell/quik.plugin.zsh \
      ~/.oh-my-zsh/custom/plugins/quik/quik.plugin.zsh
```

Then add `quik` to your `plugins=(...)` array in `.zshrc`.

## Prerequisites

You need at least one of these CLIs on your `PATH`:

- [`claude`](https://docs.claude.com/en/docs/claude-code) — recommended.
- [`codex`](https://github.com/openai/codex) — experimental.

And a terminal that supports sending custom escape sequences on a chord. See [keybinds.md](keybinds.md) for `Cmd+Enter` setup.
