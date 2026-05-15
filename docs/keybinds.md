# Keybinds

`quik` listens for whatever escape sequence is bound via `QUIK_KEYBIND` (default: `^_`, i.e. `Ctrl+Underscore`, which corresponds to byte `0x1f`). The trick to making `Cmd+Enter` feel natural is having your terminal send `0x1f` on that chord.

## Per-terminal setup

### Ghostty

```
keybind = cmd+enter=text:\x1f
```

### iTerm2

Preferences → Keys → Key Bindings → `+`
- Keyboard Shortcut: `⌘ Return`
- Action: `Send Hex Code`
- Hex: `0x1f`

### Kitty

```
map cmd+enter send_text all \x1f
```

### Alacritty (`alacritty.toml`)

```toml
[[keyboard.bindings]]
key = "Return"
mods = "Command"
chars = ""
```

### WezTerm (`wezterm.lua`)

```lua
config.keys = {
  { key = "Enter", mods = "CMD", action = wezterm.action.SendString "\x1f" },
}
```

### Tmux passthrough

If you run inside tmux, make sure the escape isn't intercepted. Tmux passes `0x1f` through by default.

## Choosing a different chord

If `Cmd+Enter` clashes with another shortcut, pick any unused control byte. Common alternatives:

| Chord | Byte | `QUIK_KEYBIND` value |
| --- | --- | --- |
| `Ctrl+G` | `0x07` | `^G` |
| `Ctrl+B` | `0x02` | `^B` |
| `Ctrl+\` | `0x1c` | `^\\` |

Set the env var before sourcing the plugin:

```sh
export QUIK_KEYBIND='^G'
source "$(brew --prefix)/share/quik/quik.plugin.zsh"
```
