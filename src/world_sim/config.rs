use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_sim_width")]
    pub sim_width: f32,
    #[serde(default = "default_sim_height")]
    pub sim_height: f32,
    #[serde(default = "default_cell_size")]
    pub cell_size: f32,
    #[serde(default = "default_rng_seed")]
    pub rng_seed: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sim_width: default_sim_width(),
            sim_height: default_sim_height(),
            cell_size: default_cell_size(),
            rng_seed: default_rng_seed(),
        }
    }
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&text)?;
        Ok(config)
    }
}

// serde requires named functions for per-field defaults.
fn default_sim_width() -> f32 {
    700.0
}
fn default_sim_height() -> f32 {
    600.0
}
fn default_cell_size() -> f32 {
    4.0
}
fn default_rng_seed() -> u64 {
    0
}
