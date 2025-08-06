mod rotary_knob;

use eframe::egui;
use rotary_knob::RotaryKnob;

fn main() -> Result<(), eframe::Error> {
  let options = eframe::NativeOptions::default();
  eframe::run_native(
    "Rotary Knob App",
    options,
    Box::new(|_cc| Box::new(MyApp::default())),
  )
}

#[derive(Default)]
struct MyApp {
  knob1: f32,
  knob2: f32,
}

impl eframe::App for MyApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.heading("Rotary Knobs and Buttons");

      ui.horizontal(|ui| {
        ui.add(RotaryKnob::new(&mut self.knob1, 0.0, 1.0).with_label("Knob 1"));
        ui.add(RotaryKnob::new(&mut self.knob2, 0.0, 1.0).with_label("Knob 2"));
      });


      ui.separator();

      if ui.button("Button 1").clicked() {
        println!("Button 1 clicked");
      }

      if ui.button("Button 2").clicked() {
        println!("Button 2 clicked");
      }

      ui.separator();
      ui.label(format!("Knob 1 Value: {:.2}", self.knob1));
      ui.label(format!("Knob 2 Value: {:.2}", self.knob2));
    });
  }
}
