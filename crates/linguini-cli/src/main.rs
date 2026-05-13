use clap::error::ErrorKind;
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    match linguini_cli::run(env::args().skip(1), env::current_dir()) {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(linguini_cli::CliError::Args(error)) => match error.kind() {
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                print!("{error}");
                ExitCode::SUCCESS
            }
            _ => {
                eprint!("{error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
