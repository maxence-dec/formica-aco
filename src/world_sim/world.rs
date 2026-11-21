use rand::rngs::StdRng;
use rand::SeedableRng;

const WALL_EXTENT: f32 = 100_000.0; // far larger than any plausible sim coordinate

use crate::{
    ants::{self, colony::Colony, nest::Nest},
    world_sim::config::Config,
};

pub struct FoodSource {
    pub pos: (f32, f32),
    pub radius: f32,
    tokens_remaining: u32,
}

impl FoodSource {
    pub fn new(pos: (f32, f32), radius: f32, tokens: u32) -> Self {
        Self { pos, radius, tokens_remaining: tokens }
    }

    pub fn try_consume(&mut self) -> bool {
        if self.tokens_remaining > 0 {
            self.tokens_remaining -= 1;
            true
        } else {
            false
        }
    }

    pub fn is_active(&self) -> bool {
        self.tokens_remaining > 0
    }
}

pub struct Obstacle {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub struct World {
    pub food_sources: Vec<FoodSource>,
    pub obstacles: Vec<Obstacle>,

    pub ants: Colony,
    pub(crate) rng: StdRng,
}

impl World {
    pub fn new(world_config: Config, ants_config: ants::config::Config) -> anyhow::Result<Self> {
        let mut rng = StdRng::seed_from_u64(world_config.rng_seed);
        let ants = Colony::new(&ants_config, &world_config).unwrap();
        let food_sources = place_food_sources(&mut rng, &world_config, &ants_config, &ants.nest);
        let obstacles = place_world_limit(world_config);

        Ok(Self { food_sources, obstacles, ants, rng })
    }

    pub fn tick(&mut self) {
        self.ants.pheromone.evaporate(self.ants.config.evaporation_factor);
        self.ants.move_tplus(&self.obstacles, &self.food_sources, &mut self.rng);
        self.ants.resolve_transitions(&mut self.food_sources);
    }
}

pub fn resolve_motion(
    obstacles: &Vec<Obstacle>, pos: (f32, f32), vel: (f32, f32),
) -> ((f32, f32), (f32, f32)) {
    let mut current_pos = (pos.0 + vel.0, pos.1 + vel.1);
    let mut current_vel = vel;

    const MAX_RESOLUTION_PASSES: u32 = 2; //4;

    for _ in 0..MAX_RESOLUTION_PASSES {
        let mut normal_x = 0.0_f32;
        let mut normal_y = 0.0_f32;
        let mut fixed_x = current_pos.0;
        let mut fixed_y = current_pos.1;
        let mut hit_any = false;

        for obs in obstacles {
            let hit_x = current_pos.0 > obs.x && current_pos.0 < obs.x + obs.w;
            let hit_y = current_pos.1 > obs.y && current_pos.1 < obs.y + obs.h;

            if !hit_x || !hit_y {
                continue;
            }

            hit_any = true;
            let is_vertical_wall = obs.w < obs.h;

            if is_vertical_wall {
                let hit_from_right = pos.0 >= obs.x + obs.w;
                normal_x = if hit_from_right { 1.0 } else { -1.0 };
                fixed_x = if hit_from_right { obs.x + obs.w } else { obs.x };
            } else {
                let hit_from_below = pos.1 >= obs.y + obs.h;
                normal_y = if hit_from_below { 1.0 } else { -1.0 };
                fixed_y = if hit_from_below { obs.y + obs.h } else { obs.y };
            }
        }

        if !hit_any {
            return (current_pos, current_vel);
        }

        let len = (normal_x * normal_x + normal_y * normal_y).sqrt();
        let (nx, ny) = if len > 0.0 {
            (normal_x / len, normal_y / len)
        } else {
            (0.0, 0.0)
        };

        let dot = current_vel.0 * nx + current_vel.1 * ny;
        current_vel = (current_vel.0 - 2.0 * dot * nx, current_vel.1 - 2.0 * dot * ny);
        current_pos = (fixed_x, fixed_y);
    }

    (current_pos, current_vel)
}

fn place_world_limit(world_config: Config) -> Vec<Obstacle> {
    let w = world_config.sim_width;
    let h = world_config.sim_height;
    const T: f32 = WALL_EXTENT; // wall thickness, shared so all four corners overlap consistently

    vec![
        Obstacle { x: -T, y: -T, w: w + 2.0 * T, h: T }, // top
        Obstacle { x: -T, y: h, w: w + 2.0 * T, h: T },  // bottom
        Obstacle { x: -T, y: -T, w: T, h: h + 2.0 * T }, // left
        Obstacle { x: w, y: -T, w: T, h: h + 2.0 * T },  // right
    ]
}

fn place_food_sources(
    rng: &mut StdRng, world_config: &Config, ants_config: &ants::config::Config, nest: &Nest,
) -> Vec<FoodSource> {
    use rand::Rng;

    let mut sources: Vec<FoodSource> = Vec::new();
    for src_idx in 0..ants_config.food_source_count {
        let mut placed = false;
        for _ in 0..1000 {
            let x = rng.gen_range(0.0..world_config.sim_width);
            let y = rng.gen_range(0.0..world_config.sim_height);

            let dx = x - nest.pos.0;
            let dy = y - nest.pos.1;
            if (dx * dx + dy * dy).sqrt() < ants_config.min_food_distance_from_nest {
                continue;
            }

            let too_close = sources.iter().any(|s: &FoodSource| {
                let dx = x - s.pos.0;
                let dy = y - s.pos.1;
                (dx * dx + dy * dy).sqrt() < ants_config.min_food_pairwise_distance
            });
            if too_close {
                continue;
            }

            sources.push(FoodSource::new(
                (x, y),
                ants_config.food_radius,
                ants_config.food_tokens_per_source,
            ));
            placed = true;
            break;
        }
        if !placed {
            panic!(
                "failed to place food source {} after 1000 attempts; \
                    consider relaxing placement constraints",
                src_idx + 1
            );
        }
    }
    sources
}
