//! Fallback skip rules, used only when no `.gitignore` applies, plus the
//! dotfile check.

/// Directories skipped when the project has no `.gitignore` of its own.
/// `.git` is additionally always hidden by the walker.
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
pub fn is_hidden(name: &str) -> bool {
    name.starts_with('.') && name != "." && name != ".."
}

/// `true` for directories in the built-in fallback list.
pub fn is_ignored_dir(name: &str) -> bool {
    IGNORED_DIRS.contains(&name)
}

pub fn ignored_dirs_help() -> String {
    IGNORED_DIRS.join(", ")
}
