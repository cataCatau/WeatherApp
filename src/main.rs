use eframe::egui;
use egui::{Align, Color32, RichText};
use poll_promise::Promise;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

type FullWeatherData = (CurrentData, HourlyData, DailyForecast, CurrentAqi);
#[derive(Debug, PartialEq, Clone)]
enum AppState {
    Search,
    Loading,
    Result {
        location: String,
        current: CurrentData,
        hourly: HourlyData,
        forecast: Box<DailyForecast>,
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
    weather_code: u32,
    surface_pressure: f32,
    wind_speed_10m: f32,
    precipitation: f32,
}
#[derive(Deserialize, Debug, Clone, PartialEq)]
struct HourlyData {
    time: Vec<String>,
    temperature_2m: Vec<f32>,
}

//67
#[derive(Deserialize, Debug, Clone, PartialEq)]
struct DailyForecast {
    time: Vec<String>,
    temperature_2m_min: Vec<f32>,
    temperature_2m_max: Vec<f32>,
    uv_index_max: Vec<f32>,
    sunrise: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct Favorites {
    locations: Vec<String>,
}

impl Favorites {
    fn load() -> Self {
        if Path::new("favorites.json").exists()
            && let Ok(content) = fs::read_to_string("favorites.json")
            && let Ok(favs) = serde_json::from_str(&content)
        {
            return favs;
        }

        Self::default()
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write("favorites.json", json);
        }
    }

    fn add(&mut self, city: String) {
        if !self.locations.contains(&city) {
            self.locations.push(city);
            self.save();
        }
    }

    fn remove(&mut self, city: &String) {
        self.locations.retain(|x| x != city);
        self.save();
    }
}

fn get_metric_color(
    val: f32,
    limit_good: f32,
    limit_fair: f32,
    limit_mod: f32,
    limit_poor: f32,
) -> egui::Color32 {
    if val < limit_good {
        egui::Color32::GREEN
    } else if val < limit_fair {
        egui::Color32::YELLOW
    } else if val < limit_mod {
        egui::Color32::from_rgb(255, 165, 0)
    } else if val < limit_poor {
        egui::Color32::RED
    } else {
        egui::Color32::from_rgb(82, 85, 89)
    }
}
fn get_pressure_color(val: f32) -> Color32 {
    if val < 1000.0 {
        Color32::RED // presiune mica (furtuna)
    } else if val < 1020.0 {
        Color32::GREEN // presiune normala
    } else {
        Color32::from_rgb(100, 200, 255) //presiune mare (anticiclon, cer senin)
    }
}
fn draw_button(
    ui: &mut egui::Ui,
    text: &str,
    textcolor: egui::Color32,
    size: f32,
) -> egui::Response {
    ui.scope(|ui| {
        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;

        ui.button(egui::RichText::new(text).size(size).color(textcolor))
    })
    .inner
}
fn draw_metric(ui: &mut egui::Ui, name: &str, val: f32, max: f32, unit: &str, color: Color32) {
    ui.colored_label(
        Color32::WHITE,
        egui::RichText::new(name).size(24.0).strong(),
    );
    let progress = (val / max).clamp(0.0, 1.0);
    let progress_bar = egui::ProgressBar::new(progress)
        .fill(color)
        .desired_width(250.0)
        .text(egui::RichText::new(format!("{} {}", val, unit)).color(egui::Color32::BLACK));
    ui.add(progress_bar);
}
fn get_weather_emoji(code: u32) -> &'static str {
    match code {
        0 => "☀️",            // Cer senin
        1 => "🌤️",            // Predominant senin
        2 => "⛅",            // Parțial noros
        3 => "☁️",            // Înnorat
        45 | 48 => "🌫️",      // Ceață
        51 | 53 | 55 => "🌧️", // Burniță (am scos textul "drizzle")
        56 | 57 => "🌨️",      // Burniță înghețată (doar un icon)
        61 | 63 | 65 => "🌧️", // Ploaie
        66 | 67 => "🌨️",      // Ploaie înghețată
        71 | 73 | 75 => "❄️", // Ninsoare
        77 => "🌨️",           // Grăunțe de zăpadă
        80..82 => "🌦️",       // Averse de ploaie
        85 | 86 => "🌨️",      // Averse de zăpadă
        95 => "⛈️",           // Furtună
        96 | 99 => "⛈️",      // Furtună cu grindină
        _ => "❓",            // Necunoscut
    }
}

async fn fetch_weather(city: String) -> Result<FullWeatherData, String> {
    let geo_url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=ro&format=json",
        city
    );
    let geo_resp = match reqwest::get(&geo_url).await {
        Ok(resp) => resp,
        Err(e) => return Err(format!("GEO network error: {}", e)),
    };
    let geo_data = match geo_resp.json::<GeoSearch>().await {
        Ok(data) => data,
        Err(e) => return Err(format!("JSON parsing error: {}", e)),
    };
    let results = match geo_data.results {
        Some(r) => r,
        None => return Err("No results received".to_string()),
    };
    let loc = match results.first() {
        Some(l) => l.clone(),
        None => return Err("City not found".to_string()),
    };
    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code,surface_pressure,wind_speed_10m,precipitation&daily=temperature_2m_max,temperature_2m_min,uv_index_max,sunrise&timezone=auto&forecast_days=7&hourly=temperature_2m",
        loc.latitude, loc.longitude
    );
    let poluation_url = format!(
        "https://air-quality-api.open-meteo.com/v1/air-quality?latitude={}&longitude={}&current=european_aqi,pm2_5,carbon_monoxide,nitrogen_dioxide,sulphur_dioxide,ozone",
        loc.latitude, loc.longitude
    );
    let w_task = async {
        let w_resp = match reqwest::get(&weather_url).await {
            Ok(resp) => resp,
            Err(e) => return Err(format!("Weather network error: {}", e)),
        };
        match w_resp.json::<WeatherResponse>().await {
            Ok(data) => Ok(data),
            Err(e) => Err(format!("Weather JSON parsing error: {}", e)),
        }
    };
    let aqi_task = async {
        let aqi_resp = match reqwest::get(&poluation_url).await {
            Ok(resp) => resp,
            Err(e) => return Err(format!("AQI Network error: {}", e)),
        };
        match aqi_resp.json::<AirQualityResponse>().await {
            Ok(data) => Ok(data),
            Err(e) => Err(format!("AQI JSON Parsing error:  {}", e)),
        }
    };
    let (w_result, aqi_result) = tokio::join!(w_task, aqi_task);
    let w_data = match w_result {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    let aqi_data = match aqi_result {
        Ok(data) => data,
        Err(e) => return Err(e),
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
    favorites: Favorites,
}

impl Default for App {
    fn default() -> Self {
        Self {
            city_name: "".to_owned(),
            cur_state: AppState::Search,
            promise: None,
            favorites: Favorites::load(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::light();
        style.visuals.window_fill = egui::Color32::from_rgb(15, 23, 42);
        style.visuals.panel_fill = egui::Color32::from_rgb(15, 23, 42);
        ctx.set_style(style);
        if let Some(promise) = self.promise.take() {
            if let Some(result) = promise.ready() {
                self.cur_state = match result {
                    Ok((cur, hour, fore, aqi_val)) => AppState::Result {
                        location: self.city_name.clone(),
                        current: cur.clone(),
                        hourly: hour.clone(),
                        forecast: Box::new(fore.clone()),
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
        let mut next_state = None;
        egui::CentralPanel::default().show(ctx, |ui| match &self.cur_state {
            AppState::Search => {
                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.heading(
                        egui::RichText::new("Weather Dashboard")
                            .size(24.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    );
                    ui.add_space(250.0);
                    ui.label(
                        egui::RichText::new("Search a location")
                            .size(24.0)
                            .color(egui::Color32::WHITE),
                    );
                    ui.scope(|ui| {
                        ui.set_max_width(340.0);
                        ui.horizontal(|ui| {
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut self.city_name)
                                    .font(egui::FontId::proportional(20.0))
                                    .desired_width(300.0)
                                    .horizontal_align(egui::Align::Center),
                            );
                            if draw_button(ui, "🔎", egui::Color32::WHITE, 24.0).clicked()
                                || (response.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                            {
                                let city = self.city_name.clone();
                                self.cur_state = AppState::Loading;
                                self.promise =
                                    Some(Promise::spawn_async(
                                        async move { fetch_weather(city).await },
                                    ));
                            }
                        });
                    });

                    ui.scope(|ui| {
                        ui.set_max_width(300.0);
                        ui.collapsing(
                            RichText::new("⭐ Favorites").strong().color(Color32::GOLD),
                            |ui| {
                                ui.add_space(10.0);
                                egui::ScrollArea::vertical()
                                    .max_height(150.0)
                                    .auto_shrink([false, true])
                                    .show(ui, |ui| {
                                        let favs = self.favorites.locations.clone();

                                        for location in favs {
                                            ui.horizontal(|ui| {
                                                ui.set_width(300.0);

                                                ui.with_layout(
                                                    egui::Layout::top_down(Align::Center),
                                                    |ui| {
                                                        ui.horizontal(|ui| {
                                                            if draw_button(
                                                                ui,
                                                                &location,
                                                                egui::Color32::WHITE,
                                                                20.0,
                                                            )
                                                            .clicked()
                                                            {
                                                                self.city_name = location.clone();
                                                                self.cur_state = AppState::Loading;
                                                                let location_clone =
                                                                    location.clone();
                                                                self.promise =
                                                                    Some(Promise::spawn_async(
                                                                        async move {
                                                                            fetch_weather(
                                                                                location_clone,
                                                                            )
                                                                            .await
                                                                        },
                                                                    ));
                                                            }

                                                            if draw_button(
                                                                ui,
                                                                "❌",
                                                                egui::Color32::RED,
                                                                20.0,
                                                            )
                                                            .clicked()
                                                            {
                                                                self.favorites.remove(&location);
                                                            }
                                                        });
                                                    },
                                                );
                                            });
                                            ui.add_space(5.0);
                                        }
                                    });
                            },
                        );
                    });
                });
            }
            AppState::Loading => {
                ui.vertical_centered(|ui| {
                    ui.add_space(150.0);
                    ui.add(egui::Spinner::new().size(60.0).color(egui::Color32::WHITE));

                    ui.add_space(20.0);
                    ui.label(
                        RichText::new("Please wait...")
                            .size(20.0)
                            .color(egui::Color32::WHITE),
                    );
                });
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
                    .rounding(16.0)
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(30)))
                    .inner_margin(10.0);
                ui.vertical_centered(|ui| {
                    card_style.show(ui, |ui| {
                        ui.set_width(350.0);
                        ui.set_max_height(100.0);
                        ui.with_layout(
                            egui::Layout::top_down(egui::Align::Center)
                                .with_main_align(egui::Align::Center),
                            |ui| {
                                ui.heading(
                                    RichText::new(location)
                                        .size(32.0)
                                        .strong()
                                        .color(Color32::WHITE),
                                );

                                let is_fav = self.favorites.locations.contains(location);
                                let btn_text = "☆";
                                let btn_color = if is_fav { Color32::GOLD } else { Color32::GRAY };

                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new(btn_text).size(24.0).color(btn_color),
                                        )
                                        .frame(false),
                                    )
                                    .clicked()
                                {
                                    if is_fav {
                                        self.favorites.remove(location);
                                    } else {
                                        self.favorites.add(location.clone());
                                    }
                                }
                            },
                        );

                        ui.label(
                            RichText::new(format!("{} °C", current.temperature_2m))
                                .size(28.0)
                                .color(Color32::from_rgb(100, 200, 255)),
                        );
                    });
                });
                ui.add_space(50.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        card_style.show(ui, |ui| {
                            egui::ScrollArea::horizontal()
                                .max_height(300.0)
                                .auto_shrink([true, true])
                                .show(ui, |ui| {
                                    for i in 0..24 {
                                        ui.set_max_width(200.0);
                                        ui.with_layout(
                                            egui::Layout::top_down(egui::Align::Center),
                                            |ui| {
                                                let only_hour =
                                                    match hourly.time[i].split('T').next_back() {
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
                                        ui.add_space(25.0);
                                    }
                                });
                        });
                    });
                });
                ui.columns(3, |cols| {
                    cols[0].with_layout(
                        egui::Layout::top_down(egui::Align::LEFT).with_main_wrap(false),
                        |ui| {
                            ui.add_space(20.0);
                            card_style.show(ui, |ui| {
                                ui.set_width(100.0);
                                ui.heading("Daily Forecast");
                                for i in 0..7 {
                                    ui.separator();
                                    ui.horizontal(|ui| {
                                        ui.colored_label(
                                            egui::Color32::WHITE,
                                            egui::RichText::new(forecast.time[i].to_string())
                                                .strong()
                                                .size(20.0),
                                        );
                                        ui.colored_label(
                                            egui::Color32::LIGHT_BLUE,
                                            egui::RichText::new(format!(
                                                "MIN : {:?}",
                                                forecast.temperature_2m_min[i]
                                            ))
                                            .size(20.0),
                                        );
                                        ui.colored_label(
                                            egui::Color32::RED,
                                            egui::RichText::new(format!(
                                                "MAX : {:?}",
                                                forecast.temperature_2m_max[i]
                                            ))
                                            .size(20.0),
                                        );
                                        ui.add_space(10.0);
                                    });
                                    ui.add_space(10.0);
                                }
                            });
                        },
                    );

                    cols[1].vertical_centered(|ui| {
                        ui.add_space(20.0);
                        card_style.show(ui, |ui| {
                            ui.set_width(250.0);

                            ui.set_max_height(400.0);

                            //aqi
                            let color = get_metric_color(aqi.european_aqi, 20.0, 40.0, 60.0, 80.0);
                            ui.colored_label(
                                Color32::WHITE,
                                egui::RichText::new("Air Quality Index").size(24.0).strong(),
                            );
                            ui.colored_label(
                                color,
                                egui::RichText::new(format!("{}", aqi.european_aqi)).size(40.0),
                            );
                            ui.separator();
                            ui.add_space(20.0);
                            //pm2.5
                            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                let color = get_metric_color(aqi.pm2_5, 10.0, 20.0, 25.0, 50.0);
                                draw_metric(ui, "PM 2.5", aqi.pm2_5, 100.0, " ", color);
                                //CO
                                let color = get_metric_color(
                                    aqi.carbon_monoxide,
                                    1000.0,
                                    2500.0,
                                    5000.0,
                                    10000.0,
                                );
                                draw_metric(
                                    ui,
                                    "Carbon Monoxide",
                                    aqi.carbon_monoxide,
                                    10000.0,
                                    " ",
                                    color,
                                );

                                //NO2
                                let color = get_metric_color(
                                    aqi.nitrogen_dioxide,
                                    40.0,
                                    90.0,
                                    120.0,
                                    230.0,
                                );
                                draw_metric(
                                    ui,
                                    "Nitrogen Dioxide",
                                    aqi.nitrogen_dioxide,
                                    340.0,
                                    " ",
                                    color,
                                );

                                //SO2
                                let color = get_metric_color(
                                    aqi.sulphur_dioxide,
                                    100.0,
                                    200.0,
                                    350.0,
                                    500.0,
                                );
                                draw_metric(
                                    ui,
                                    "Sulphur Dioxide",
                                    aqi.sulphur_dioxide,
                                    750.0,
                                    " ",
                                    color,
                                );

                                //O3

                                let color = get_metric_color(aqi.ozone, 50.0, 100.0, 130.0, 240.0);

                                draw_metric(ui, "Ozone", aqi.ozone, 380.0, " ", color);
                            });
                        });
                    });
                    cols[2].with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                        ui.add_space(20.0);
                        ui.set_width(300.0);
                        card_style.show(ui, |ui| {
                            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                ui.add_space(5.0);
                                ui.colored_label(
                                    egui::Color32::WHITE,
                                    egui::RichText::new(get_weather_emoji(current.weather_code))
                                        .size(80.0),
                                );
                                ui.separator();
                                ui.add_space(20.0);
                                // sunrise
                                let only_hour = match forecast.sunrise[0].split('T').next_back() {
                                    Some(hour) => hour,
                                    None => &forecast.sunrise[0],
                                };
                                ui.colored_label(
                                    egui::Color32::WHITE,
                                    egui::RichText::new(format!("Sunrise 🌅 : {}", only_hour))
                                        .size(24.0),
                                );
                                ui.add_space(5.0);
                                //Wind Speed
                                let color = get_metric_color(
                                    current.wind_speed_10m,
                                    20.0,
                                    40.0,
                                    60.0,
                                    80.0,
                                );
                                draw_metric(
                                    ui,
                                    "Wind Speed 💨 ",
                                    current.wind_speed_10m,
                                    100.0,
                                    "km/h",
                                    color,
                                );
                                // Precipitation
                                let color =
                                    get_metric_color(current.precipitation, 2.5, 5.0, 10.0, 15.0);
                                draw_metric(
                                    ui,
                                    "Precipitation 💧",
                                    current.precipitation,
                                    50.0,
                                    "mm",
                                    color,
                                );
                                //Max UV
                                let color =
                                    get_metric_color(forecast.uv_index_max[0], 2.0, 5.0, 7.0, 10.0);
                                draw_metric(
                                    ui,
                                    "Max UV Index 🔆",
                                    forecast.uv_index_max[0],
                                    11.0,
                                    " ",
                                    color,
                                );
                                // Pressure
                                ui.colored_label(
                                    Color32::WHITE,
                                    egui::RichText::new("Pressure 🕒").size(24.0).strong(),
                                );

                                let press_min = 950.0;
                                let press_max = 1050.0;

                                let press_progress = ((current.surface_pressure - press_min)
                                    / (press_max - press_min))
                                    .clamp(0.0, 1.0);

                                ui.add(
                                    egui::ProgressBar::new(press_progress)
                                        .fill(get_pressure_color(current.surface_pressure))
                                        .desired_width(250.0)
                                        .text(
                                            egui::RichText::new(format!(
                                                "{} hPa",
                                                current.surface_pressure
                                            ))
                                            .color(egui::Color32::BLACK),
                                        ),
                                );
                            });
                        });
                    });
                });
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    card_style.show(ui, |ui| {
                        ui.set_max_width(10.0);
                        if draw_button(ui, "🏠", egui::Color32::WHITE, 40.0).clicked() {
                            next_state = Some(AppState::Search);
                        }
                    });
                });
            }

            AppState::Error(err_msg) => {
                let err_msg_clone = err_msg.clone();
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading(RichText::new("error").color(Color32::RED).size(30.0));
                    ui.add_space(20.0);
                    ui.label(RichText::new(&err_msg_clone).size(18.0));
                    ui.add_space(20.0);

                    if ui.button("Back").clicked() {
                        next_state = Some(AppState::Search);
                    }
                });
            }
        });

        if let Some(state) = next_state {
            self.cur_state = state;
        }
    }
}
fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "emoji_font".to_owned(),
        egui::FontData::from_static(include_bytes!("C:\\Windows\\Fonts\\seguiemj.ttf")),
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
async fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0])
            .with_title("WeatherDashboard"),
        ..Default::default()
    };

    eframe::run_native(
        "App",
        options,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);
            Box::new(App::default())
        }),
    )
}
