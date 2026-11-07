# wtree

A file and directory tree viewer written in Rust, with no external dependencies.

```text
$ wtree
.
├── src
│   ├── cli.rs
│   ├── ignore.rs
│   ├── main.rs
│   ├── style.rs
│   └── walker.rs
├── Cargo.lock
├── Cargo.toml
├── LICENSE
└── README.md

1 directory, 9 files
```

On a terminal the output is colored:

| Element | Style |
|---|---|
| Directories | bold blue |
| Symlinks | cyan, followed by `-> target` |
| Connectors | dim gray (`├──`, `└──`, `│`) |
| Footer | summary line: `N directories, M files` |

## About

wtree is a small alternative to the `tree` command. It has no external crates, so the whole program is a few source files.

- Stripped release binary is about 332 KB.
- Release build takes about 3 seconds.
- Output is streamed while the directory is walked. The kernel's `d_type` is used, so no `stat` call is needed per entry. A tree of about 200 files prints in about 1 ms.
- Dotfiles and common development directories are skipped by default (see [Skipped by default](#skipped-by-default)).
- Exits cleanly on broken pipes (`wtree | head`) and skips unreadable subdirectories.
- Honors `NO_COLOR` and `CLICOLOR_FORCE`.

## Install

### From source

Requires a Rust toolchain ([rustup](https://rustup.rs)).

```bash
git clone https://github.com/voidd-sam/wtree.git
cd wtree
cargo build --release

# The binary is at target/release/wtree
sudo cp target/release/wtree /usr/local/bin/   # or ~/.local/bin
```

### With cargo

```bash
cargo install --git https://github.com/voidd-sam/wtree
```

### Prebuilt binary

Prebuilt x86-64 Linux binaries are on the [releases page](https://github.com/voidd-sam/wtree/releases). Put one anywhere on your `PATH`:

```bash
cp wtree ~/.local/bin/    # or /usr/local/bin with sudo
chmod +x ~/.local/bin/wtree
```

Check the install:

```bash
wtree --version
```

## Usage

```bash
wtree [OPTIONS] [PATH]
```

| Option | Description |
|---|---|
| `<PATH>` | Directory to list (default: current directory) |
| `-d`, `--depth <N>` | Maximum display depth (children of the root are level 1) |
| `-a`, `--all` | Show hidden files and directories |
| `-h`, `--help` | Print help |
| `-V`, `--version` | Print version |

## Examples

Current directory:

```bash
wtree
```

A specific directory, 2 levels deep:

```bash
wtree ~/projects/my-app --depth 2
```

Include hidden files and the normally skipped directories:

```bash
wtree --all
```

Keep color when piping into `less`:

```bash
CLICOLOR_FORCE=1 wtree | less -R
```

Turn color off:

```bash
NO_COLOR=1 wtree
```

## Behavior

- **Sorting:** directories first, then files. Alphabetical and case-insensitive within each group.
- **Symlinks:** printed in cyan with `-> target`. The target is shown even when color is off.
- **Color:** on when stdout is a terminal, off when piped or redirected. `NO_COLOR` turns it off and `CLICOLOR_FORCE` turns it on.
- **Broken pipes:** if the reader closes early (for example `wtree | head`), wtree exits without an error message.
- **Unreadable subdirectories:** skipped silently.

### Environment variables

| Variable | Effect |
|---|---|
| `NO_COLOR` | Disables color, even on a terminal |
| `CLICOLOR_FORCE` | Forces color, even when piped or redirected |

### Skipped by default

Dotfiles and these directories are hidden:

```text
node_modules  target  .git  dist  build  .next  __pycache__  .venv
```

Use `--all` to show them.

## Project layout

```text
wtree/
├── Cargo.toml        # package and release profile (LTO, strip, panic=abort)
├── Cargo.lock
├── LICENSE
├── README.md
└── src/
    ├── main.rs       # entry point, help/version text, error mapping
    ├── cli.rs        # argument parser (no clap)
    ├── ignore.rs     # hidden-name and dev-directory skip rules
    ├── style.rs      # ANSI color helpers and TTY detection
    └── walker.rs     # directory walk, tree rendering, summary footer
```

## Contributing

Issues and pull requests are welcome at https://github.com/voidd-sam/wtree/.

The project is intentionally small: no external dependencies and a handful of files. Changes that keep it that way are the most likely to be merged.

## License

MIT. See [LICENSE](LICENSE).
