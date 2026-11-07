wtree

A fast, lightweight, colorful file & directory tree viewer — written in pure Rust, with zero external dependencies.

Type one command. Get a beautiful tree of any directory.

License: MITLanguage: RustDependencies: 0Binary size: 332 KB

Why wtree · Install · Usage · Examples · Behavior notes · Project layout · Contributing · License
What it looks like

 $ wtree.├── src│   ├── cli.rs│   ├── ignore.rs│   ├── main.rs│   ├── style.rs│   └── walker.rs├── Cargo.lock├── Cargo.toml├── LICENSE└── README.md1 directory, 9 files

On a terminal, the output is colorized end to end:
Element	Style
Directories	bold blue
Symlinks	cyan, followed by -> target
Connectors	dim gray box-drawing characters (├──, └──, │)
Footer	summary line — N directories, M files
Why wtree

The classic tree command is great, but its implementations are either written in C (hard to read and extend) or pull in dozens of dependencies. wtree takes a different road:

    Tiny — a single ~332 KB stripped binary, zero external crates, and a release build in about 3 seconds.
    Fast — output is streamed while the directory is walked, and the kernel's d_type is used so no stat call is needed per entry. A tree of ~200 files prints in ~1 ms.
    Beautiful by default — directories in bold blue, symlinks in cyan with their target, gray box-drawing connectors, and a summary footer.
    Sane defaults — dotfiles and noisy development directories (node_modules, target, .git, dist, build, .next, __pycache__, .venv) are skipped, so you actually see your project.
    A polite CLI citizen — it exits cleanly on broken pipes (wtree | head), skips unreadable subdirectories instead of crashing, and honors the NO_COLOR and CLICOLOR_FORCE conventions.

At a glance
Metric	Value
Stripped binary size	~332 KB
External dependencies	0
Release build time	~3 seconds
Tree of ~200 files	printed in ~1 ms
Install
From source (recommended)

Requires a Rust toolchain — rustup is the easiest way to get one.

git clone https://github.com/voidd-sam/wtree.gitcd wtreecargo build --release# The binary is at target/release/wtreesudo cp target/release/wtree /usr/local/bin/   # or ~/.local/bin

With cargo

cargo install --git https://github.com/voidd-sam/wtree

Prebuilt binary

Check the releases page for prebuilt x86-64 Linux binaries, then drop one anywhere on your PATH:

cp wtree ~/.local/bin/    # or /usr/local/bin with sudochmod +x ~/.local/bin/wtree

Verify the installation:

wtree --version

Usage

wtree [OPTIONS] [PATH]

Option	Description
<PATH>	Directory to list (default: current directory)
-d, --depth <N>	Maximum display depth (children of the root are level 1)
-a, --all	Show hidden files and directories (dotfiles)
-h, --help	Print help
-V, --version	Print version
Full wtree --help output
Examples

Tree of the current directory:

wtree

Tree of a specific project, capped at 2 levels deep:

wtree ~/projects/my-app --depth 2

Show everything (including .git, .env, node_modules, …):

wtree --all

Force color when piping into less:

CLICOLOR_FORCE=1 wtree | less -R

Disable color entirely:

NO_COLOR=1 wtree

    Tip: color is automatic — wtree colorizes on a terminal and switches to plain text when piped or redirected, so wtree | less stays clean. CLICOLOR_FORCE=1 is the escape hatch when you want colors through a pipe.

Behavior notes

    Sorting — directories first, then files; case-insensitive alphabetical order within each group.
    Symlinks — printed in cyan with -> target; the target is shown even when color is disabled.
    Color — auto-enabled when stdout is a TTY, auto-disabled when the output is piped or redirected. NO_COLOR disables it, CLICOLOR_FORCE forces it on.
    Broken pipes — when the consumer closes early (e.g. wtree | head), wtree exits cleanly instead of spewing a broken-pipe error.
    Unreadable subdirectories — skipped silently instead of aborting the whole run.

Environment variables
Variable	Effect
NO_COLOR	Disables color, even on a TTY
CLICOLOR_FORCE	Forces color on, even when piped or redirected
Skipped by default

To keep the output useful rather than noisy, wtree hides dotfiles and these development directories:

node_modules  target  .git  dist  build  .next  __pycache__  .venv

Pass --all to see everything.
Project layout

wtree/├── Cargo.toml        # package + release profile (LTO, strip, panic=abort)├── Cargo.lock├── LICENSE├── README.md└── src/    ├── main.rs       # entry point, help/version text, error mapping    ├── cli.rs        # hand-rolled arg parser (no clap)    ├── ignore.rs     # hidden-name + dev-dir skip rules    ├── style.rs      # ANSI color helpers + TTY detection    └── walker.rs     # directory walk, tree rendering, summary footer

Contributing

Issues and pull requests are welcome at https://github.com/voidd-sam/wtree/.

wtree is deliberately tiny: zero external dependencies, one small crate, a handful of files. If you'd like to contribute, keep that spirit — small, readable, dependency-free changes are the most likely to be merged.
