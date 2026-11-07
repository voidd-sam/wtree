//! tree-rs — a fast, lightweight, colorful file/folder tree viewer (pure std).

mod cli;
mod ignore;
mod style;
mod walker;

use std::process::ExitCode;

const VERSION: &str = "tree-rs 0.1.0";

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    match cli::parse(argv) {
        cli::ParseResult::Help => {
            print_help();
            ExitCode::SUCCESS
        }
        cli::ParseResult::Version => {
            println!("{}", VERSION);
            ExitCode::SUCCESS
        }
        cli::ParseResult::Error(msg) => {
            eprintln!("tree: {}", msg);
            eprintln!("try 'tree --help' for more information");
            ExitCode::from(2)
        }
        cli::ParseResult::Ok(c) => {
            let cfg = walker::Config {
                root: c.path,
                max_depth: c.max_depth,
                show_hidden: c.show_hidden,
                colors: style::should_color(),
            };
            match walker::run(&cfg) {
                Ok(_) => ExitCode::SUCCESS,
                Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                    // Downstream consumer (e.g. `head`) closed the pipe early.
                    // This is expected, not an error — exit quietly.
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("tree: {}", e);
                    ExitCode::from(1)
                }
            }
        }
    }
}

fn print_help() {
    println!("{} — a fast, lightweight file/folder tree viewer", VERSION);
    println!();
    println!("USAGE:");
    println!("    tree [OPTIONS] [PATH]");
    println!();
    println!("ARGS:");
    println!("    <PATH>    Directory to list (default: current directory)");
    println!();
    println!("OPTIONS:");
    println!("    -d, --depth <N>    Maximum display depth (children of root are level 1)");
    println!("    -a, --all           Show hidden files and directories (dotfiles)");
    println!("    -h, --help          Print this help");
    println!("    -V, --version       Print version");
    println!();
    println!(
        "By default these directories are skipped: {}",
        ignore::ignored_dirs_help()
    );
    println!("Dotfiles (.git, .env, .vscode, ...) are hidden unless --all is passed.");
    println!("Color is auto-enabled on a TTY and disabled when piped/redirected,");
    println!("honoring the NO_COLOR and CLICOLOR_FORCE environment variables.");
}
