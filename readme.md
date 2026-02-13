# helper-git

A simple, beautiful terminal user interface (TUI) for.

## Current Features (v0.1.0)
- Interactive `git status`
  - Arrow key navigation
  - Highlighted selection
  - Repository state detection

⚠️ This release focuses exclusively on the `status` command.
Additional Git workflows (add, commit, push, diff, etc.) are planned for future versions.

Built with:
- [ratatui](https://github.com/ratatui-org/ratatui) – terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) – terminal manipulation
- [git2-rs](https://github.com/rust-lang/git2-rs) – libgit2 bindings
- [anyhow](https://github.com/dtolnay/anyhow) – clean error handling

https://github.com/mi-tec/helper-git

## Features

- Shows working tree status (untracked, modified, added, type changed, renamed, etc.)
- Color-coded status labels (red = new/untracked, yellow = modified, green = staged/added, orange = type change)
- Keyboard navigation: ↑/↓ (or j/k), Home, End
- Highlighted selected file with arrow indicator
- Clean "working tree clean" message when nothing to show
- Press `q` or `Esc` to quit
- Uses alternate screen buffer → clean exit

## Demo

![Demo](assets/helper-git-status.gif)

## Installation

### From source (recommended for now)

```bash
# Clone the repo
git clone https://github.com/mi-tec/helper-git
cd helper-git

# Build and install (or just run with cargo run)
cargo install --path .
```

## Usage
```hg status```
