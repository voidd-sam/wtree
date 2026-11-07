//! Command-line argument parsing for `tree`.
//!
//! Hand-rolled (no clap) to keep the binary tiny and the build fast.

use std::path::PathBuf;

pub struct Cli {
    pub path: PathBuf,
    pub max_depth: Option<usize>,
    pub show_hidden: bool,
}

pub enum ParseResult {
    Ok(Cli),
    Help,
    Version,
    Error(String),
}

pub fn parse(argv: Vec<String>) -> ParseResult {
    let mut path: Option<PathBuf> = None;
    let mut max_depth: Option<usize> = None;
    let mut show_hidden = false;
    let mut want_help = false;
    let mut want_version = false;

    let mut i = 1;
    while i < argv.len() {
        let arg = &argv[i];
        match arg.as_str() {
            "-h" | "--help" => want_help = true,
            "-V" | "--version" => want_version = true,
            "-a" | "--all" => show_hidden = true,
            "-d" | "--depth" => {
                i += 1;
                if i >= argv.len() {
                    return ParseResult::Error(format!("missing value for {}", arg));
                }
                match argv[i].parse::<usize>() {
                    Ok(n) => max_depth = Some(n),
                    Err(_) => {
                        return ParseResult::Error(format!("invalid depth '{}' (expected a number)", argv[i]))
                    }
                }
            }
            s if s.starts_with("--depth=") => match s["--depth=".len()..].parse::<usize>() {
                Ok(n) => max_depth = Some(n),
                Err(_) => return ParseResult::Error(format!("invalid depth in '{}'", s)),
            },
            s if s.starts_with("--") => return ParseResult::Error(format!("unknown flag '{}'", s)),
            s if s.starts_with('-') && s.len() > 1 => {
                return ParseResult::Error(format!("unknown flag '{}'", s))
            }
            _ => {
                if path.is_none() {
                    path = Some(PathBuf::from(arg));
                } else {
                    return ParseResult::Error(format!("unexpected argument '{}'", arg));
                }
            }
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
    })
}
