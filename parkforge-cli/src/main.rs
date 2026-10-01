mod cli;
mod diagnostic;

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    match cli::Cli::parse().command {
        cli::Command::Overlay(args) => match args.command {
            cli::OverlayCommand::Create(args) => match run_create_overlay(&args) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::FAILURE
                }
            },
        },
        cli::Command::Build(args) => match run_build(&args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::FAILURE
            }
        },
        cli::Command::Check(args) => run_check(&args),
        cli::Command::Extract(args) => match run_extract(&args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(parkforge::ExtractError::Identify(
                parkforge::IdentifyError::UnsupportedNkitIso { input },
            )) => {
                eprintln!(
                    "error: NKit ISO {} is not supported; use a clean standard Wii ISO instead",
                    input.display()
                );
                ExitCode::FAILURE
            }
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::FAILURE
            }
        },
        cli::Command::Rebuild(args) => match run_rebuild(args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::FAILURE
            }
        },
    }
}

fn run_create_overlay(args: &cli::CreateOverlayArgs) -> Result<(), parkforge::OverlayError> {
    eprintln!("Creating source overlay…");
    parkforge::create_source_overlay(&args.project, &args.game_id)?;
    eprintln!("Source overlay created");
    Ok(())
}

fn run_build(args: &cli::BuildArgs) -> Result<(), parkforge::BuildError> {
    parkforge::build(
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
    )
}

fn run_check(args: &cli::CheckArgs) -> ExitCode {
    match parkforge::check(&args.project, |game_id, diagnostic| {
        eprintln!("{game_id}:");
        diagnostic::render(&diagnostic);
    }) {
        Ok(report) => {
            for revision in report.revisions() {
                match revision.result() {
                    Ok(report) => {
                        for error in report.errors() {
                            eprintln!("error [{}]: {error}", revision.game_id());
                        }
                    }
                    Err(error) => {
                        eprintln!("error [{}]: {error}", revision.game_id());
                    }
                }
            }
            if report.is_success() {
                eprintln!("Project check succeeded");
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_extract(args: &cli::ExtractArgs) -> Result<(), parkforge::ExtractError> {
    let mut last_disc_percent = None;
    parkforge::extract(&args.project, &args.input_iso, |progress| match progress {
        parkforge::ExtractionProgress::Disc(progress) => {
            let percent = if progress.total() == 0 {
                100
            } else {
                progress.completed() * 100 / progress.total()
            };
            if last_disc_percent != Some(percent) {
                eprintln!("Extracting ISO: {percent}%");
                last_disc_percent = Some(percent);
            }
        }
        parkforge::ExtractionProgress::Archives {
            completed,
            discovered,
        } => {
            eprintln!("Extracting archives: {completed} / {discovered}");
        }
    })
    .map(|game_id| {
        eprintln!("Extracted game revision {game_id}");
    })
}

fn run_rebuild(args: cli::RebuildArgs) -> Result<(), parkforge::RebuildError> {
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
