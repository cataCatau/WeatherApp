use eframe::egui;
use egui::{Align, Color32};
use poll_promise::Promise;
use serde::Deserialize;

type FullWeatherData = (CurrentData, HourlyData, DailyForecast, CurrentAqi);
#[derive(Debug, PartialEq, Clone)]
enum AppState {
    Search,
    Loading,
    Result {
        location: String,
        current: CurrentData,
        hourly: HourlyData,
        forecast: DailyForecast,
        aqi: CurrentAqi,
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
    hourly: HourlyData,
    daily: DailyForecast,
}
#[derive(Deserialize, Debug, PartialEq)]
struct AirQualityResponse {
    current: CurrentAqi,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct CurrentAqi {
    european_aqi: f32,
    pm2_5: f32,
    carbon_monoxide: f32,
    nitrogen_dioxide: f32,
    sulphur_dioxide: f32,
    ozone: f32,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
struct CurrentData {
    temperature_2m: f32,
}
#[derive(Deserialize, Debug, Clone, PartialEq)]
struct HourlyData {
    time: Vec<String>,
    temperature_2m: Vec<f32>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct DailyForecast {
    time: Vec<String>,
    temperature_2m_min: Vec<f32>,
    temperature_2m_max: Vec<f32>,
}

async fn fetch_weather(city: String) -> Result<FullWeatherData, String> {
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
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m&daily=temperature_2m_max,temperature_2m_min&timezone=auto&forecast_days=7&hourly=temperature_2m",
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
    let poluation_url = format!(
        "https://air-quality-api.open-meteo.com/v1/air-quality?latitude={}&longitude={}&current=european_aqi,pm2_5,carbon_monoxide,nitrogen_dioxide,sulphur_dioxide,ozone",
        loc.latitude, loc.longitude
    );
    let aqi_resp = match reqwest::get(&poluation_url).await {
        Ok(resp) => resp,
        Err(e) => return Err(format!("Eroare rețea Vreme: {}", e)),
    };
    let aqi_data = match aqi_resp.json::<AirQualityResponse>().await {
        Ok(data) => data,
        Err(e) => return Err(format!("Eroare JSON Vreme: {}", e)),
    };
    Ok((
        w_data.current,
        w_data.hourly,
        w_data.daily,
        aqi_data.current,
    ))
}
struct App {
    city_name: String,
    cur_state: AppState,
    promise: Option<Promise<Result<FullWeatherData, String>>>,
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
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::light();
        style.visuals.window_fill = egui::Color32::from_rgb(0, 160, 225);
        style.visuals.panel_fill = egui::Color32::from_rgb(0, 160, 225);
        ctx.set_style(style);
        if let Some(promise) = self.promise.take() {
            if let Some(result) = promise.ready() {
                self.cur_state = match result {
                    Ok((cur, hour, fore, aqi_val)) => AppState::Result {
                        location: self.city_name.clone(),
                        current: cur.clone(),
                        hourly: hour.clone(),
                        forecast: fore.clone(),
                        aqi: aqi_val.clone(),
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
                    ui.label(egui::RichText::new("Introduceti locatia").size(24.0));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.city_name)
                            .font(egui::FontId::proportional(20.0))
                            .desired_width(300.0)
                            .horizontal_align(egui::Align::Center),
                    );

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
                current,
                forecast,
                hourly,
                aqi,
            } => {
                let card_style = egui::Frame::none()
                    .fill(egui::Color32::from_rgba_premultiplied(20, 30, 40, 180))
                    .rounding(egui::Rounding::same(16.0))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(30)))
                    .inner_margin(10.0);
                ui.vertical_centered(|ui| {
                    card_style.show(ui, |ui| {
                        ui.set_max_width(150.0);
                        ui.colored_label(
                            egui::Color32::WHITE,
                            egui::RichText::new(location.to_string())
                                .size(32.0)
                                .strong(),
                        );
                        ui.colored_label(
                            egui::Color32::WHITE,
                            egui::RichText::new(format!("{} °C", current.temperature_2m))
                                .size(24.0),
                        );
                    });
                });

                ui.add_space(50.0);
                ui.horizontal(|ui| {
                    egui::ScrollArea::horizontal()
                        .max_height(300.0)
                        .auto_shrink([true, true])
                        .show(ui, |ui| {
                            for i in 0..24 {
                                card_style.show(ui, |ui| {
                                    ui.set_max_width(200.0);
                                    ui.with_layout(
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            let only_hour = match hourly.time[i].split('T').last() {
                                                Some(hour) => hour,
                                                None => &hourly.time[i],
                                            };
                                            ui.colored_label(
                                                egui::Color32::WHITE,
                                                egui::RichText::new(only_hour).strong(),
                                            );
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "{}°C",
                                                    hourly.temperature_2m[i]
                                                ))
                                                .size(24.0)
                                                .strong()
                                                .color(egui::Color32::WHITE),
                                            );
                                        },
                                    );
                                });
                                ui.add_space(25.0);
                            }
                        });
                });
                ui.horizontal_top(|ui| {
                    ui.with_layout(
                        egui::Layout::top_down(egui::Align::LEFT).with_main_wrap(false),
                        |ui| {
                            ui.add_space(20.0);
                            for i in 0..7 {
                                card_style.show(ui, |ui| {
                                    ui.set_max_width(150.0);
                                    ui.with_layout(
                                        egui::Layout::top_down(egui::Align::Center),
                                        |ui| {
                                            ui.colored_label(
                                                egui::Color32::WHITE,
                                                egui::RichText::new(forecast.time[i].to_string())
                                                    .strong(),
                                            );
                                            ui.colored_label(
                                                egui::Color32::WHITE,
                                                egui::RichText::new(format!(
                                                    "MIN : {:?}   MAX : {:?} ",
                                                    forecast.temperature_2m_min[i],
                                                    forecast.temperature_2m_max[i]
                                                ))
                                                .size(10.0),
                                            );
                                        },
                                    );
                                });
                                ui.add_space(20.0);
                            }
                        },
                    );
                    ui.add_space(430.0);
                    ui.vertical(|ui| {
                        ui.add_space(20.0);
                        card_style.show(ui, |ui| {
                            ui.set_width(250.0);
                            ui.set_height(200.0);
                            ui.set_max_height(300.0);

                            fn get_pollution_color(
                                val: f32,
                                limit_good: f32,
                                limit_fair: f32,
                                limit_mod: f32,
                                limit_poor: f32,
                            ) -> egui::Color32 {
                                if val < limit_good {
                                    egui::Color32::LIGHT_GREEN
                                } else if val < limit_fair {
                                    egui::Color32::GREEN
                                } else if val < limit_mod {
                                    egui::Color32::YELLOW
                                } else if val < limit_poor {
                                    egui::Color32::RED
                                } else {
                                    egui::Color32::BLACK
                                }
                            }
                            //aqi
                            let color =
                                get_pollution_color(aqi.european_aqi, 20.0, 40.0, 60.0, 80.0);
                            ui.colored_label(
                                Color32::WHITE,
                                egui::RichText::new("Air Quality Index").size(24.0).strong(),
                            );
                            let progress = (aqi.european_aqi / 100.0).clamp(0.0, 1.0);
                            let progress_bar = egui::ProgressBar::new(progress)
                                .fill(color)
                                .desired_width(200.0)
                                .text(format!("AQI: {}", aqi.european_aqi));
                            ui.add(progress_bar);
                            //pm2.5
                            let color = get_pollution_color(aqi.pm2_5, 10.0, 20.0, 25.0, 50.0);
                            ui.colored_label(
                                Color32::WHITE,
                                egui::RichText::new("PM 2.5").size(24.0).strong(),
                            );
                            let progress = (aqi.pm2_5 / 100.0).clamp(0.0, 1.0);
                            let progress_bar = egui::ProgressBar::new(progress)
                                .fill(color)
                                .desired_width(200.0)
                                .text(format!("PM 2.5: {}", aqi.pm2_5));
                            ui.add(progress_bar);
                            //CO
                            let color = get_pollution_color(
                                aqi.carbon_monoxide,
                                1000.0,
                                2500.0,
                                5000.0,
                                10000.0,
                            );
                            ui.colored_label(
                                Color32::WHITE,
                                egui::RichText::new("Carbon Monoxide").size(24.0).strong(),
                            );
                            let progress = (aqi.carbon_monoxide / 10000.0).clamp(0.0, 1.0);
                            let progress_bar = egui::ProgressBar::new(progress)
                                .fill(color)
                                .desired_width(200.0)
                                .text(format!("CO: {}", aqi.carbon_monoxide));
                            ui.add(progress_bar);

                            //NO2
                            let color =
                                get_pollution_color(aqi.nitrogen_dioxide, 40.0, 90.0, 120.0, 230.0);
                            ui.colored_label(
                                Color32::WHITE,
                                egui::RichText::new("Nitrogen Dioxide").size(24.0).strong(),
                            );
                            let progress = (aqi.nitrogen_dioxide / 340.0).clamp(0.0, 1.0);
                            let progress_bar = egui::ProgressBar::new(progress)
                                .fill(color)
                                .desired_width(200.0)
                                .text(format!("NO2: {}", aqi.nitrogen_dioxide));
                            ui.add(progress_bar);

                            //SO2
                            let color = get_pollution_color(
                                aqi.sulphur_dioxide,
                                100.0,
                                200.0,
                                350.0,
                                500.0,
                            );
                            ui.colored_label(
                                Color32::WHITE,
                                egui::RichText::new("Sulphur Dioxide").size(24.0).strong(),
                            );
                            let progress = (aqi.sulphur_dioxide / 750.0).clamp(0.0, 1.0);
                            let progress_bar = egui::ProgressBar::new(progress)
                                .fill(color)
                                .desired_width(200.0)
                                .text(format!("SO2: {}", aqi.sulphur_dioxide));
                            ui.add(progress_bar);

                            //O3

                            let color = get_pollution_color(aqi.ozone, 50.0, 100.0, 130.0, 240.0);
                            ui.colored_label(
                                Color32::WHITE,
                                egui::RichText::new("Ozone").size(24.0).strong(),
                            );
                            let progress = (aqi.ozone / 380.0).clamp(0.0, 1.0);
                            let progress_bar = egui::ProgressBar::new(progress)
                                .fill(color)
                                .desired_width(200.0)
                                .text(format!("O3: {}", aqi.ozone));
                            ui.add(progress_bar);
                        });
                    });
                });

                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    card_style.show(ui, |ui| {
                        ui.set_max_width(150.0);
                        if ui.button("back").clicked() {
                            self.cur_state = AppState::Search;
                        }
                    });
                });
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
