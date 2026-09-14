mod cli;

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let result = match cli::Cli::parse().command {
        cli::Command::Build(args) => {
            parkforge::build(&args.project, &args.game_id, |progress| match progress {
                parkforge::BuildProgress::CopyingOriginal => eprintln!("Copying original…"),
                parkforge::BuildProgress::Operations { completed, total } => {
                    eprintln!("Processing operations: {completed} / {total}");
                }
                parkforge::BuildProgress::Committing => eprintln!("Committing build…"),
            })
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
