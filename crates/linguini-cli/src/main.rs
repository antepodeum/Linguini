use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    match linguini_cli::run(env::args().skip(1), env::current_dir()) {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
