mod cli;

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let result = match cli::Cli::parse().command {
        cli::Command::Overlay(args) => match args.command {
            cli::OverlayCommand::Create(args) => {
                eprintln!("Creating source overlay…");
                let result = parkforge::create_source_overlay(&args.project, &args.game_id);
                if result.is_ok() {
                    eprintln!("Source overlay created");
                }
                result
            }
        },
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
