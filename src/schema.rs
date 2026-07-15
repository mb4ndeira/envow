use std::{collections::HashMap, path::Path};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct VarSpec {
    #[serde(rename = "type")]
    pub var_type: Option<String>,
    pub required: Option<bool>,
    pub default: Option<String>,
    pub example: Option<String>,
    pub description: Option<String>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub choices: Option<Vec<String>>,
    pub env: Option<String>,
    pub values: Option<HashMap<String, String>>,
    pub generate: Option<String>,
}

impl VarSpec {
    pub fn is_required(&self) -> bool {
        self.required.unwrap_or(false) && self.default.is_none()
    }

    pub fn applies_to_env(&self, cli_env: Option<&str>) -> bool {
        match (&self.env, cli_env) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some(var_env), Some(cli)) => var_env == cli,
        }
    }
}

pub type Section = HashMap<String, VarSpec>;
pub type Schema = HashMap<String, Section>;

pub fn load(path: &Path) -> anyhow::Result<Schema> {
    let content = std::fs::read_to_string(path)
        .map_err(|_| anyhow::anyhow!("schema not found: {}", path.display()))?;
    toml::from_str(&content).map_err(|e| anyhow::anyhow!("invalid schema: {e}"))
}
