use std::env;
use std::path::PathBuf;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let source_dir = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let output_dir = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let check = match args.next() {
        None => false,
        Some(flag) if flag == "--check" => true,
        Some(_) => return Err(usage()),
    };
    if args.next().is_some() {
        return Err(usage());
    }

    linguini_cldr_macros::generate(&source_dir, &output_dir, check)
}

fn usage() -> String {
    "usage: generate_cldr <pinned-source-dir> <generated-output-dir> [--check]".to_owned()
}
