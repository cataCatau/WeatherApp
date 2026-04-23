#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

mod api;
mod app;
mod favorites;
mod ui;

use app::App;

//fn configure_fonts(ctx: &egui::Context) {
//     let mut fonts = egui::FontDefinitions::default();

//     fonts.font_data.insert(
//         "emoji_font".to_owned(),
//         egui::FontData::from_static(include_bytes!("../seguiemj.ttf")),
//     );

//     fonts
//         .families
//         .entry(egui::FontFamily::Proportional)
//         .or_default()
//         .push("emoji_font".to_owned());
//     fonts
//         .families
//         .entry(egui::FontFamily::Monospace)
//         .or_default()
//         .push("emoji_font".to_owned());

//     ctx.set_fonts(fonts);
//}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0])
            .with_title("WeatherDashboard"),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "App",
        options,
        Box::new(|cc| {
            let _ = cc; // App starts without custom font configuration.
            Box::new(App::default())
        }),
    ) {
        eprintln!("Fatal Error while running GUI: {}", e);
    }
}

// ==========================================
// PUNCTUL DE INTRARE PENTRU BROWSER (WASM)
// ==========================================
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirectionam logurile catre consola din browser (F12)
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // Asigura-te ca in index.html ai <canvas id="the_canvas_id"></canvas>
                web_options,
                Box::new(|cc| {
                    let _ = cc; // App starts without custom font configuration.
                    Box::new(App::default())
                }),
            )
            .await
            .expect("failed to start eframe");
    });
}
