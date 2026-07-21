use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use parkforge_project::{BuildConfig, GameId, Project, ProjectConfig, ProjectMetadata};

#[derive(Debug, thiserror::Error)]
enum CliError {
    #[error(transparent)]
    Build(#[from] parkforge_build::Error),
    #[error(transparent)]
    Project(#[from] parkforge_project::Error),
    #[error(transparent)]
    Extraction(#[from] parkforge_extractor::Error),
    #[error("extracted ISO is for game ID {found}, but --game-id {expected} was requested")]
    GameIdMismatch { expected: GameId, found: GameId },
    #[error("manifest is for game ID {found}, but was loaded for {expected}")]
    ManifestGameIdMismatch { expected: GameId, found: GameId },
    #[error(transparent)]
    GameId(#[from] parkforge_model::GameIdError),
}

#[derive(Parser)]
#[command(name = "parkforge", version, about = "pokepark mod tooling")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(subcommand)]
    Project(ProjectCommand),
    Extract {
        path: PathBuf,
        iso: PathBuf,
        #[arg(long)]
        game_id: Option<String>,
    },
    #[command(subcommand)]
    Overlay(OverlayCommand),
    Build {
        path: PathBuf,
        game_id: String,
    },
    Rebuild {
        path: PathBuf,
        game_id: String,
    },
}

#[derive(Subcommand)]
enum ProjectCommand {
    Create {
        path: PathBuf,
        #[arg(long)]
        name: String,
        #[arg(long)]
        version: String,
    },
    Validate {
        path: PathBuf,
    },
    #[command(subcommand)]
    Game(GameCommand),
    Games {
        path: PathBuf,
    },
}

#[derive(Subcommand)]
enum GameCommand {
    Add {
        path: PathBuf,
        game_id: String,
        #[arg(long)]
        label: Option<String>,
    },
}

#[derive(Subcommand)]
enum OverlayCommand {
    Create { path: PathBuf, game_id: String },
}

fn main() -> ExitCode {
    match run(Cli::parse().command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(command: Command) -> Result<(), CliError> {
    match command {
        Command::Project(command) => run_project(command),
        Command::Extract { path, iso, game_id } => run_extract(&path, &iso, game_id),
        Command::Overlay(command) => run_overlay(command),
        Command::Build { path, game_id } => run_build(&path, game_id),
        Command::Rebuild { path, game_id } => run_rebuild(&path, game_id),
    }
}

fn run_project(command: ProjectCommand) -> Result<(), CliError> {
    match command {
        ProjectCommand::Create {
            path,
            name,
            version,
        } => run_project_create(&path, name, version),
        ProjectCommand::Validate { path } => run_project_validate(&path),
        ProjectCommand::Game(command) => run_project_game(command),
        ProjectCommand::Games { path } => run_project_games(&path),
    }
}

fn run_project_create(path: &Path, name: String, version: String) -> Result<(), CliError> {
    let config = ProjectConfig {
        project: ProjectMetadata { name, version },
        games: Vec::new(),
        build: BuildConfig::default(),
    };
    Project::create(path, config)?;
    println!("created project at {}", path.display());
    Ok(())
}

fn run_project_validate(path: &Path) -> Result<(), CliError> {
    // `Project::open` validates
    Project::open(path)?;
    println!("project at {} is valid", path.display());
    Ok(())
}

fn run_project_games(path: &Path) -> Result<(), CliError> {
    let project = Project::open(path)?;
    for game_id in project.games() {
        println!("{game_id}");
    }
    Ok(())
}
fn run_project_game(command: GameCommand) -> Result<(), CliError> {
    match command {
        GameCommand::Add {
            path,
            game_id,
            label,
        } => {
            let mut project = Project::open(&path)?;
            let game_id = GameId::new(game_id)?;
            project.add_game(game_id.clone(), label)?;
            println!("registered game ID {game_id} in {}", path.display());
            Ok(())
        }
    }
}

fn run_extract(path: &Path, iso: &Path, game_id: Option<String>) -> Result<(), CliError> {
    let project = Project::open(path)?;
    let expected_game_id = game_id.map(GameId::new).transpose()?;
    let detected_game_id = parkforge_extractor::read_game_id(iso)?;

    if let Some(expected) = expected_game_id {
        if expected != detected_game_id {
            return Err(CliError::GameIdMismatch {
                expected,
                found: detected_game_id,
            });
        }
    }
    project.require_registered_game(&detected_game_id)?;

    let manifest = parkforge_extractor::extract(iso, &project.original_root())?;
    let game_id = manifest.game_id;

    println!(
        "extracted game ID {} into {}",
        game_id,
        project.original(&game_id).root().display()
    );
    Ok(())
}

fn run_overlay(command: OverlayCommand) -> Result<(), CliError> {
    match command {
        OverlayCommand::Create { path, game_id } => run_overlay_create(&path, game_id),
    }
}

fn run_overlay_create(path: &Path, game_id: String) -> Result<(), CliError> {
    let project = Project::open(path)?;
    let game_id = GameId::new(game_id)?;

    let manifest_path = project.require_original(&game_id)?.manifest_path();
    let manifest = parkforge_extractor::Manifest::load(&manifest_path)?;
    if manifest.game_id != game_id {
        return Err(CliError::ManifestGameIdMismatch {
            expected: game_id,
            found: manifest.game_id,
        });
    }

    project.create_source_overlay(&game_id, manifest.directories())?;

    println!(
        "created source overlay for {game_id} at {}",
        project.source(&game_id).root().display()
    );
    Ok(())
}

fn run_build(path: &Path, game_id: String) -> Result<(), CliError> {
    let project = Project::open(path)?;
    let game_id = GameId::new(game_id)?;
    let original = project.require_original(&game_id)?;
    let source = project.source(&game_id);
    let build = project.build(&game_id);
    parkforge_build::build(&parkforge_build::BuildRequest {
        game_id: game_id.as_str(),
        original_root: original.root(),
        source_root: source.root(),
        build_root: build.root(),
        config: project.build_config(),
    })?;
    println!("built game ID {game_id} into {}", build.root().display());
    Ok(())
}

fn run_rebuild(path: &Path, game_id: String) -> Result<(), CliError> {
    let project = Project::open(path)?;
    let game_id = GameId::new(game_id)?;
    let original = project.require_original(&game_id)?;
    let build = project.require_build(&game_id)?;
    let manifest = parkforge_extractor::Manifest::load(&original.manifest_path())?;
    if manifest.game_id != game_id {
        return Err(CliError::ManifestGameIdMismatch {
            expected: game_id,
            found: manifest.game_id,
        });
    }
    let output = project.dist(&game_id).iso_path();
    parkforge_extractor::rebuild(build.root(), &manifest, &output, |progress| {
        println!("progress: {progress}%");
    })?;
    println!("rebuilt game ID {game_id} into {}", output.display());
    Ok(())
}
