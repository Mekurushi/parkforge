use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use parkforge_types::GameId;

#[derive(Parser)]
#[command(name = "parkforge")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    Build(BuildArgs),
    Overlay(OverlayArgs),
    Rebuild(RebuildArgs),
}

#[derive(Args)]
pub(crate) struct OverlayArgs {
    #[command(subcommand)]
    pub command: OverlayCommand,
}

#[derive(Subcommand)]
pub(crate) enum OverlayCommand {
    Create(CreateOverlayArgs),
}

#[derive(Args)]
pub(crate) struct CreateOverlayArgs {
    pub game_id: GameId,
    #[arg(long, default_value = ".")]
    pub project: PathBuf,
}

#[derive(Args)]
pub(crate) struct BuildArgs {
    pub game_id: GameId,
    #[arg(long, default_value = ".")]
    pub project: PathBuf,
}

#[derive(Args)]
pub(crate) struct RebuildArgs {
    pub game_id: GameId,
    pub output_iso: Option<PathBuf>,
    #[arg(long, default_value = ".")]
    pub project: PathBuf,
}
