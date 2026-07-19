use serde::{Deserialize, Serialize};

use parkforge_model::GameId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectMetadata,
    #[serde(rename = "game", default)]
    pub games: Vec<GameConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub game_id: GameId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}
