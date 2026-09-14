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
}

#[derive(Args)]
pub(crate) struct BuildArgs {
    pub game_id: GameId,
    #[arg(long, default_value = ".")]
    pub project: PathBuf,
}
