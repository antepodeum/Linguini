use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    match linguini_cli::run(env::args().skip(1), env::current_dir()) {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            if error.use_stdout() {
                print!("{error}");
            } else {
                eprintln!("{error}");
            }

            if error.exit_code() == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    }
}
