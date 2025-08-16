use eframe::egui::{Color32, Stroke, Visuals};

pub fn turquoise_mode_visuals() -> Visuals {
  let mut visuals = Visuals::light();
  visuals.widgets.inactive.bg_fill = Color32::from_rgb(0x40, 0xe0, 0xd0); // Turquoise
  visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::BLACK);
  visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x48, 0xd1, 0xcc); // MediumTurquoise
  visuals.widgets.active.bg_fill = Color32::from_rgb(0x00, 0xce, 0xd1); // DarkTurquoise
  visuals.override_text_color = Some(Color32::BLACK);
  visuals
}