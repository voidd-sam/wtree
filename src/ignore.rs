//! A small, std-only `.gitignore` engine.
//!
//! Supported: comments, blank lines, `!negation`, `dir/` (directories only),
//! leading `/` anchoring, `*`, `?`, `[abc]`, `[a-z]`, `[!abc]`, `\` escapes and
//! `**` (leading, middle and trailing). Last matching rule wins, and rules from
//! deeper `.gitignore` files override the ones above them.
//!
//! Not supported (yet): the global `core.excludesFile`.

use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------- glob tokens

enum Tok {
    Lit(u8),
    /// `?` - exactly one character (UTF-8 aware).
    Any,
    /// `*` - any run of characters inside one path segment.
    Star,
    Class(Class),
}

struct Class {
    negated: bool,
    ranges: Vec<(u8, u8)>,
}

impl Class {
    fn matches(&self, c: u8) -> bool {
        self.ranges.iter().any(|&(lo, hi)| lo <= c && c <= hi) != self.negated
    }
}

enum Seg {
    /// `**` - zero or more whole path segments.
    DoubleStar,
    Pat(Vec<Tok>),
}

fn parse_class(b: &[u8], start: usize) -> Option<(Class, usize)> {
    let mut i = start + 1;
    let mut negated = false;
    if matches!(b.get(i), Some(b'!') | Some(b'^')) {
        negated = true;
        i += 1;
    }
    let first = i;
    let mut ranges = Vec::new();
    loop {
        let c = *b.get(i)?;
        if c == b']' && i > first {
            return Some((Class { negated, ranges }, i + 1));
        }
        if i + 2 < b.len() && b[i + 1] == b'-' && b[i + 2] != b']' {
            ranges.push((c, b[i + 2]));
            i += 3;
        } else {
            ranges.push((c, c));
            i += 1;
        }
    }
}

fn compile(seg: &str) -> Vec<Tok> {
    let b = seg.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'*' => {
                if !matches!(out.last(), Some(Tok::Star)) {
                    out.push(Tok::Star);
                }
                i += 1;
            }
            b'?' => {
                out.push(Tok::Any);
                i += 1;
            }
            b'\\' if i + 1 < b.len() => {
                out.push(Tok::Lit(b[i + 1]));
                i += 2;
            }
            b'[' => match parse_class(b, i) {
                Some((class, next)) => {
                    out.push(Tok::Class(class));
                    i = next;
                }
                None => {
                    out.push(Tok::Lit(b'['));
                    i += 1;
                }
            },
            c => {
                out.push(Tok::Lit(c));
                i += 1;
            }
        }
    }
    out
}

/// Length in bytes of the UTF-8 character starting at `i`.
fn char_len(text: &[u8], i: usize) -> usize {
    let mut n = 1;
    while i + n < text.len() && (text[i + n] & 0xC0) == 0x80 {
        n += 1;
    }
    n
}

/// Match one path segment against compiled tokens. Iterative with single-star
/// backtracking, so it can never blow up exponentially.
fn seg_match(toks: &[Tok], text: &[u8]) -> bool {
    let (mut p, mut t) = (0usize, 0usize);
    let mut star: Option<(usize, usize)> = None;
    loop {
        if t >= text.len() {
            return toks[p..].iter().all(|k| matches!(k, Tok::Star));
        }
        if p < toks.len() {
            match &toks[p] {
                Tok::Star => {
                    star = Some((p + 1, t));
                    p += 1;
                    continue;
                }
                Tok::Lit(b) if *b == text[t] => {
                    p += 1;
                    t += 1;
                    continue;
                }
                Tok::Any => {
                    p += 1;
                    t += char_len(text, t);
                    continue;
                }
                Tok::Class(c) if c.matches(text[t]) => {
                    p += 1;
                    t += 1;
                    continue;
                }
                _ => {}
            }
        }
        match star {
            Some((sp, st)) => {
                let nt = st + char_len(text, st);
                star = Some((sp, nt));
                p = sp;
                t = nt;
            }
            None => return false,
        }
    }
}

fn match_segs(pat: &[Seg], path: &[&str]) -> bool {
    match pat.first() {
        None => path.is_empty(),
        Some(Seg::DoubleStar) => {
            if pat.len() == 1 {
                // trailing `/**` matches everything *inside*, not the dir itself
                return !path.is_empty();
            }
            (0..=path.len()).any(|k| match_segs(&pat[1..], &path[k..]))
        }
        Some(Seg::Pat(toks)) => match path.first() {
            Some(p) => seg_match(toks, p.as_bytes()) && match_segs(&pat[1..], &path[1..]),
            None => false,
        },
    }
}

// ---------------------------------------------------------------------- rules

struct Rule {
    negated: bool,
    dir_only: bool,
    segs: Vec<Seg>,
}

fn parse_line(raw: &str) -> Option<Rule> {
    let mut line = raw.trim_end_matches(|c| c == '\r' || c == '\n');
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    // trailing spaces are dropped unless escaped with a backslash
    let bytes = line.as_bytes();
    let mut end = bytes.len();
    while end > 0 && bytes[end - 1] == b' ' && !(end >= 2 && bytes[end - 2] == b'\\') {
        end -= 1;
    }
    line = &line[..end];

    let mut negated = false;
    if let Some(rest) = line.strip_prefix('!') {
        negated = true;
        line = rest;
    }
    let mut dir_only = false;
    if let Some(rest) = line.strip_suffix('/') {
        dir_only = true;
        line = rest;
    }
    // a slash anywhere (including a leading one) anchors the pattern
    let anchored = line.contains('/');
    let line = line.strip_prefix('/').unwrap_or(line);

    let mut segs: Vec<Seg> = line
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s == "**" {
                Seg::DoubleStar
            } else {
                Seg::Pat(compile(s))
            }
        })
        .collect();
    if segs.is_empty() {
        return None;
    }
    // no slash: the name may match at any depth, i.e. `name` == `**/name`
    if !anchored && !matches!(segs[0], Seg::DoubleStar) {
        segs.insert(0, Seg::DoubleStar);
    }
    Some(Rule {
        negated,
        dir_only,
        segs,
    })
}

struct RuleSet {
    /// Directory the `.gitignore` lives in; patterns are relative to it.
    base: PathBuf,
    rules: Vec<Rule>,
}

impl RuleSet {
    fn from_text(base: PathBuf, text: &str) -> RuleSet {
        RuleSet {
            base,
            rules: text.lines().filter_map(parse_line).collect(),
        }
    }

    /// `Some(true)` = ignored, `Some(false)` = re-included, `None` = no rule matched.
    fn decide(&self, segs: &[&str], is_dir: bool) -> Option<bool> {
        for rule in self.rules.iter().rev() {
            if rule.dir_only && !is_dir {
                continue;
            }
            if match_segs(&rule.segs, segs) {
                return Some(!rule.negated);
            }
        }
        None
    }
}

// ---------------------------------------------------------------------- stack

/// The `.gitignore` files that apply to the directory currently being walked,
/// outermost first. The walker pushes on entering a directory and pops on leaving.
pub struct Stack {
    sets: Vec<RuleSet>,
}

impl Stack {
    /// Build the starting stack for `root` (must be an absolute path). If `root`
    /// is inside a git repo, the repo's `.git/info/exclude` and the `.gitignore`
    /// of every directory above `root` are loaded, so `wtree src/` behaves
    /// exactly like `wtree` run from the repo root.
    pub fn for_root(root: &Path) -> Stack {
        let mut stack = Stack { sets: Vec::new() };
        let repo = match root.ancestors().find(|a| a.join(".git").exists()) {
            Some(r) => r,
            None => return stack,
        };
        stack.push_file(repo, &repo.join(".git").join("info").join("exclude"));
        let mut chain: Vec<&Path> = root
            .ancestors()
            .skip(1)
            .take_while(|a| a.starts_with(repo))
            .collect();
        chain.reverse();
        for dir in chain {
            stack.push_dir(dir);
        }
        stack
    }

    fn push_file(&mut self, base: &Path, file: &Path) -> bool {
        let bytes = match fs::read(file) {
            Ok(b) => b,
            Err(_) => return false,
        };
        let set = RuleSet::from_text(base.to_path_buf(), &String::from_utf8_lossy(&bytes));
        if set.rules.is_empty() {
            return false;
        }
        self.sets.push(set);
        true
    }

    /// Load `dir/.gitignore` if present. Returns `true` if something was pushed
    /// (the caller must then call `pop`).
    pub fn push_dir(&mut self, dir: &Path) -> bool {
        self.push_file(dir, &dir.join(".gitignore"))
    }

    pub fn pop(&mut self) {
        self.sets.pop();
    }

    /// `true` when no ignore rules apply at all (caller falls back to defaults).
    pub fn is_empty(&self) -> bool {
        self.sets.is_empty()
    }

    /// Is the absolute `path` ignored? Deepest `.gitignore` wins; inside one
    /// file, the last matching line wins.
    pub fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        for set in self.sets.iter().rev() {
            let rel = match path.strip_prefix(&set.base) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let rel_s = rel.to_string_lossy();
            let segs: Vec<&str> = rel_s
                .split(std::path::MAIN_SEPARATOR)
                .filter(|s| !s.is_empty())
                .collect();
            if segs.is_empty() {
                continue;
            }
            if let Some(verdict) = set.decide(&segs, is_dir) {
                return verdict;
            }
        }
        false
    }
}

// ---------------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;

    fn ig(rules: &str, path: &str, is_dir: bool) -> bool {
        let set = RuleSet::from_text(PathBuf::new(), rules);
        let segs: Vec<&str> = path.split('/').collect();
        set.decide(&segs, is_dir).unwrap_or(false)
    }

    #[test]
    fn dir_only_rule() {
        assert!(ig("target/", "target", true));
        assert!(!ig("target/", "target", false));
        assert!(ig("target/", "crates/a/target", true));
    }

    #[test]
    fn name_matches_any_depth() {
        assert!(ig("*.log", "a.log", false));
        assert!(ig("*.log", "x/y/z.log", false));
        assert!(!ig("*.log", "x/y/z.txt", false));
    }

    #[test]
    fn anchored() {
        assert!(ig("/build", "build", true));
        assert!(!ig("/build", "src/build", true));
        assert!(ig("doc/frotz", "doc/frotz", false));
        assert!(!ig("doc/frotz", "a/doc/frotz", false));
    }

    #[test]
    fn negation_last_wins() {
        let r = "*.log\n!keep.log\n";
        assert!(ig(r, "a.log", false));
        assert!(!ig(r, "keep.log", false));
    }

    #[test]
    fn double_star() {
        assert!(ig("**/foo", "a/b/foo", false));
        assert!(ig("a/**/b", "a/b", false));
        assert!(ig("a/**/b", "a/x/y/b", false));
        assert!(ig("foo/**", "foo/bar", false));
        assert!(!ig("foo/**", "foo", true));
    }

    #[test]
    fn classes_and_question_mark() {
        assert!(ig("file[0-9].txt", "file7.txt", false));
        assert!(!ig("file[0-9].txt", "filex.txt", false));
        assert!(ig("file[!0-9].txt", "filex.txt", false));
        assert!(ig("a?c", "abc", false));
        assert!(ig("a?c", "a\u{e9}c", false));
        assert!(!ig("a?c", "ac", false));
    }

    #[test]
    fn comments_blanks_escapes() {
        assert!(!ig("# comment\n\n", "comment", false));
        assert!(ig("\\#file", "#file", false));
        assert!(ig("\\!important", "!important", false));
    }

    #[test]
    fn star_backtracking() {
        assert!(ig("*a*a*b", "xaxaxb", false));
        assert!(!ig("*a*a*b", "xaxaxc", false));
    }
}
