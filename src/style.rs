//! Minimal ANSI styling. No crate — raw escape codes keep the binary dependency-free.

const RESET: &str = "\x1b[0m";

pub struct Style;

impl Style {
    /// Bold blue — used for directories.
    pub fn dir(s: &str) -> String {
        format!("\x1b[1;34m{}{}", s, RESET)
    }
    /// Cyan — used for symbolic links (with `-> target`).
    pub fn symlink(s: &str) -> String {
        format!("\x1b[36m{}{}", s, RESET)
    }
    /// Bright-black (gray) — used for the tree connectors and the summary footer.
    pub fn gray(s: &str) -> String {
        format!("\x1b[90m{}{}", s, RESET)
    }
}

/// Decide whether color should be enabled, honoring NO_COLOR / CLICOLOR_FORCE
/// and falling back to "is stdout a TTY?".
pub fn should_color() -> bool {
    use std::io::IsTerminal;
    let no_color = std::env::var_os("NO_COLOR")
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    let force = std::env::var_os("CLICOLOR_FORCE")
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    force || (std::io::stdout().is_terminal() && !no_color)
}
