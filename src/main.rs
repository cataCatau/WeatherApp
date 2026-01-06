#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

mod api;
mod app;
mod favorites;
mod ui;

use app::App;

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "emoji_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../seguiemj.ttf")), 
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("emoji_font".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("emoji_font".to_owned());

    ctx.set_fonts(fonts);
}

#[tokio::main]
async fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0])
            .with_title("WeatherDashboard"),
        ..Default::default()
    };

   if let Err(e)= eframe::run_native(
        "App",
        options,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);
            Box::new(App::default())
        }),
    ) {
        eprintln!("Fatal Error while running GUI: {}",e);
    }
}
