// use ryan::environment::Environment;
// use ryan::parser;

// fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
//     let file = std::env::args().into_iter().collect::<Vec<_>>()[1].clone();
//     let ryans = std::fs::read_to_string(&file)?;

//     for ryan in ryans.split("---") {
//         let parsed = parser::parse(ryan)?;
//         println!("> {}", parsed);
//         match parser::eval(Environment::new(Some(&file)), &parsed) {
//             Ok(ok) => println!("ok= {ok}",),
//             Err(err) => println!("err= {err}",),
//         }
//     }

//     Ok(())
// }

use std::io::Write;

use clap::Parser;
use termcolor::{ColorChoice, StandardStream};

/// The Ryan configuration language CLI.
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
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
    let output: serde_json::Value = match cli.file.as_str() {
        "-" => ryan::from_reader_with_env(&env, std::io::stdin().lock())?,
        path => ryan::from_path_with_env(&env, path)?,
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
