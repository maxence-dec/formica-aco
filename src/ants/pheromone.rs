#[derive(Clone, Copy)]
pub enum Channel {
    Home,
    Food,
}

pub struct PheromoneGrid {
    pub home: Vec<f32>,
    pub food: Vec<f32>,
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
}

impl PheromoneGrid {
    pub fn new(sim_width: f32, sim_height: f32, cell_size: f32) -> Self {
        let width = (sim_width / cell_size).round() as usize;
        let height = (sim_height / cell_size).round() as usize;
        Self {
            home: vec![0.0; width * height],
            food: vec![0.0; width * height],
            width,
            height,
            cell_size,
        }
    }

    /// Read from the specified channel at grid coordinates.
    /// Caller must clamp gx, gy to [0, width) and [0, height).
    pub fn read(&self, channel: Channel, gx: usize, gy: usize) -> f32 {
        let idx = gy * self.width + gx;
        match channel {
            Channel::Home => self.home[idx],
            Channel::Food => self.food[idx],
        }
    }

    /// Deposit at world position, single-cell stamp, clamps to grid bounds.
    pub fn deposit(&mut self, channel: Channel, world_x: f32, world_y: f32, amount: f32) {
        let gx = ((world_x / self.cell_size) as i32).clamp(0, self.width as i32 - 1) as usize;
        let gy = ((world_y / self.cell_size) as i32).clamp(0, self.height as i32 - 1) as usize;
        let idx = gy * self.width + gx;
        match channel {
            Channel::Home => self.home[idx] += amount,
            Channel::Food => self.food[idx] += amount,
        }
    }

    /// Multiply both channels in place by `factor`. Called once per tick.
    pub fn evaporate(&mut self, factor: f32) {
        for v in &mut self.home {
            *v *= factor;
        }
        for v in &mut self.food {
            *v *= factor;
        }
    }
}
