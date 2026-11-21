use eframe::egui;
use eframe::egui::{Color32, Pos2, Shape};
use std::f32::consts::TAU;

use crate::app::{SATURATION, SIM_MARGIN};
use crate::{
    ants::{colony::AntState, pheromone::Channel},
    app,
};

impl app::App {
    pub fn draw_sim(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none()) // no default panel padding/fill, we're drawing manually
            .show(ctx, |ui| {
                let painter = ui.painter();
                let mut shapes: Vec<Shape> = Vec::new();

                shapes.push(Shape::rect_filled(
                    ui.max_rect(), // fill this panel's own rect, not the whole window
                    0.0,
                    Color32::from_rgba_premultiplied(20, 20, 20, 255),
                ));

                self.world_texture_and_shapes(ctx, &mut shapes);

                let sim_rect = egui::Rect::from_min_size(
                    egui::pos2(SIM_MARGIN, SIM_MARGIN),
                    egui::vec2(self.world_config.sim_width, self.world_config.sim_height),
                );
                shapes.push(Shape::rect_stroke(
                    sim_rect,
                    0.0,
                    egui::Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(80, 80, 80, 255)),
                ));

                painter.extend(shapes);
            });
    }

    fn world_texture_and_shapes(&mut self, ctx: &egui::Context, shapes: &mut Vec<Shape>) {
        let Some(w) = self.world.as_ref() else {
            return;
        };

        let ph = &w.ants.pheromone;
        let mut pixels: Vec<Color32> = Vec::with_capacity(ph.width * ph.height);
        for gy in 0..ph.height {
            for gx in 0..ph.width {
                let home = map_intensity(ph.read(Channel::Home, gx, gy), SATURATION);
                let food = map_intensity(ph.read(Channel::Food, gx, gy), SATURATION);
                pixels.push(Color32::from_rgba_premultiplied(home, food, 0, 255));
            }
        }
        let image = egui::ColorImage { size: [ph.width, ph.height], pixels };

        match &mut self.pheromone_texture {
            None => {
                self.pheromone_texture = Some(ctx.load_texture(
                    "pheromone_texture",
                    image,
                    egui::TextureOptions::NEAREST,
                ));
            },
            Some(texture) => texture.set(image, egui::TextureOptions::NEAREST),
        }

        if let Some(texture) = &self.pheromone_texture {
            let dest_size =
                egui::vec2(ph.width as f32 * ph.cell_size, ph.height as f32 * ph.cell_size);
            let dest_rect =
                egui::Rect::from_min_size(egui::pos2(SIM_MARGIN, SIM_MARGIN), dest_size);
            // egui's version of draw_texture_ex: a textured-rect shape,
            // pushed into the same batch as everything else rather than
            // painted with its own separate call.
            shapes.push(Shape::image(
                texture.id(),
                dest_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            ));
        }

        for source in &w.food_sources {
            let alpha = if source.is_active() { 255 } else { 20 };
            shapes.push(Shape::circle_filled(
                Pos2::new(source.pos.0 + SIM_MARGIN, source.pos.1 + SIM_MARGIN),
                source.radius,
                Color32::from_rgba_unmultiplied(0, 228, 48, alpha),
            ));
        }

        // ---- nest ----
        shapes.push(Shape::circle_filled(
            Pos2::new(w.ants.nest.pos.0 + SIM_MARGIN, w.ants.nest.pos.1 + SIM_MARGIN),
            w.ants.nest.radius,
            Color32::DARK_BLUE,
        ));

        for i in 0..w.ants.heading.len() {
            let x = w.ants.pos_x[i] + SIM_MARGIN;
            let y = w.ants.pos_y[i] + SIM_MARGIN;
            let h = w.ants.heading[i];
            let color = match w.ants.state[i] {
                AntState::Foraging => Color32::WHITE,
                AntState::Returning => Color32::from_rgb(255, 165, 0), // ORANGE
            };

            let p1 = Pos2::new(x + h.cos() * 3.0, y + h.sin() * 3.0);
            let p2 = Pos2::new(x + (h + TAU / 3.0).cos() * 2.0, y + (h + TAU / 3.0).sin() * 2.0);
            let p3 = Pos2::new(x + (h - TAU / 3.0).cos() * 2.0, y + (h - TAU / 3.0).sin() * 2.0);

            shapes.push(Shape::convex_polygon(vec![p1, p2, p3], color, egui::Stroke::NONE));
        }
    }
}

fn map_intensity(value: f32, saturation: f32) -> u8 {
    let normalised = (value.max(0.0).ln_1p() / saturation.ln_1p()).min(1.0);
    (normalised * 255.0) as u8
}
