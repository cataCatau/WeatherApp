use eframe::egui;
use egui::Align;
use poll_promise::Promise;
use serde::Deserialize;
#[derive(Debug, PartialEq, Clone)]
enum AppState {
    Search,
    Loading,
    Result {
        location: String,
        temperature: f32,
        forecast: DailyForecast,
    },
    Error(String),
}
#[derive(Deserialize, Debug)]
struct GeoSearch {
    results: Option<Vec<GeoLocation>>,
}
#[derive(Deserialize, Debug, Clone)]
struct GeoLocation {
    latitude: f64,
    longitude: f64,
}
#[derive(Deserialize, Debug)]
struct WeatherResponse {
    current: CurrentData,
    daily: DailyForecast,
}

#[derive(Deserialize, Debug)]
struct CurrentData {
    temperature_2m: f32,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct DailyForecast {
    time: Vec<String>,
    temperature_2m_min: Vec<f32>,
    temperature_2m_max: Vec<f32>,
}

async fn fetch_weather(city: String) -> Result<(f32, DailyForecast), String> {
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
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m&daily=temperature_2m_max,temperature_2m_min&timezone=auto&forecast_days=7",
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
    Ok((w_data.current.temperature_2m, w_data.daily))
}
struct App {
    city_name: String,
    cur_state: AppState,
    promise: Option<Promise<Result<(f32, DailyForecast), String>>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            city_name: "Bucuresti".to_owned(),
            cur_state: AppState::Search,
            promise: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(promise) = self.promise.take() {
            if let Some(result) = promise.ready() {
                self.cur_state = match result {
                    Ok((temp, fore)) => AppState::Result {
                        location: self.city_name.clone(),
                        temperature: *temp,
                        forecast: fore.clone(),
                    },
                    Err(err) => AppState::Error(err.clone()),
                };
            } else {
                // adica daca nu e gata promise-ul
                self.promise = Some(promise);
                ctx.request_repaint();
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| match &self.cur_state {
            AppState::Search => {
                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.heading(egui::RichText::new("Weather Dashboard").size(24.0).strong());
                    ui.add_space(250.0);
                    ui.label(egui::RichText::new("Introduceti orasul").size(24.0));
                    ui.text_edit_singleline(&mut self.city_name);

                    if ui.button("search").clicked() {
                        let city = self.city_name.clone();
                        self.cur_state = AppState::Loading;
                        self.promise =
                            Some(Promise::spawn_async(
                                async move { fetch_weather(city).await },
                            ));
                    }
                });
            }
            AppState::Loading => {
                ui.spinner();
                ui.label("please wait");
            }
            AppState::Result {
                location,
                temperature,
                forecast,
            } => {
                ui.heading(format!("vremea in {}", location));
                ui.add_space(10.0);
                ui.colored_label(
                    egui::Color32::GREEN,
                    egui::RichText::new(format!("{} C", temperature)).size(24.0),
                );
                ui.with_layout(
                    egui::Layout::left_to_right(egui::Align::TOP).with_main_wrap(false),
                    |ui| {
                        for i in 0..7 {
                            egui::Frame::group(ui.style()).show(ui, |ui| {
                                ui.set_max_width(150.0);
                                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                    ui.label(
                                        egui::RichText::new(format!("{}", forecast.time[i]))
                                            .strong(),
                                    );
                                    ui.label(egui::RichText::new("MAX").size(10.0));
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{}°C",
                                            forecast.temperature_2m_max[i]
                                        ))
                                        .size(24.0)
                                        .strong()
                                        .color(egui::Color32::RED),
                                    );
                                    ui.label(egui::RichText::new("MIN").size(10.0));
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{}°C",
                                            forecast.temperature_2m_min[i]
                                        ))
                                        .size(18.0)
                                        .color(egui::Color32::BLUE),
                                    );
                                });
                            });
                            ui.add_space(10.0);
                        }
                    },
                );

                if ui.button("back").clicked() {
                    self.cur_state = AppState::Search;
                }
            }

            AppState::Error(_err) => {
                ui.heading("eroare");
            }
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
