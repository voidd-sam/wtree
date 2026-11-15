//! Minimal ANSI styling and file-type classification.
//! No crate - raw escape codes keep the binary dependency-free.
//!
//! Icons are Nerd Font glyphs, written as `\u{...}` escapes so the source stays
//! readable in any editor. They are only printed with `--icons`.

use std::io::IsTerminal;

const RESET: &str = "\x1b[0m";

/// Wrap `s` in an SGR code such as `"1;34"` or `"38;5;114"`.
pub fn paint(code: &str, s: &str) -> String {
    format!("\x1b[{}m{}{}", code, s, RESET)
}

pub struct Style;

impl Style {
    /// Bold blue - directories.
    pub fn dir(s: &str) -> String {
        paint("1;34", s)
    }
    /// Cyan - symbolic links (with `-> target`).
    pub fn symlink(s: &str) -> String {
        paint("36", s)
    }
    /// Gray - tree connectors and the summary footer.
    pub fn gray(s: &str) -> String {
        paint("90", s)
    }
    /// Darker gray - entries that are ignored but shown anyway (`--no-ignore`).
    pub fn ignored(s: &str) -> String {
        paint("38;5;240", s)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Code,
    Script,
    Config,
    Doc,
    Image,
    Media,
    Archive,
    Plain,
}

impl Kind {
    pub fn color(self) -> Option<&'static str> {
        match self {
            Kind::Code => Some("38;5;114"),
            Kind::Script => Some("1;32"),
            Kind::Config => Some("38;5;179"),
            Kind::Doc => Some("38;5;223"),
            Kind::Image => Some("38;5;176"),
            Kind::Media => Some("38;5;141"),
            Kind::Archive => Some("38;5;167"),
            Kind::Plain => None,
        }
    }
}

pub const ICON_DIR: &str = "\u{f07b}";
pub const ICON_SYMLINK: &str = "\u{f0c1}";
const ICON_FILE: &str = "\u{f15b}";

/// Classify a file by name: returns its color group and its Nerd Font icon.
pub fn classify(name: &str) -> (Kind, &'static str) {
    match name {
        "Cargo.toml" | "Cargo.lock" => return (Kind::Config, "\u{e7a8}"),
        ".gitignore" | ".gitattributes" | ".gitmodules" => return (Kind::Config, "\u{e702}"),
        "Makefile" | "makefile" | "justfile" | "Justfile" => return (Kind::Script, "\u{f013}"),
        "Dockerfile" => return (Kind::Config, "\u{f308}"),
        "LICENSE" | "LICENCE" | "COPYING" => return (Kind::Doc, "\u{f15c}"),
        _ => {}
    }
    if name.len() >= 6 && name[..6].eq_ignore_ascii_case("readme") {
        return (Kind::Doc, "\u{e73e}");
    }
    let ext = match name.rsplit_once('.') {
        Some((stem, e)) if !stem.is_empty() => e.to_ascii_lowercase(),
        _ => return (Kind::Plain, ICON_FILE),
    };
    match ext.as_str() {
        "rs" => (Kind::Code, "\u{e7a8}"),
        "py" => (Kind::Code, "\u{e73c}"),
        "js" | "mjs" | "cjs" | "jsx" => (Kind::Code, "\u{e74e}"),
        "ts" | "tsx" => (Kind::Code, "\u{e628}"),
        "go" => (Kind::Code, "\u{e627}"),
        "c" | "h" => (Kind::Code, "\u{e61e}"),
        "cpp" | "cc" | "hpp" => (Kind::Code, "\u{e61d}"),
        "java" => (Kind::Code, "\u{e738}"),
        "lua" => (Kind::Code, "\u{e620}"),
        "html" | "htm" => (Kind::Code, "\u{e736}"),
        "css" | "scss" => (Kind::Code, "\u{e749}"),
        "sh" | "bash" | "zsh" | "fish" => (Kind::Script, "\u{f489}"),
        "md" | "markdown" => (Kind::Doc, "\u{e73e}"),
        "txt" | "rst" => (Kind::Doc, "\u{f15c}"),
        "pdf" => (Kind::Doc, "\u{f1c1}"),
        "json" => (Kind::Config, "\u{e60b}"),
        "toml" | "yaml" | "yml" | "ini" | "conf" | "cfg" => (Kind::Config, "\u{e615}"),
        "lock" => (Kind::Config, "\u{f023}"),
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "bmp" | "ico" => {
            (Kind::Image, "\u{f1c5}")
        }
        "mp3" | "wav" | "flac" | "ogg" | "mp4" | "mkv" | "mov" | "webm" => {
            (Kind::Media, "\u{f1c8}")
        }
        "zip" | "tar" | "gz" | "xz" | "bz2" | "zst" | "7z" | "rar" => (Kind::Archive, "\u{f1c6}"),
        _ => (Kind::Plain, ICON_FILE),
    }
}

/// Decide whether color should be enabled, honoring NO_COLOR / CLICOLOR_FORCE
/// and falling back to "is stdout a TTY?".
pub fn should_color() -> bool {
    let no_color = std::env::var_os("NO_COLOR")
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    let force = std::env::var_os("CLICOLOR_FORCE")
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    force || (std::io::stdout().is_terminal() && !no_color)
}
