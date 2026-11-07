# tree-rs

A fast, lightweight, colorful file/folder tree viewer written in **pure Rust** (zero external dependencies). Type one command, get a beautiful tree of any directory.

```
$ tree .
.
├── src
│   ├── cli.rs
│   ├── ignore.rs
│   ├── main.rs
│   ├── style.rs
│   └── walker.rs
├── Cargo.lock
└── Cargo.toml

1 directory, 7 files
```

## Why

The classic `tree` command is great, but its implementations are either C (hard to read/extend) or pull in dozens of dependencies. `tree-rs` is:

- **Tiny** — 332 KB stripped binary, zero dependencies, compiles in ~3 seconds.
- **Fast** — streams output as it walks, uses the kernel `d_type` (no `stat` per entry). A tree of ~200 files prints in ~1 ms.
- **Beautiful by default** — directories in bold blue, symlinks in cyan with their target, gray box-drawing connectors, and a summary footer.
- **Sane defaults** — skips dotfiles and noisy dev directories (`node_modules`, `target`, `.git`, `dist`, `build`, `.next`, `__pycache__`, `.venv`) so you actually see your project.

## Install

### From source (recommended)

```sh
git clone <your-repo-url> tree-rs
cd tree-rs
cargo build --release
# Binary is at target/release/tree
sudo cp target/release/tree /usr/local/bin/   # or ~/.local/bin
```

### Prebuilt binary

The `download/` folder in this delivery contains a prebuilt `tree` binary for x86-64 Linux. Copy it onto your `PATH`:

```sh
cp download/tree ~/.local/bin/   # or /usr/local/bin with sudo
```

Verify:

```sh
tree --version
```

## Usage

```
tree-rs 0.1.0 — a fast, lightweight file/folder tree viewer

USAGE:
    tree [OPTIONS] [PATH]

ARGS:
    <PATH>    Directory to list (default: current directory)

OPTIONS:
    -d, --depth <N>    Maximum display depth (children of root are level 1)
    -a, --all           Show hidden files and directories (dotfiles)
    -h, --help          Print this help
    -V, --version       Print version

By default these directories are skipped: node_modules, target, .git, dist, build, .next, __pycache__, .venv
Dotfiles (.git, .env, .vscode, ...) are hidden unless --all is passed.
Color is auto-enabled on a TTY and disabled when piped/redirected,
honoring the NO_COLOR and CLICOLOR_FORCE environment variables.
```

## Examples

Tree of the current directory:

```sh
tree
```

Tree of a specific project, capped at 2 levels deep:

```sh
tree ~/projects/my-app --depth 2
```

Show everything (including `.git`, `.env`, `node_modules`, …):

```sh
tree --all
```

Force color when piping into `less`:

```sh
CLICOLOR_FORCE=1 tree | less -R
```

Disable color entirely:

```sh
NO_COLOR=1 tree
```

## Behavior notes

- **Sorting:** directories first, then files; case-insensitive alphabetical within each group.
- **Symlinks:** printed in cyan with `-> target` (target shown even in plain mode).
- **Color:** auto-enabled when stdout is a TTY, auto-disabled when piped/redirected. Honors `NO_COLOR` (disables) and `CLICOLOR_FORCE` (forces on).
- **Broken pipes:** when the consumer closes early (e.g. `tree | head`), `tree` exits cleanly instead of printing a broken-pipe error.
- **Unreadable subdirectories:** skipped silently rather than aborting the run.

## Project layout

```
tree-rs/
├── Cargo.toml        # package + release profile (LTO, strip, panic=abort)
├── README.md
└── src/
    ├── main.rs       # entry point, help/version text, error mapping
    ├── cli.rs        # hand-rolled arg parser (no clap)
    ├── ignore.rs     # hidden-name + dev-dir skip rules
    ├── style.rs      # ANSI color helpers + TTY detection
    └── walker.rs     # directory walk, tree rendering, summary footer
```

## License

MIT
