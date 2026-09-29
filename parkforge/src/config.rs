use std::path::Path;

use parkforge_project::read_project_config;
use parkforge_types::GameId;

use crate::error::{Error, Result};

pub fn configured_game_ids(project_config: &Path) -> Result<Vec<GameId>> {
    let config = read_project_config(project_config).map_err(|source| Error::Project {
        root: project_config.to_path_buf(),
        source: Box::new(source),
    })?;
    Ok(config.games.into_keys().collect())
}
