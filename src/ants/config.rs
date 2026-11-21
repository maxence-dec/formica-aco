use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    ///Ants
    #[serde(default = "default_count")]
    pub number: usize,
    #[serde(default = "default_speed")]
    pub speed: f32,
    #[serde(default = "default_turn_rate")]
    pub turn_rate: f32,
    #[serde(default = "default_noise_angle")]
    pub noise_angle: f32,
    #[serde(default = "default_sensor_angle")]
    pub sensor_angle: f32,
    #[serde(default = "default_sensor_distance")]
    pub sensor_distance: f32,
    ///Pheromones
    #[serde(default = "default_evaporation_factor")]
    pub evaporation_factor: f32,
    #[serde(default = "default_deposit_amount")]
    pub deposit_amount: f32,
    ///Food
    #[serde(default = "default_nest_radius")]
    pub nest_radius: f32,
    #[serde(default = "default_food_radius")]
    pub food_radius: f32,
    #[serde(default = "default_food_source_count")]
    pub food_source_count: usize,
    #[serde(default = "default_food_tokens_per_source")]
    pub food_tokens_per_source: u32,
    #[serde(default = "default_min_food_distance_from_nest")]
    pub min_food_distance_from_nest: f32,
    #[serde(default = "default_min_food_pairwise_distance")]
    pub min_food_pairwise_distance: f32,
    #[serde(default = "default_inbound_interactable_sensing")]
    pub inbound_interactable_sensing: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            number: default_count(),
            speed: default_speed(),
            turn_rate: default_turn_rate(),
            noise_angle: default_noise_angle(),
            sensor_angle: default_sensor_angle(),
            sensor_distance: default_sensor_distance(),
            evaporation_factor: default_evaporation_factor(),
            deposit_amount: default_deposit_amount(),
            nest_radius: default_nest_radius(),
            food_radius: default_food_radius(),
            food_source_count: default_food_source_count(),
            food_tokens_per_source: default_food_tokens_per_source(),
            min_food_distance_from_nest: default_min_food_distance_from_nest(),
            min_food_pairwise_distance: default_min_food_pairwise_distance(),
            inbound_interactable_sensing: default_inbound_interactable_sensing(),
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
fn default_count() -> usize {
    500
}
fn default_speed() -> f32 {
    1.0
}
fn default_turn_rate() -> f32 {
    0.3
}
fn default_noise_angle() -> f32 {
    0.1
}
fn default_sensor_angle() -> f32 {
    0.524
}
fn default_sensor_distance() -> f32 {
    12.0
}
fn default_evaporation_factor() -> f32 {
    0.98
}
fn default_deposit_amount() -> f32 {
    1.0
}
fn default_nest_radius() -> f32 {
    20.0
}
fn default_food_radius() -> f32 {
    15.0
}
fn default_food_source_count() -> usize {
    3
}
fn default_food_tokens_per_source() -> u32 {
    100
}
fn default_min_food_distance_from_nest() -> f32 {
    150.0
}
fn default_min_food_pairwise_distance() -> f32 {
    100.0
}
fn default_inbound_interactable_sensing() -> bool {
    false
}
