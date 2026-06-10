# prompt

A fast shell prompt with git-state detection, path shortening, and Python venv display.

## Install

### Homebrew (recommended)

```sh
brew install gorilskij/tap/prompt
```

### cargo

```sh
cargo install prompt
```

> **Note:** the crate name `prompt` may be taken on crates.io. If so, use the Homebrew method or build from source.

### Build from source

```sh
cargo build --release
# binary is at target/release/prompt
```

## Shell setup

After installing, add one line to your shell config so the prompt is used.

### fish

```fish
echo 'prompt init fish | source' >> ~/.config/fish/config.fish
```

### bash

```bash
echo 'eval "$(prompt init bash)"' >> ~/.bashrc
```

### zsh

```zsh
echo 'eval "$(prompt init zsh)"' >> ~/.zshrc
```

Then restart your shell (or `source` the config file).

## Configuration

Create `~/.config/prompt/config.toml` to customise path aliases:

```toml
[aliases]
c = "~/code"
D = "~/Desktop"
```

Aliases are matched as path prefixes and replace the matching portion with the alias key in the prompt display.

## Prompt anatomy

```
⟨branch|~/c/project⟩ 
```

- `⟨…⟩` delimiters are blue
- Python venv shown when `VIRTUAL_ENV_PROMPT` is set
- Git branch shown in the current repo (detached HEAD shows abbreviated hash)
- Path is shortened: intermediate directories are truncated to their first letter
- `!` prefix if any rendering error occurred
