use rand::Rng;
use rand::{rngs::StdRng, SeedableRng};

use crate::ants::{
    config::Config,
    nest::Nest,
    pheromone::{Channel, PheromoneGrid},
};
use crate::world_sim::world::{self, FoodSource, Obstacle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AntState {
    Foraging,
    Returning,
}

pub struct Colony {
    pub pos_x: Vec<f32>,
    pub pos_y: Vec<f32>,
    pub heading: Vec<f32>,
    pub state: Vec<AntState>,

    pub nest: Nest,
    pub pheromone: PheromoneGrid,

    pub config: Config,
}

impl Colony {
    pub fn new(
        ants_config: &Config, world_config: &crate::world_sim::config::Config,
    ) -> anyhow::Result<Self> {
        use rand::Rng;
        use std::f32::consts::TAU;

        let mut rng = StdRng::seed_from_u64(world_config.rng_seed);

        let pheromone = PheromoneGrid::new(
            world_config.sim_width,
            world_config.sim_height,
            world_config.cell_size,
        );

        let nest = Nest {
            pos: (world_config.sim_width / 2.0, world_config.sim_height / 2.0),
            radius: ants_config.nest_radius,
        };

        let mut pos_x = Vec::with_capacity(ants_config.number);
        let mut pos_y = Vec::with_capacity(ants_config.number);
        let mut heading = Vec::with_capacity(ants_config.number);
        let mut state = Vec::with_capacity(ants_config.number);

        for _ in 0..ants_config.number {
            pos_x.push(nest.pos.0);
            pos_y.push(nest.pos.1);
            heading.push(rng.gen_range(0.0..TAU));
            state.push(AntState::Foraging);
        }

        Ok(Self { pos_x, pos_y, heading, state, nest, pheromone, config: ants_config.clone() })
    }

    /// Read pheromone at a world-space position, clamping to grid bounds.
    fn sample(&self, channel: Channel, wx: f32, wy: f32) -> f32 {
        let gx = ((wx / self.pheromone.cell_size) as i32).clamp(0, self.pheromone.width as i32 - 1)
            as usize;
        let gy = ((wy / self.pheromone.cell_size) as i32).clamp(0, self.pheromone.height as i32 - 1)
            as usize;
        self.pheromone.read(channel, gx, gy)
    }

    fn pheromone_sensing(&mut self, food_sources: &[FoodSource], ant: usize) -> f32 {
        let mut inbound = true;
        let h = self.heading[ant];
        let x: f32 = self.pos_x[ant];
        let y = self.pos_y[ant];

        let dx = x - self.nest.pos.0;
        let dy = y - self.nest.pos.1;
        if (dx * dx + dy * dy).sqrt() < self.config.nest_radius {
            inbound = false;
        } else {
            for source in food_sources.iter() {
                if source.is_active() {
                    let dx = x - source.pos.0;
                    let dy = y - source.pos.1;
                    if (dx * dx + dy * dy).sqrt() < self.config.food_radius {
                        inbound = false;
                        break;
                    }
                }
            }
        }

        let mut turn = 0.0_f32;
        if inbound {
            let centre_val = self.sample(
                match self.state[ant] {
                    AntState::Foraging => Channel::Food,
                    AntState::Returning => Channel::Home,
                },
                x + h.cos() * self.config.sensor_distance,
                y + h.sin() * self.config.sensor_distance,
            );
            let left_val = self.sample(
                match self.state[ant] {
                    AntState::Foraging => Channel::Food,
                    AntState::Returning => Channel::Home,
                },
                x + (h - self.config.sensor_angle).cos() * self.config.sensor_distance,
                y + (h - self.config.sensor_angle).sin() * self.config.sensor_distance,
            );
            let right_val = self.sample(
                match self.state[ant] {
                    AntState::Foraging => Channel::Food,
                    AntState::Returning => Channel::Home,
                },
                x + (h + self.config.sensor_angle).cos() * self.config.sensor_distance,
                y + (h + self.config.sensor_angle).sin() * self.config.sensor_distance,
            );
            // Argmax: left wins → turn left, right wins → turn right, otherwise straight.
            if left_val > centre_val && left_val > right_val {
                turn = -self.config.turn_rate
            } else if right_val > centre_val && right_val > left_val {
                turn = self.config.turn_rate
            };
        }
        turn
    }

    /// Food and nest contact checks, state transitions.
    pub fn resolve_transitions(&mut self, food_sources: &mut [FoodSource]) {
        let n = self.pos_x.len();
        for i in 0..n {
            let x = self.pos_x[i];
            let y = self.pos_y[i];
            match self.state[i] {
                AntState::Foraging => {
                    for source in food_sources.iter_mut() {
                        if source.is_active() {
                            let dx = x - source.pos.0;
                            let dy = y - source.pos.1;
                            let circle_radius = (dx * dx + dy * dy).sqrt();
                            if circle_radius < self.config.food_radius {
                                source.try_consume();
                                self.state[i] = AntState::Returning;
                                break;
                            }
                        }
                    }
                },
                AntState::Returning => {
                    let dx = x - self.nest.pos.0;
                    let dy = y - self.nest.pos.1;
                    if (dx * dx + dy * dy).sqrt() < self.config.nest_radius {
                        self.state[i] = AntState::Foraging;
                    }
                },
            }
        }
    }

    /// Per-ant motion loop: sense, steer, move, bounce, deposit.
    pub fn move_tplus<T: Rng>(
        &mut self, obstacle: &Vec<Obstacle>, food_sources: &[FoodSource], rng: &mut T,
    ) {
        let n = self.pos_x.len();
        for i in 0..n {
            // Foraging reads food, deposits home. Returning reads home, deposits food.

            let turn = self.pheromone_sensing(food_sources, i);

            self.heading[i] +=
                turn + rng.gen_range(-self.config.noise_angle..self.config.noise_angle);

            let vel = (
                self.heading[i].cos() * self.config.speed,
                self.heading[i].sin() * self.config.speed,
            );
            let (new_pos, new_vel) =
                world::resolve_motion(obstacle, (self.pos_x[i], self.pos_y[i]), vel);

            self.heading[i] = new_vel.1.atan2(new_vel.0);
            self.pos_x[i] = new_pos.0;
            self.pos_y[i] = new_pos.1;

            self.pheromone.deposit(
                match self.state[i] {
                    AntState::Foraging => Channel::Home,
                    AntState::Returning => Channel::Food,
                },
                self.pos_x[i],
                self.pos_y[i],
                self.config.deposit_amount,
            );
        }
    }
}
