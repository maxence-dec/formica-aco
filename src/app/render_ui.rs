use eframe::egui;

use crate::app::{self, AppState, UiCommand, MAX_RENDER_SPEED};

impl app::App {
    pub fn draw_ui(&mut self, ctx: &egui::Context) -> UiCommand {
        // draw egui UI panel
        // W/ returning UiCommand
        let mut command = UiCommand::None;
        let widgets_enabled = matches!(self.state, AppState::Configuring);

        egui::SidePanel::right("control_panel").exact_width(300.0).resizable(false).show(
            ctx,
            |ui: &mut egui::Ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // --- Buttons row ---
                    self.buttons_row(&mut command, ui);
                    // --- Seed ---
                    self.seed(widgets_enabled, ui);
                    // --- Simulation ---
                    self.simulation(widgets_enabled, ui);
                    // --- Ant ---
                    self.ant(widgets_enabled, ui);
                    // --- Sensor ---
                    self.sensor(widgets_enabled, ui);
                    // --- Pheromone ---
                    self.pheromone(widgets_enabled, ui);
                    // --- Nest ---
                    self.nest(widgets_enabled, ui);
                    // --- Food ---
                    self.food(widgets_enabled, ui);
                    // --- World (read-only) ---
                    self.world_read(&mut command, widgets_enabled, ui);
                }); // ScrollArea
            },
        );
        command
    }

    fn world_read(&mut self, command: &mut UiCommand, widgets_enabled: bool, ui: &mut egui::Ui) {
        ui.label("WORLD (read-only)");
        ui.label(format!(
            "Width: {}  Height: {}",
            self.world_config.sim_width as u32, self.world_config.sim_height as u32
        ));
        ui.label(format!("Cell size: {}", self.world_config.cell_size as u32));

        ui.separator();

        ui.add_enabled_ui(widgets_enabled, |ui| {
            if ui
                .add_sized([ui.available_width(), 0.0], egui::Button::new("Reset to defaults"))
                .clicked()
            {
                *command = UiCommand::Reset;
            }
        });

        // ui.add_enabled_ui(widgets_enabled, |ui| {
        //     if ui
        //         .add_sized(
        //             [ui.available_width(), 0.0],
        //             egui::Button::new("Import config.toml"),
        //         )
        //         .clicked()
        //     {
        //         *command = UiCommand::Import;
        //     }
        // });

        ui.separator();
    }

    fn food(&mut self, widgets_enabled: bool, ui: &mut egui::Ui) {
        ui.label("FOOD");
        ui.add_enabled_ui(widgets_enabled, |ui| {
            ui.horizontal(|ui| {
                ui.label("Sources");
                let mut s = self.ants_config.food_source_count.to_string();
                if ui.text_edit_singleline(&mut s).changed() {
                    if let Ok(v) = s.parse::<usize>() {
                        self.ants_config.food_source_count = v;
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("Tokens");
                let mut s = self.ants_config.food_tokens_per_source.to_string();
                if ui.text_edit_singleline(&mut s).changed() {
                    if let Ok(v) = s.parse::<u32>() {
                        self.ants_config.food_tokens_per_source = v;
                    }
                }
            });
            slider_row(ui, "Radius", &mut self.ants_config.food_radius, 5.0..=40.0);
            slider_row(
                ui,
                "Min nest",
                &mut self.ants_config.min_food_distance_from_nest,
                50.0..=300.0,
            );
            slider_row(
                ui,
                "Min pair",
                &mut self.ants_config.min_food_pairwise_distance,
                20.0..=200.0,
            );
        });

        ui.separator();
    }

    fn nest(&mut self, widgets_enabled: bool, ui: &mut egui::Ui) {
        ui.label("NEST");
        ui.add_enabled_ui(widgets_enabled, |ui| {
            slider_row(ui, "Radius", &mut self.ants_config.nest_radius, 5.0..=50.0);
        });

        ui.separator();
    }

    fn pheromone(&mut self, widgets_enabled: bool, ui: &mut egui::Ui) {
        ui.label("PHEROMONE");
        ui.add_enabled_ui(widgets_enabled, |ui| {
            slider_row(ui, "Evaporation", &mut self.ants_config.evaporation_factor, 0.90..=0.999);
            slider_row(ui, "Deposit", &mut self.ants_config.deposit_amount, 0.1..=5.0);
        });

        ui.separator();
    }

    fn sensor(&mut self, widgets_enabled: bool, ui: &mut egui::Ui) {
        ui.label("SENSOR");
        ui.add_enabled_ui(widgets_enabled, |ui| {
            slider_row(ui, "Angle", &mut self.ants_config.sensor_angle, 0.1..=1.0);
            slider_row(ui, "Distance", &mut self.ants_config.sensor_distance, 4.0..=40.0);
        });

        ui.separator();
    }

    fn ant(&mut self, widgets_enabled: bool, ui: &mut egui::Ui) {
        ui.label("ANT");
        ui.add_enabled_ui(widgets_enabled, |ui| {
            slider_row(ui, "Speed", &mut self.ants_config.speed, 0.1..=5.0);
            slider_row(ui, "Turn rate", &mut self.ants_config.turn_rate, 0.05..=1.0);
            slider_row(ui, "Noise", &mut self.ants_config.noise_angle, 0.0..=0.5);
        });

        ui.separator();
    }

    fn simulation(&mut self, widgets_enabled: bool, ui: &mut egui::Ui) {
        ui.label("SIMULATION");
        ui.add_enabled_ui(widgets_enabled, |ui| {
            ui.horizontal(|ui| {
                ui.label("Ants");
                let mut s = self.ants_config.number.to_string();
                if ui.text_edit_singleline(&mut s).changed() {
                    if let Ok(v) = s.parse::<usize>() {
                        self.ants_config.number = v;
                    }
                }
            });
        });

        ui.separator();
    }

    fn seed(&mut self, widgets_enabled: bool, ui: &mut egui::Ui) {
        ui.add_enabled_ui(widgets_enabled, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Random seed").clicked() {
                    use ::rand::Rng;
                    self.world_config.rng_seed = ::rand::thread_rng().gen();
                }
            });
            ui.horizontal(|ui| {
                ui.label("Seed:");
                let mut seed_str = self.world_config.rng_seed.to_string();
                if ui.text_edit_singleline(&mut seed_str).changed() {
                    if let Ok(v) = seed_str.parse::<u64>() {
                        self.world_config.rng_seed = v;
                    }
                }
            });
        });
        ui.separator();
    }

    fn buttons_row(
        &mut self,
        //render_speedup: &mut f32,
        command: &mut UiCommand,
        ui: &mut egui::Ui,
    ) {
        ui.horizontal(|ui| {
            let start_label = match self.state {
                AppState::Configuring => "Start",
                AppState::Running => "Pause",
                AppState::Paused => "Resume",
            };
            if ui.button(start_label).clicked() {
                *command = match self.state {
                    AppState::Configuring => UiCommand::Start,
                    AppState::Running => UiCommand::Pause,
                    AppState::Paused => UiCommand::Resume,
                };
            }
            ui.add_enabled_ui(!matches!(self.state, AppState::Configuring), |ui| {
                if ui.button("Stop").clicked() {
                    *command = UiCommand::Stop;
                }
            });
            ui.horizontal(|ui| {
                slider_row(ui, "Speed", &mut self.sim_speed, 0.0_f32..=MAX_RENDER_SPEED);
            });
        });
        ui.separator();
    }
}

fn slider_row(
    ui: &mut egui::Ui, label: &str, value: &mut f32, range: std::ops::RangeInclusive<f32>,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::Slider::new(value, range).show_value(false));
        ui.label(format!("{:.2}", value));
    });
}
