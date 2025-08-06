mod rotary_knob;
use eframe::egui;
use rotary_knob::RotaryKnob;

#[derive(Default)]
struct MyApp {
  knob1: f32,
  knob2: f32,
  slider_vals: [f32; 4],
}

fn main() -> Result<(), eframe::Error> {
  let options = eframe::NativeOptions::default();
  eframe::run_native(
    "Rust Synthesator",
    options,
    Box::new(|_cc| Box::new(MyApp::default())),
  )
}

impl eframe::App for MyApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
      let panel_rect = ui.max_rect();

      // Top knobs and title
      let top_height = 100.0;
      ui.allocate_ui_at_rect(
        egui::Rect::from_min_size(panel_rect.min, egui::vec2(panel_rect.width(), top_height)),
        |ui| {
          ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.add(RotaryKnob::new(&mut self.knob1, 0.0, 1.0).with_label("Knob 1"));

            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
              ui.heading("RUST SYNTHESATOR");
            });

            ui.add(RotaryKnob::new(&mut self.knob2, 0.0, 1.0).with_label("Knob 2"));
            ui.add_space(10.0);
          });
        },
      );

      // Bottom section: buttons & sliders
      let bottom_height = 200.0;
      let bottom_rect = egui::Rect::from_min_size(
        panel_rect.left_bottom() - egui::vec2(0.0, bottom_height),
        egui::vec2(panel_rect.width(), bottom_height),
      );

      ui.allocate_ui_at_rect(bottom_rect, |ui| {
        ui.horizontal(|ui| {
          // Left buttons
          ui.vertical(|ui| {
            if ui.button("Button 1").clicked() {
              println!("Button 1 clicked");
            }
            if ui.button("Button 2").clicked() {
              println!("Button 2 clicked");
            }
          });

          ui.add_space(20.0);

          // Center sliders
          ui.horizontal_centered(|ui| {
            for val in &mut self.slider_vals {
              ui.add(
                egui::Slider::new(val, 0.0..=1.0)
                  .vertical()
                  .text(""),
              );
              ui.add_space(10.0);
            }
          });

          ui.add_space(20.0);

          // Right buttons
          ui.vertical(|ui| {
            if ui.button("Button 3").clicked() {
              println!("Button 3 clicked");
            }
            if ui.button("Button 4").clicked() {
              println!("Button 4 clicked");
            }
          });
        });
      });
    });
  }
}
