use eframe::egui;
use egui::Align;
use poll_promise::Promise;
use serde::Deserialize;
#[derive(Deserialize, Debug)]
struct GeoSearch {
    results: Option<Vec<GeoLocation>>,
}
#[derive(Deserialize, Debug, Clone)]
struct GeoLocation {
    name: String,
    latitude: f64,
    longitude: f64,
    country: Option<String>,
}
#[derive(Deserialize, Debug)]
struct WeatherResponse {
    current: CurrentData,
}

#[derive(Deserialize, Debug)]
struct CurrentData {
    temperature_2m: f32,
    time: String,
}

async fn fetch_weather(city: String) -> Result<f32, String> {
    let geo_url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=ro&format=json",
        city
    );
    let geo_resp = match reqwest::get(&geo_url).await {
        Ok(resp) => resp,
        Err(e) => return Err(format!("Eroare rețea GEO: {}", e)),
    };
    let geo_data = match geo_resp.json::<GeoSearch>().await {
        Ok(data) => data,
        Err(e) => return Err(format!("Eroare JSON Geo: {}", e)),
    };
    let results = match geo_data.results {
        Some(r) => r,
        None => return Err("Nu am primit lista de rezultate.".to_string()),
    };
    let loc = match results.first() {
        Some(l) => l.clone(),
        None => return Err("Orașul nu a fost găsit.".to_string()),
    };
    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m",
        loc.latitude, loc.longitude
    );
    let w_resp = match reqwest::get(&weather_url).await {
        Ok(resp) => resp,
        Err(e) => return Err(format!("Eroare rețea Vreme: {}", e)),
    };
    let w_data = match w_resp.json::<WeatherResponse>().await {
        Ok(data) => data,
        Err(e) => return Err(format!("Eroare JSON Vreme: {}", e)),
    };
    Ok(w_data.current.temperature_2m)
}
struct App {
    city_name: String,
    promise: Option<Promise<Result<f32, String>>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            city_name: "Bucuresti".to_owned(),
            promise: None,
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
                    let city = self.city_name.clone();
                    self.promise = Some(Promise::spawn_async(
                        async move { fetch_weather(city).await },
                    ));
                }
                if let Some(promise) = &self.promise {
                    match promise.ready() {
                        Some(result) => match result {
                            Ok(temp) => {
                                ui.colored_label(
                                    egui::Color32::GREEN,
                                    egui::RichText::new(format!("{} C", temp)).size(24.0),
                                );
                            }
                            Err(err) => {
                                ui.colored_label(egui::Color32::RED, err);
                            }
                        },
                        None => {
                            ui.spinner();
                            ui.label("Se incarca...");
                        }
                    }
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
