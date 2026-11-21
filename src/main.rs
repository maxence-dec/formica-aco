use eframe::egui;
use formica_aco::app::App;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("formica-aco")
            .with_inner_size([1043.0, 640.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native("formica-aco", options, Box::new(|_cc| Ok(Box::new(App::new()))))
}
