//! Directory walking + tree rendering.
//!
//! Designed to be fast and allocation-light:
//! - Uses `read_dir` and the kernel-provided `d_type` (no extra `stat` per entry).
//! - Streams output to a locked stdout handle as it walks (no buffering whole trees).
//! - Sorts directories before files, alphabetically within each group.

use std::fs::{self, FileType};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::ignore;
use crate::style::Style;

pub struct Config {
    pub root: PathBuf,
    pub max_depth: Option<usize>,
    pub show_hidden: bool,
    pub colors: bool,
}

#[derive(Default)]
pub struct Counts {
    pub dirs: usize,
    pub files: usize,
}

/// Print the tree for `cfg.root` and the summary footer. Returns entry counts.
pub fn run(cfg: &Config) -> io::Result<Counts> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let meta = fs::metadata(&cfg.root).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("cannot access '{}': {}", cfg.root.display(), e),
        )
    })?;

    let root_label = cfg.root.display().to_string();
    let root_styled = if cfg.colors {
        Style::dir(&root_label)
    } else {
        root_label.clone()
    };
    writeln!(out, "{}", root_styled)?;

    let mut counts = Counts::default();
    if meta.is_dir() {
        walk_dir(&cfg.root, "", 1, cfg, &mut out, &mut counts)?;
    }
    print_footer(&mut out, &counts, cfg.colors)?;
    Ok(counts)
}

fn print_footer(out: &mut impl Write, counts: &Counts, colors: bool) -> io::Result<()> {
    let footer = format!(
        "{} director{}, {} file{}",
        counts.dirs,
        if counts.dirs == 1 { "y" } else { "ies" },
        counts.files,
        if counts.files == 1 { "" } else { "s" }
    );
    let footer = if colors {
        Style::gray(&footer)
    } else {
        footer
    };
    writeln!(out)?;
    writeln!(out, "{}", footer)
}

fn walk_dir(
    dir: &Path,
    prefix: &str,
    level: usize,
    cfg: &Config,
    out: &mut impl Write,
    counts: &mut Counts,
) -> io::Result<()> {
    // depth cap: level 1 = immediate children of the root.
    if let Some(max) = cfg.max_depth {
        if level > max {
            return Ok(());
        }
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()), // skip unreadable directories silently
    };

    let mut items: Vec<(String, FileType)> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !cfg.show_hidden && ignore::is_hidden(&name) {
            continue;
        }
        let ft = match entry.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        if ft.is_dir() && ignore::is_ignored_dir(&name) {
            continue;
        }
        items.push((name, ft));
    }

    // Directories first, then files; case-insensitive alphabetical within each.
    items.sort_by(|a, b| match (a.1.is_dir(), b.1.is_dir()) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
    });

    let total = items.len();
    for (i, (name, ft)) in items.iter().enumerate() {
        let last = i + 1 == total;
        let connector = if last { "└── " } else { "├── " };
        let tree_part = format!("{}{}", prefix, connector);
        let tree_styled = if cfg.colors {
            Style::gray(&tree_part)
        } else {
            tree_part
        };

        let name_styled = render_name(ft, dir, name, cfg.colors);
        writeln!(out, "{}{}", tree_styled, name_styled)?;

        if ft.is_dir() {
            counts.dirs += 1;
            let cont = if last { "    " } else { "│   " };
            let new_prefix = format!("{}{}", prefix, cont);
            walk_dir(&dir.join(name), &new_prefix, level + 1, cfg, out, counts)?;
        } else {
            counts.files += 1;
        }
    }
    Ok(())
}

/// Build the display text for an entry (adding `-> target` for symlinks) and
/// optionally apply ANSI color. The `-> target` text is always shown; only the
/// color toggles.
fn render_name(ft: &FileType, dir: &Path, name: &str, colors: bool) -> String {
    if ft.is_symlink() {
        let target = fs::read_link(dir.join(name))
            .ok()
            .map(|t| format!(" -> {}", t.display()))
            .unwrap_or_default();
        let display = format!("{}{}", name, target);
        if colors {
            Style::symlink(&display)
        } else {
            display
        }
    } else if ft.is_dir() {
        if colors {
            Style::dir(name)
        } else {
            name.to_string()
        }
    } else {
        name.to_string()
    }
}
