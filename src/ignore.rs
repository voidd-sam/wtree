//! Default skip rules: hidden entries (dotfiles) and common noisy dev directories.

/// Directories that are skipped even when `--all` is passed, because they are
/// build output / dependency caches and would drown out everything else.
const IGNORED_DIRS: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    "dist",
    "build",
    ".next",
    "__pycache__",
    ".venv",
];

/// `true` for names that start with a dot (`.git`, `.env`, `.vscode`).
/// `.` and `..` are not treated as hidden (they are not real entries anyway).
pub fn is_hidden(name: &str) -> bool {
    name.starts_with('.') && name != "." && name != ".."
}

/// `true` for directories in the default dev-ignore list.
pub fn is_ignored_dir(name: &str) -> bool {
    IGNORED_DIRS.contains(&name)
}

pub fn ignored_dirs_help() -> String {
    IGNORED_DIRS.join(", ")
}
