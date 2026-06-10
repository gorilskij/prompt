# gprompt

A fast shell gprompt with git-state detection, path shortening, and Python venv display.

## Install

### Homebrew (recommended)

```sh
brew install gorilskij/tap/gprompt
```

### npm

```sh
npm install -g gprompt
```

### cargo

```sh
cargo install gprompt
```

### Build from source

```sh
cargo build --release
# binary is at target/release/gprompt
```

## Shell setup

After installing, add one line to your shell config so the gprompt is used.

### fish

```fish
echo 'gprompt init fish | source' >> ~/.config/fish/config.fish
```

### bash

```bash
echo 'eval "$(gprompt init bash)"' >> ~/.bashrc
```

### zsh

```zsh
echo 'eval "$(gprompt init zsh)"' >> ~/.zshrc
```

Then restart your shell (or `source` the config file).

## Configuration

Create `~/.config/gprompt/config.toml` to customise path aliases:

```toml
[aliases]
c = "~/code"
D = "~/Desktop"
```

Aliases are matched as path prefixes and replace the matching portion with the alias key in the gprompt display.

## Prompt anatomy

```
⟨branch|~/c/project⟩ 
```

- `⟨…⟩` delimiters are blue
- Python venv shown when `VIRTUAL_ENV_PROMPT` is set
- Git branch shown in the current repo (detached HEAD shows abbreviated hash)
- Path is shortened: intermediate directories are truncated to their first letter
- `!` prefix if any rendering error occurred
