use eframe::egui::{Color32, Stroke, Visuals};

pub fn diagnostics_mode_visuals() -> Visuals {
  let mut visuals = Visuals::dark();
  visuals.widgets.inactive.bg_fill = Color32::from_gray(50);
  visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_gray(200));
  visuals.widgets.hovered.bg_fill = Color32::from_gray(70);
  visuals.widgets.active.bg_fill = Color32::from_gray(90);
  visuals.override_text_color = Some(Color32::from_gray(240));
  visuals.window_fill = Color32::from_gray(30);
  visuals
}