pub mod render_sim;
pub mod render_ui;
use std::time::{Duration, Instant};

use crate::{
    ants,
    world_sim::{
        self,
        world::{self, World},
    },
};
use eframe::egui;

pub const N: usize = 150;
pub(crate) const SIM_MARGIN: f32 = 20.0;
pub(crate) const MAX_RENDER_SPEED: f32 = 5.0;
pub(crate) const SATURATION: f32 = 50.0;
pub const MAX_TICKS_PER_FRAME: u32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Configuring,
    Running,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiCommand {
    None,
    Start,
    Stop,
    Pause,
    Resume,
    Reset,
    //Import,
}

pub struct App {
    command: UiCommand,
    pub(crate) state: AppState,
    pub(crate) world: Option<world::World>,
    pub(crate) world_config: world_sim::config::Config,
    pub(crate) ants_config: ants::config::Config,
    last_frame_time: Instant,
    pub(crate) pheromone_texture: Option<egui::TextureHandle>,
    accumulator: f32,
    tick_duration: f32, // fixed sim timestep, e.g. 1.0/60.0 — does not change with speed
    pub(crate) sim_speed: f32, // the actual user-facing speed multiplier
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> App {
        App {
            command: UiCommand::None,
            state: AppState::Configuring,
            world: None,
            world_config: world_sim::config::Config::default(),
            ants_config: ants::config::Config::default(),
            last_frame_time: Instant::now(),
            pheromone_texture: None,
            accumulator: 0.0_f32,
            tick_duration: 1.0_f32 / 60.0_f32,
            sim_speed: 1.0_f32,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame_time).as_secs_f32().min(0.25);
        self.last_frame_time = now;

        if self.state == AppState::Running {
            self.accumulator += frame_time;
            let effective_tick = self.tick_duration / self.sim_speed;
            if let Some(w) = &mut self.world {
                let mut ticks_this_frame = 0;
                while self.accumulator >= effective_tick && ticks_this_frame < MAX_TICKS_PER_FRAME {
                    w.tick();
                    self.accumulator -= effective_tick;
                    ticks_this_frame += 1;
                }
                if ticks_this_frame == MAX_TICKS_PER_FRAME && self.accumulator >= effective_tick {
                    self.accumulator = 0.0;
                }
            }
        }

        let render_target = Duration::from_secs_f32(1.0 / 24.0);
        let elapsed = self.last_frame_time.elapsed();
        if elapsed < render_target {
            ctx.request_repaint_after(render_target - elapsed);
        } else {
            ctx.request_repaint();
        }

        self.draw_sim(ctx);
        self.command = self.draw_ui(ctx);

        match self.command {
            UiCommand::Start => {
                match World::new(self.world_config.clone(), self.ants_config.clone()) {
                    Ok(world) => self.world = Some(world),
                    Err(e) => {
                        // surface it however fits your app — log it, store it in a
                        // field the UI reads to show an error panel, etc.
                        eprintln!("failed to create world: {e:?}");
                    },
                }
                self.state = AppState::Running;
            },
            UiCommand::Stop => {
                self.state = AppState::Configuring;
                self.world = None;
                self.sim_speed = 1.0_f32;
            },
            UiCommand::Pause => {
                self.state = AppState::Paused;
                self.sim_speed = 1.0_f32;
            },
            UiCommand::Resume => {
                self.state = AppState::Running;
            },
            UiCommand::Reset => {
                self.world_config = world_sim::config::Config::default();
                self.ants_config = ants::config::Config::default();
            },
            // UiCommand::Import => {
            //     ui_config = SimConfig::load("config.toml").unwrap_or_default();
            // }
            UiCommand::None => {},
        }
    }
}
