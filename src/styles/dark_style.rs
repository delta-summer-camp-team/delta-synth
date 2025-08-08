use eframe::egui::{Color32, Stroke, Visuals};

pub fn dark_mode_visuals() -> Visuals {
  let mut visuals = Visuals::dark();
  visuals.widgets.inactive.bg_fill = Color32::from_gray(40);
  visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_gray(180));
  visuals.widgets.hovered.bg_fill = Color32::from_gray(60);
  visuals.widgets.active.bg_fill = Color32::from_gray(80);
  visuals.override_text_color = Some(Color32::from_gray(220));
  visuals.window_fill = Color32::from_gray(28);
  visuals
}