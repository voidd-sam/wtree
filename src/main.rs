//! wtree - a fast, lightweight, colorful file/folder tree viewer.

mod cli;
mod gitignore;
mod ignore;
mod style;
mod walker;

use std::process::ExitCode;

// Everything below comes from Cargo.toml, so name and version can never drift.
const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    match cli::parse(argv) {
        cli::ParseResult::Help => {
            print_help();
            ExitCode::SUCCESS
        }
        cli::ParseResult::Version => {
            println!("{} {}", NAME, VERSION);
            ExitCode::SUCCESS
        }
        cli::ParseResult::Error(msg) => {
            eprintln!("{}: {}", NAME, msg);
            eprintln!("try '{} --help' for more information", NAME);
            ExitCode::from(2)
        }
        cli::ParseResult::Ok(c) => {
            let icons_env = std::env::var_os("WTREE_ICONS")
                .map(|v| !v.is_empty() && v != "0")
                .unwrap_or(false);
            let cfg = walker::Config {
                root: c.path,
                max_depth: c.max_depth,
                show_hidden: c.show_hidden,
                no_ignore: c.no_ignore,
                icons: c.icons || icons_env,
                colors: style::should_color(),
            };
            match walker::run(&cfg) {
                Ok(_) => ExitCode::SUCCESS,
                Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                    // Downstream consumer (e.g. `head`) closed the pipe early.
                    // Expected, not an error - exit quietly.
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{}: {}", NAME, e);
                    ExitCode::from(1)
                }
            }
        }
    }
}

fn print_help() {
    println!(
        "{} {} - a fast, lightweight file/folder tree viewer",
        NAME, VERSION
    );
    println!();
    println!("USAGE:");
    println!("    {} [OPTIONS] [PATH]", NAME);
    println!();
    println!("ARGS:");
    println!("    <PATH>    Directory to list (default: current directory)");
    println!();
    println!("OPTIONS:");
    println!("    -d, -L, --depth <N>   Maximum display depth (children of root are level 1)");
    println!("    -a, --all             Show hidden files and directories (dotfiles)");
    println!("    -i, --no-ignore       Show ignored entries too (dimmed) instead of hiding them");
    println!("        --icons           Show Nerd Font icons (or set WTREE_ICONS=1)");
    println!("    -h, --help            Print this help");
    println!("    -V, -v, --version     Print version");
    println!();
    println!("IGNORING:");
    println!("    .gitignore files are honored (nested ones too, plus .git/info/exclude),");
    println!(
        "    even when you run {} from a subfolder of the repository.",
        NAME
    );
    println!(
        "    Without any .gitignore these directories are skipped: {}",
        ignore::ignored_dirs_help()
    );
    println!("    The .git directory stays hidden unless you pass --no-ignore.");
    println!();
    println!("ENVIRONMENT:");
    println!("    NO_COLOR         Disable color, even on a terminal");
    println!("    CLICOLOR_FORCE   Force color, even when piped");
    println!("    WTREE_ICONS      Enable icons by default");
}
