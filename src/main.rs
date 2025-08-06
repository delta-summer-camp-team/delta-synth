mod rotary_knob;
use eframe::egui;
use rotary_knob::RotaryKnob;
use egui::{RichText, Color32, Vec2, Button};
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
      let top_height = 200.0;
      ui.allocate_ui_at_rect(
        egui::Rect::from_min_size(panel_rect.min, egui::vec2(panel_rect.width(), top_height)),
        |ui| {
          ui.horizontal(|ui| {
            ui.add_space(100.0);
            ui.vertical(|ui| {
              ui.add_space(250.0);  // push the widget down by 15 pixels
              ui.label("Lower item");
            });
            ui.add(RotaryKnob::new(&mut self.knob1, -1.0, 1.0)
              .with_label("Knob 2")
              .with_size(60.0)
              .show_value(true));

            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
              ui.label(
                RichText::new("RUST SYNTHESATOR")
                  .monospace()
                  .heading()
                  .size(28.0)
                  .color(Color32::from_rgb(255, 204, 0))
              );
            });
            ui.add_space(-175.0);
            ui.add(RotaryKnob::new(&mut self.knob2, -1.0, 1.0)
              .with_label("Knob 2")
              .with_size(60.0)
              .show_value(true));
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
                egui::Slider::new(val, -1.0..=1.0)
                  .vertical()
                  .text(""),
              );
              ui.add_space(20.0);
            }
          });

          ui.add_space(20.0);

          // Right buttons
          ui.vertical(|ui| {
            if ui.add(egui::Button::new("Button 3").min_size(egui::Vec2::new(120.0, 40.0))).clicked() {
              println!("Button 3 clicked");
            }
            if ui
              .add(
                Button::new(
                  RichText::new("BUTTON 4")
                    .heading()
                    .monospace()
                    .color(Color32::BLACK),
                )
                  .min_size(Vec2::new(140.0, 50.0))         // bigger button
                  .fill(Color32::from_rgb(0xf3, 0xa3, 0x09)) // background color: #f3a309
              )
              .clicked()
            {
              println!("Button 4 clicked");
            }
          });
        });
      });
    });
  }
}
