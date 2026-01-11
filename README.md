# sesh

A fully-featured Terminal User Interface for managing GNU Screen sessions.

[![CI](https://github.com/ali205412/sesh/actions/workflows/ci.yml/badge.svg)](https://github.com/ali205412/sesh/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Features

- **Session Management** - List, create, attach, detach, and kill screen sessions
- **Window Management** - Manage windows within sessions
- **Live Preview** - See terminal content of selected sessions
- **Templates** - Create sessions from YAML templates
- **SSH Support** - Manage remote screen sessions
- **Git Integration** - Show git branch/status for session directories
- **Shell Integration** - Fish, Bash, and Zsh hooks and completions
- **Keyboard-Driven** - Vim-style navigation (hjkl) + arrow keys

## Installation

### From Source

```bash
cargo install --git https://github.com/ali205412/sesh
```

### Fedora (COPR)

```bash
sudo dnf copr enable ali205412/sesh
sudo dnf install sesh
```

### From Releases

Download the latest binary from [Releases](https://github.com/ali205412/sesh/releases).

## Usage

```bash
# Launch TUI
sesh

# CLI commands
sesh list              # List sessions
sesh new <name>        # Create session
sesh attach <session>  # Attach to session
sesh kill <session>    # Kill session
sesh start <template>  # Create from template
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Attach to session |
| `n` | New session |
| `d` | Detach session |
| `x` | Kill session |
| `w` | View windows |
| `t` | Templates |
| `/` | Search |
| `?` | Help |
| `q` | Quit |

## Configuration

Config file: `~/.config/sesh/config.toml`

```toml
[ui]
theme = "dark"
show_preview = true
preview_lines = 8

[screen]
attach_mode = "exec"  # or "spawn" for new terminal
spawn_terminal = "alacritty"

[navigation]
vim_keys = true
arrow_keys = true
mouse_enabled = true
```

## Templates

Templates: `~/.config/sesh/templates/*.yaml`

```yaml
name: webdev
description: "Web development environment"
root: ~/projects/${PROJECT_NAME}

windows:
  - name: editor
    command: nvim .
  - name: server
    command: npm run dev
  - name: shell
```

## Shell Integration

### Fish

Add to `~/.config/fish/conf.d/sesh.fish`:

```fish
# Abbreviations
abbr --add ss 'sesh'
abbr --add sl 'sesh list'
abbr --add sn 'sesh new'
abbr --add sa 'sesh attach'

# Keybinding: Ctrl+S to launch sesh
bind \cs 'commandline -r sesh; commandline -f execute'
```

### Bash/Zsh

Run `sesh` and check the help for shell integration scripts.

## Building

```bash
git clone https://github.com/ali205412/sesh
cd sesh
cargo build --release
```

## License

MIT License - see [LICENSE](LICENSE)
