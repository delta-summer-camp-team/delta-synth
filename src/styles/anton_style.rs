// src/styles/anton_style.rs
use eframe::egui::{Color32, Stroke, Visuals};

pub fn anton_mode_visuals() -> Visuals {
  let mut visuals = Visuals::dark();
  visuals.widgets.inactive.bg_fill = Color32::from_rgba_unmultiplied(0, 0, 0, 0);
  visuals.widgets.inactive.fg_stroke = Stroke::new(2.0, Color32::from_rgb(0xff, 0xff, 0xff));
  visuals.widgets.hovered.bg_fill = Color32::from_rgba_unmultiplied(0, 0, 0, 0);
  visuals.widgets.active.bg_fill = Color32::from_rgba_unmultiplied(0, 0, 0, 0);
  visuals.override_text_color = Some(Color32::from_rgb(0xff, 0xff, 0xff));
  visuals.window_fill = Color32::from_rgb(0x2d, 0x2d, 0x2d);
  visuals
}