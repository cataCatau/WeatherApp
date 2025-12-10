use eframe::{egui, wgpu::hal::auxil::db};
use egui::Align;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct GeoSearch {
    results: Option<Vec<GeoLocation>>,
}
#[derive(Deserialize, Debug)]
struct GeoLocation {
    name: String,
    latitude: f64,
    longitude: f64,
    country: Option<String>,
}
struct App {
    city_name: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            city_name: "Bucuresti".to_owned(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                ui.heading(egui::RichText::new("Weather Dashboard").size(24.0).strong());
                ui.add_space(250.0);
                ui.label(egui::RichText::new("Introduceti orasul").size(24.0));
                ui.text_edit_singleline(&mut self.city_name);

                if ui.button("search").clicked() {
                    let city=self.city_name.clone();
                    tokio::spawn(async move {
                        let geo_url = format!(
                            "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=ro&format=json",
                            city);
                        match reqwest::get(&geo_url).await {
                            Ok(resp) => {
                                match resp.json::<GeoSearch>().await {
                                    Ok(geo_data) => {
                                        if let Some(results) = geo_data.results {
                                            if let Some(loc) = results.first() {
                                                println!("{},{}", loc.latitude, loc.longitude);
                                            }
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                            Err(_) => {}
                        }
                    });
                }
            });
        });
    }
}
#[tokio::main]
async fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0])
            .with_title("WeatherDashboard"),
        ..Default::default()
    };

    eframe::run_native("App", options, Box::new(|_cc| Box::new(App::default())))
}
