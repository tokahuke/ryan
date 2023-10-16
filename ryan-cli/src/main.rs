use std::io::Write;

use clap::Parser;
use termcolor::{ColorChoice, StandardStream};

/// The Ryan configuration language CLI.
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// If set, will interpret the FILE not as a filename, but as actual Ryan code.
    #[clap(long, short)]
    command: bool,
    /// The name of the file to be executed. Pass `-` to read from standard input.
    file: String,
    /// Hermetic mode: disables all imports.
    #[clap(long)]
    hermetic: bool,
    /// Disables fancy color output. This app detects `tty`s, so you don't need to
    /// worry about setting this option when piping.
    #[clap(long)]
    no_color: bool,
}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    // Config:
    let env = if cli.hermetic {
        ryan::Environment::builder()
            .import_loader(ryan::environment::NoImport)
            .build()
    } else {
        ryan::Environment::builder().build()
    };

    // Eval:
    let output: serde_json::Value = match (cli.command, cli.file.as_str()) {
        (false, "-") => ryan::from_reader_with_env(&env, std::io::stdin().lock())?,
        (false, path) => ryan::from_path_with_env(&env, path)?,
        (true, code) => ryan::from_str_with_env(&env, code)?,
    };

    // Print:
    let stdout = StandardStream::stdout(if cli.no_color || atty::isnt(atty::Stream::Stdout) {
        ColorChoice::Never
    } else {
        ColorChoice::Auto
    });
    termcolor_json::to_writer(&mut stdout.lock(), &output)?;
    stdout.lock().write_all(b"\n")?;

    Ok(())
}
