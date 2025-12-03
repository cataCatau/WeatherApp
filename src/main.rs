use eframe::egui;
use egui::Align;
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
            ui.heading(egui::RichText::new("Weather Dashboard").size(24.0).strong());
            ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                ui.label("Introduceti orasul");
                ui.text_edit_singleline(&mut self.city_name);

                if ui.button("search").clicked() {
                    println!("in {} nu stim ce vreme e", self.city_name);
                }
            });
        });
    }
}
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0])
            .with_title("Egui Example"),
        ..Default::default()
    };

    eframe::run_native("App", options, Box::new(|_cc| Box::new(App::default())))
}
