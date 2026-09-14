mod cli;

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let result = match cli::Cli::parse().command {
        cli::Command::Build(args) => parkforge::build(&args.project, &args.game_id),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
