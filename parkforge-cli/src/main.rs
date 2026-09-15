mod cli;
mod diagnostic;

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
        cli::Command::Build(args) => parkforge::build(
            &args.project,
            &args.game_id,
            |progress| match progress {
                parkforge::BuildProgress::CopyingOriginal => eprintln!("Copying original…"),
                parkforge::BuildProgress::Operations { completed, total } => {
                    eprintln!("Processing operations: {completed} / {total}");
                }
                parkforge::BuildProgress::Committing => eprintln!("Committing build…"),
            },
            |diagnostic| diagnostic::render(&diagnostic),
        ),
        cli::Command::Rebuild(args) => {
            let output = args.output_iso.unwrap_or_else(|| {
                args.project
                    .join("dist")
                    .join(format!("{}.iso", args.game_id))
            });
            let mut last_disc_percent = None;
            parkforge::rebuild(
                &args.project,
                &args.game_id,
                &output,
                |progress| match progress {
                    parkforge::RebuildProgress::Archives { completed, total } => {
                        eprintln!("Repacking archives: {completed} / {total}");
                    }
                    parkforge::RebuildProgress::Disc(progress) => {
                        let percent = if progress.total() == 0 {
                            100
                        } else {
                            progress.completed() * 100 / progress.total()
                        };
                        if last_disc_percent != Some(percent) {
                            eprintln!("Building ISO: {percent}%");
                            last_disc_percent = Some(percent);
                        }
                    }
                },
            )
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
