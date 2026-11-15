//! Command-line argument parsing for `wtree`.
//!
//! Hand-rolled (no clap) to keep the binary tiny and the build fast.
//! Supports combined short flags (`-ai`), attached values (`-d2`, `-L2`,
//! `--depth=2`) and `--` to end option parsing.

use std::path::PathBuf;

pub struct Cli {
    pub path: PathBuf,
    pub max_depth: Option<usize>,
    pub show_hidden: bool,
    pub no_ignore: bool,
    pub icons: bool,
}

pub enum ParseResult {
    Ok(Cli),
    Help,
    Version,
    Error(String),
}

fn parse_depth(v: &str) -> Result<usize, String> {
    v.parse::<usize>()
        .map_err(|_| format!("invalid depth '{}' (expected a number)", v))
}

pub fn parse(argv: Vec<String>) -> ParseResult {
    let mut path: Option<PathBuf> = None;
    let mut max_depth: Option<usize> = None;
    let mut show_hidden = false;
    let mut no_ignore = false;
    let mut icons = false;
    let mut want_help = false;
    let mut want_version = false;
    let mut only_paths = false; // set after `--`

    let mut i = 1;
    while i < argv.len() {
        let arg = argv[i].as_str();

        if only_paths || arg == "-" || !arg.starts_with('-') {
            if path.is_none() {
                path = Some(PathBuf::from(arg));
            } else {
                return ParseResult::Error(format!("unexpected argument '{}'", arg));
            }
            i += 1;
            continue;
        }

        if let Some(long) = arg.strip_prefix("--") {
            match long {
                "" => only_paths = true,
                "help" => want_help = true,
                "version" => want_version = true,
                "all" => show_hidden = true,
                "no-ignore" => no_ignore = true,
                "icons" => icons = true,
                "depth" => {
                    i += 1;
                    match argv.get(i) {
                        Some(v) => match parse_depth(v) {
                            Ok(n) => max_depth = Some(n),
                            Err(e) => return ParseResult::Error(e),
                        },
                        None => return ParseResult::Error("missing value for --depth".into()),
                    }
                }
                _ => match long.strip_prefix("depth=") {
                    Some(v) => match parse_depth(v) {
                        Ok(n) => max_depth = Some(n),
                        Err(e) => return ParseResult::Error(e),
                    },
                    None => return ParseResult::Error(format!("unknown flag '{}'", arg)),
                },
            }
            i += 1;
            continue;
        }

        // short flags, possibly combined: -ai, -d2, -L 3
        let chars: Vec<char> = arg[1..].chars().collect();
        let mut j = 0;
        while j < chars.len() {
            match chars[j] {
                'h' => want_help = true,
                'V' | 'v' => want_version = true,
                'a' => show_hidden = true,
                'i' => no_ignore = true,
                'd' | 'L' => {
                    let attached: String = chars[j + 1..].iter().collect();
                    let value = if !attached.is_empty() {
                        attached
                    } else {
                        i += 1;
                        match argv.get(i) {
                            Some(v) => v.clone(),
                            None => {
                                return ParseResult::Error(format!(
                                    "missing value for -{}",
                                    chars[j]
                                ))
                            }
                        }
                    };
                    match parse_depth(&value) {
                        Ok(n) => max_depth = Some(n),
                        Err(e) => return ParseResult::Error(e),
                    }
                    break; // the value consumed the rest of this argument
                }
                c => return ParseResult::Error(format!("unknown flag '-{}'", c)),
            }
            j += 1;
        }
        i += 1;
    }

    if want_help {
        return ParseResult::Help;
    }
    if want_version {
        return ParseResult::Version;
    }

    ParseResult::Ok(Cli {
        path: path.unwrap_or_else(|| PathBuf::from(".")),
        max_depth,
        show_hidden,
        no_ignore,
        icons,
    })
}
