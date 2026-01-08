use eframe::egui;
use egui::{Align, Color32, RichText};
use poll_promise::Promise;

use crate::api::{self, CurrentAqi, CurrentData, DailyForecast, FullWeatherData, HourlyData};
use crate::favorites::Favorites;
use crate::ui;

#[derive(Debug, PartialEq, Clone)]
pub enum AppState {
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

pub struct App {
    pub city_name: String,
    pub cur_state: AppState,
    pub promise: Option<Promise<Result<FullWeatherData, String>>>,
    pub favorites: Favorites,
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

impl App {
    fn show_search_screen(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
            ui::get_card_style().show(ui, |ui| {
                ui.set_max_width(300.0);
                ui.heading(
                    RichText::new("Weather Dashboard")
                        .size(24.0)
                        .strong()
                        .color(Color32::WHITE),
                );
            });
            ui.add_space(250.0);
            ui::get_card_style().show(ui, |ui| {
                ui.set_max_width(300.0);
                ui.label(
                    RichText::new("Search a location")
                        .size(24.0)
                        .color(Color32::WHITE),
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
                        if ui::draw_button(ui, "🔎", Color32::WHITE, 24.0).clicked()
                            || (response.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                        {
                            let city = self.city_name.clone();
                            self.cur_state = AppState::Loading;
                            self.promise = Some(Promise::spawn_async(async move {
                                api::fetch_weather(city).await
                            }));
                        }
                    });
                });
            });

            ui.scope(|ui| {
                ui.set_max_width(300.0);
                ui.collapsing(
                    RichText::new("⭐ Favorites")
                        .strong()
                        .color(Color32::GOLD)
                        .size(24.0),
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
                                                    if ui::draw_button(
                                                        ui,
                                                        &location,
                                                        Color32::WHITE,
                                                        20.0,
                                                    )
                                                    .clicked()
                                                    {
                                                        self.city_name = location.clone();
                                                        self.cur_state = AppState::Loading;
                                                        let location_clone = location.clone();
                                                        self.promise = Some(Promise::spawn_async(
                                                            async move {
                                                                api::fetch_weather(location_clone)
                                                                    .await
                                                            },
                                                        ));
                                                    }
                                                    if ui::draw_button(ui, "❌", Color32::RED, 20.0)
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

    fn show_loading_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(150.0);
            ui.add(egui::Spinner::new().size(60.0).color(Color32::WHITE));
            ui.add_space(20.0);
            ui.label(
                RichText::new("Please wait...")
                    .size(20.0)
                    .color(Color32::WHITE),
            );
        });
    }

    fn show_results_screen(
        &mut self,
        ui: &mut egui::Ui,
        location: &String,
        current: &CurrentData,
        hourly: &HourlyData,
        forecast: &DailyForecast,
        aqi: &CurrentAqi,
    ) {
        ui.vertical_centered(|ui| {
            ui::get_card_style().show(ui, |ui| {
                ui.set_width(350.0);
                ui.set_max_height(100.0);
                ui.with_layout(
                    egui::Layout::top_down(Align::Center).with_main_align(Align::Center),
                    |ui| {
                        ui.heading(
                            RichText::new(location)
                                .size(32.0)
                                .strong()
                                .color(Color32::WHITE),
                        );

                        let is_fav = self.favorites.locations.contains(location);
                        let (btn_text, btn_color) = if is_fav {
                            ("☆", Color32::GOLD)
                        } else {
                            ("☆", Color32::GRAY)
                        };

                        if ui::draw_button(ui, btn_text, btn_color, 24.0).clicked() {
                            if is_fav {
                                self.favorites.remove(location);
                            } else {
                                self.favorites.add(location.clone());
                            }
                        }
                        ui.label(
                            RichText::new(format!("{} °C", current.temperature_2m))
                                .size(28.0)
                                .color(Color32::from_rgb(100, 200, 255)),
                        );
                    },
                );
            });
        });

        ui.add_space(50.0);

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui::get_card_style().show(ui, |ui| {
                    egui::ScrollArea::horizontal()
                        .max_height(300.0)
                        .auto_shrink([true, true])
                        .show(ui, |ui| {
                            for i in 0..24 {
                                ui.set_width(151.0);
                                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                                    let only_hour = match hourly.time[i].split('T').next_back() {
                                        Some(h) => h,
                                        None => &hourly.time[i],
                                    };
                                    ui.colored_label(
                                        Color32::WHITE,
                                        RichText::new(only_hour).strong(),
                                    );
                                    ui.label(
                                        RichText::new(format!("{}°C", hourly.temperature_2m[i]))
                                            .size(24.0)
                                            .strong()
                                            .color(Color32::WHITE),
                                    );
                                });
                                ui.add_space(50.0);
                            }
                        });
                });
            });
        });

        ui.columns(3, |cols| {
            cols[0].with_layout(
                egui::Layout::top_down(Align::LEFT).with_main_wrap(false),
                |ui| {
                    ui.add_space(20.0);
                    ui::get_card_style().show(ui, |ui| {
                        ui.set_width(100.0);
                        ui.heading("Daily Forecast");
                        for i in 0..7 {
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.colored_label(
                                    Color32::WHITE,
                                    RichText::new(&forecast.time[i]).strong().size(20.0),
                                );
                                ui.colored_label(
                                    Color32::LIGHT_BLUE,
                                    RichText::new(format!(
                                        "MIN : {:?}",
                                        forecast.temperature_2m_min[i]
                                    ))
                                    .size(20.0),
                                );
                                ui.colored_label(
                                    Color32::RED,
                                    RichText::new(format!(
                                        "MAX : {:?}",
                                        forecast.temperature_2m_max[i]
                                    ))
                                    .size(20.0),
                                );
                            });
                            ui.add_space(10.0);
                        }
                    });
                },
            );

            cols[1].vertical_centered(|ui| {
                ui.add_space(20.0);
                ui::get_card_style().show(ui, |ui| {
                    ui.set_width(250.0);
                    ui.set_max_height(400.0);
                    let color = ui::get_metric_color(aqi.european_aqi, 20.0, 40.0, 60.0, 80.0);
                    ui.colored_label(
                        Color32::WHITE,
                        RichText::new("Air Quality Index").size(24.0).strong(),
                    );
                    ui.colored_label(
                        color,
                        RichText::new(format!("{}", aqi.european_aqi)).size(40.0),
                    );
                    ui.separator();
                    ui.add_space(20.0);

                    ui.with_layout(egui::Layout::top_down(Align::LEFT), |ui| {
                        let c = ui::get_metric_color(aqi.pm2_5, 10.0, 20.0, 25.0, 50.0);
                        ui::draw_metric(ui, "PM 2.5", aqi.pm2_5, 100.0, " ", c);
                        let c = ui::get_metric_color(
                            aqi.carbon_monoxide,
                            1000.0,
                            2500.0,
                            5000.0,
                            10000.0,
                        );
                        ui::draw_metric(
                            ui,
                            "Carbon Monoxide",
                            aqi.carbon_monoxide,
                            10000.0,
                            " ",
                            c,
                        );
                        let c =
                            ui::get_metric_color(aqi.nitrogen_dioxide, 40.0, 90.0, 120.0, 230.0);
                        ui::draw_metric(
                            ui,
                            "Nitrogen Dioxide",
                            aqi.nitrogen_dioxide,
                            340.0,
                            " ",
                            c,
                        );
                        let c =
                            ui::get_metric_color(aqi.sulphur_dioxide, 100.0, 200.0, 350.0, 500.0);
                        ui::draw_metric(ui, "Sulphur Dioxide", aqi.sulphur_dioxide, 750.0, " ", c);
                        let c = ui::get_metric_color(aqi.ozone, 50.0, 100.0, 130.0, 240.0);
                        ui::draw_metric(ui, "Ozone", aqi.ozone, 380.0, " ", c);
                    });
                });
            });

            cols[2].with_layout(egui::Layout::top_down(Align::RIGHT), |ui| {
                ui.add_space(20.0);
                ui.set_width(300.0);
                ui::get_card_style().show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down(Align::LEFT), |ui| {
                        ui.add_space(5.0);
                        ui.colored_label(
                            Color32::WHITE,
                            RichText::new(ui::get_weather_emoji(current.weather_code)).size(80.0),
                        );
                        ui.separator();
                        ui.add_space(20.0);

                        let h = match forecast.sunrise[0].split('T').next_back() {
                            Some(h) => h,
                            None => &forecast.sunrise[0],
                        };
                        ui.colored_label(
                            Color32::WHITE,
                            RichText::new(format!("Sunrise 🌅 : {}", h)).size(24.0),
                        );
                        ui.add_space(5.0);

                        let c =
                            ui::get_metric_color(current.wind_speed_10m, 20.0, 40.0, 60.0, 80.0);
                        ui::draw_metric(
                            ui,
                            "Wind Speed 💨 ",
                            current.wind_speed_10m,
                            100.0,
                            "km/h",
                            c,
                        );
                        let c = ui::get_metric_color(current.precipitation, 2.5, 5.0, 10.0, 15.0);
                        ui::draw_metric(
                            ui,
                            "Precipitation 💧",
                            current.precipitation,
                            50.0,
                            "mm",
                            c,
                        );
                        let c = ui::get_metric_color(forecast.uv_index_max[0], 2.0, 5.0, 7.0, 10.0);
                        ui::draw_metric(
                            ui,
                            "Max UV Index 🔆",
                            forecast.uv_index_max[0],
                            11.0,
                            " ",
                            c,
                        );

                        ui.colored_label(
                            Color32::WHITE,
                            RichText::new("Pressure 🕒").size(24.0).strong(),
                        );
                        let p = ((current.surface_pressure - 950.0) / 100.0).clamp(0.0, 1.0);
                        ui.add(
                            egui::ProgressBar::new(p)
                                .fill(ui::get_pressure_color(current.surface_pressure))
                                .desired_width(250.0)
                                .text(
                                    RichText::new(format!("{} hPa", current.surface_pressure))
                                        .color(Color32::BLACK),
                                ),
                        );
                    });
                });
            });
        });

        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui::get_card_style().show(ui, |ui| {
                ui.set_max_width(10.0);
                if ui::draw_button(ui, "🏠", Color32::WHITE, 40.0).clicked() {
                    self.cur_state = AppState::Search;
                }
            });
        });
    }
    fn show_error_screen(&mut self, ui: &mut egui::Ui, err_msg: String) {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.heading(RichText::new("error").color(Color32::RED).size(30.0));
            ui.add_space(20.0);
            ui.label(RichText::new(&err_msg).size(18.0));
            ui.add_space(20.0);
            if ui.button("Back").clicked() {
                self.cur_state = AppState::Search;
            }
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::light();
        style.visuals.window_fill = Color32::from_rgb(15, 23, 42);
        style.visuals.panel_fill = Color32::from_rgb(15, 23, 42);
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
                self.promise = Some(promise);
                ctx.request_repaint();
            }
        }

        let state = self.cur_state.clone();

        egui::CentralPanel::default().show(ctx, |ui| match state {
            AppState::Search => self.show_search_screen(ui),
            AppState::Loading => self.show_loading_screen(ui),
            AppState::Result {
                location,
                current,
                hourly,
                forecast,
                aqi,
            } => {
                self.show_results_screen(ui, &location, &current, &hourly, &forecast, &aqi);
            }
            AppState::Error(err_msg) => {
                self.show_error_screen(ui, err_msg);
            }
        });
    }
}
