# pl

A terminal UI project launcher. Browse, search, and open your local git repositories.

## Features

- Fuzzy search across projects
- Open projects in your editor
- Open project remote in the browser (Only Github supported)
- Sort alphabetically or by recently modified
- README preview for the selected project

## Installation

```sh
cargo install --path .
```

## Configuration

Config file: `~/.config/pl/config.toml`

```toml
project_dirs = ["~/Projects"]
editor_command = "code"
```

Defaults to `~/Projects` and VS Code if no config file exists.
