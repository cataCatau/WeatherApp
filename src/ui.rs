use eframe::egui;
use egui::Color32;

pub fn get_card_style() -> egui::Frame {
    egui::Frame::none()
        .fill(egui::Color32::from_rgba_premultiplied(20, 30, 40, 180))
        .rounding(16.0)
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(30)))
        .inner_margin(10.0)
}
pub fn get_metric_color(
    val: f32,
    limit_good: f32,
    limit_fair: f32,
    limit_mod: f32,
    limit_poor: f32,
) -> Color32 {
    if val < limit_good {
        Color32::GREEN
    } else if val < limit_fair {
        Color32::YELLOW
    } else if val < limit_mod {
        Color32::from_rgb(255, 165, 0)
    } else if val < limit_poor {
        Color32::RED
    } else {
        Color32::from_rgb(82, 85, 89)
    }
}

pub fn get_pressure_color(val: f32) -> Color32 {
    if val < 1000.0 {
        Color32::RED
    } else if val < 1020.0 {
        Color32::GREEN
    } else {
        Color32::from_rgb(100, 200, 255)
    }
}

pub fn draw_button(
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

pub fn draw_metric(ui: &mut egui::Ui, name: &str, val: f32, max: f32, unit: &str, color: Color32) {
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

pub fn get_weather_emoji(code: u32) -> &'static str {
    match code {
        0 => "☀️",
        1 => "🌤️",
        2 => "⛅",
        3 => "☁️",
        45 | 48 => "🌫️",
        51 | 53 | 55 => "🌧️",
        56 | 57 => "🌨️",
        61 | 63 | 65 => "🌧️",
        66 | 67 => "🌨️",
        71 | 73 | 75 => "❄️",
        77 => "🌨️",
        80..82 => "🌦️",
        85 | 86 => "🌨️",
        95 => "⛈️",
        96 | 99 => "⛈️",
        _ => "❓",
    }
}
