use eframe::egui::{Color32, Stroke, Visuals};

pub fn orange_mode_visuals() -> Visuals {
  let mut visuals = Visuals::dark();
  visuals.widgets.inactive.bg_fill = Color32::from_rgb(0xf3, 0xa3, 0x09);
  visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::BLACK);
  visuals.widgets.hovered.bg_fill = Color32::from_rgb(0xff, 0xae, 0x19);
  visuals.widgets.active.bg_fill = Color32::from_rgb(0xb0, 0x70, 0x00);
  visuals.override_text_color = Some(Color32::BLACK);
  visuals.window_fill = Color32::from_rgb(0x2d, 0x2d, 0x2d);
  visuals
}