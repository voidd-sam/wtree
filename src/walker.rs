//! Directory walking + tree rendering.
//!
//! Fast and allocation-light:
//! - `read_dir` and the kernel-provided `d_type` (no `stat` per entry).
//! - Output is streamed to a locked stdout as the walk happens.
//! - `.gitignore` files are loaded lazily, only for directories actually entered.

use std::fs::{self, FileType};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::gitignore::Stack;
use crate::ignore;
use crate::style::{self, Style};

pub struct Config {
    pub root: PathBuf,
    pub max_depth: Option<usize>,
    pub show_hidden: bool,
    /// Show entries that would normally be ignored (dimmed) instead of hiding them.
    pub no_ignore: bool,
    pub icons: bool,
    pub colors: bool,
}

#[derive(Default)]
pub struct Counts {
    pub dirs: usize,
    pub files: usize,
    /// Entries hidden by ignore rules (not counting dotfiles).
    pub ignored: usize,
}

struct Item {
    name: String,
    ft: FileType,
    ignored: bool,
}

struct Walker<'a, W: Write> {
    cfg: &'a Config,
    out: W,
    stack: Stack,
    counts: Counts,
}

/// Print the tree for `cfg.root` and the summary footer. Returns entry counts.
pub fn run(cfg: &Config) -> io::Result<Counts> {
    let stdout = io::stdout();

    let access_err = |e: io::Error| {
        io::Error::new(
            e.kind(),
            format!("cannot access '{}': {}", cfg.root.display(), e),
        )
    };
    let meta = fs::metadata(&cfg.root).map_err(access_err)?;
    // Walk the canonical path so ignore rules from parent directories line up.
    let abs = fs::canonicalize(&cfg.root).map_err(access_err)?;

    let mut w = Walker {
        cfg,
        out: stdout.lock(),
        stack: Stack::for_root(&abs),
        counts: Counts::default(),
    };

    let mut label = cfg.root.display().to_string();
    if cfg.icons && meta.is_dir() {
        label = format!("{} {}", style::ICON_DIR, label);
    }
    let label = if cfg.colors {
        Style::dir(&label)
    } else {
        label
    };
    writeln!(w.out, "{}", label)?;

    if meta.is_dir() {
        w.walk(&abs, "", 1, false)?;
    }
    w.footer()?;
    Ok(w.counts)
}

impl<W: Write> Walker<'_, W> {
    fn footer(&mut self) -> io::Result<()> {
        let c = &self.counts;
        let mut text = format!(
            "{} director{}, {} file{}",
            c.dirs,
            if c.dirs == 1 { "y" } else { "ies" },
            c.files,
            if c.files == 1 { "" } else { "s" }
        );
        if c.ignored > 0 {
            text.push_str(&format!(", {} ignored (-i to show)", c.ignored));
        }
        let text = if self.cfg.colors {
            Style::gray(&text)
        } else {
            text
        };
        writeln!(self.out)?;
        writeln!(self.out, "{}", text)
    }

    fn walk(
        &mut self,
        dir: &Path,
        prefix: &str,
        level: usize,
        parent_ignored: bool,
    ) -> io::Result<()> {
        // depth cap: level 1 = immediate children of the root.
        if let Some(max) = self.cfg.max_depth {
            if level > max {
                return Ok(());
            }
        }
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(()), // unreadable directory: skip silently
        };

        let pushed = self.stack.push_dir(dir);
        let result = self.walk_entries(entries, dir, prefix, level, parent_ignored);
        if pushed {
            self.stack.pop();
        }
        result
    }

    fn walk_entries(
        &mut self,
        entries: fs::ReadDir,
        dir: &Path,
        prefix: &str,
        level: usize,
        parent_ignored: bool,
    ) -> io::Result<()> {
        let cfg = self.cfg;

        let mut items: Vec<Item> = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !cfg.show_hidden && ignore::is_hidden(&name) {
                continue;
            }
            let ft = match entry.file_type() {
                Ok(t) => t,
                Err(_) => continue,
            };
            let is_dir = ft.is_dir();

            let ignored = parent_ignored
                || if self.stack.is_empty() {
                    // no .gitignore anywhere: fall back to the built-in list
                    is_dir && ignore::is_ignored_dir(&name)
                } else {
                    (is_dir && name == ".git") || self.stack.is_ignored(&dir.join(&name), is_dir)
                };

            if ignored && !cfg.no_ignore {
                self.counts.ignored += 1;
                continue;
            }
            items.push(Item { name, ft, ignored });
        }

        // Directories first, then files; case-insensitive alphabetical within each.
        items.sort_by_cached_key(|i| (!i.ft.is_dir(), i.name.to_lowercase()));

        let total = items.len();
        for (i, item) in items.iter().enumerate() {
            let last = i + 1 == total;
            let connector = if last { "└── " } else { "├── " };
            let tree_part = format!("{}{}", prefix, connector);
            let tree_part = if cfg.colors {
                Style::gray(&tree_part)
            } else {
                tree_part
            };

            let label = render_name(item, dir, cfg);
            writeln!(self.out, "{}{}", tree_part, label)?;

            if item.ft.is_dir() {
                self.counts.dirs += 1;
                let cont = if last { "    " } else { "│   " };
                let new_prefix = format!("{}{}", prefix, cont);
                self.walk(&dir.join(&item.name), &new_prefix, level + 1, item.ignored)?;
            } else {
                self.counts.files += 1;
            }
        }
        Ok(())
    }
}

/// Build the display text for an entry (icon, name, `-> target`) and color it.
/// The `-> target` text is always shown; only the color and icon toggle.
fn render_name(item: &Item, dir: &Path, cfg: &Config) -> String {
    let ft = &item.ft;
    let name = item.name.as_str();

    let (kind, icon) = if ft.is_symlink() {
        (style::Kind::Plain, style::ICON_SYMLINK)
    } else if ft.is_dir() {
        (style::Kind::Plain, style::ICON_DIR)
    } else {
        style::classify(name)
    };

    let mut text = if cfg.icons {
        format!("{} {}", icon, name)
    } else {
        name.to_string()
    };
    if ft.is_symlink() {
        if let Ok(t) = fs::read_link(dir.join(name)) {
            text.push_str(&format!(" -> {}", t.display()));
        }
    }

    if !cfg.colors {
        return text;
    }
    if item.ignored {
        Style::ignored(&text)
    } else if ft.is_symlink() {
        Style::symlink(&text)
    } else if ft.is_dir() {
        Style::dir(&text)
    } else {
        match kind.color() {
            Some(code) => style::paint(code, &text),
            None => text,
        }
    }
}
