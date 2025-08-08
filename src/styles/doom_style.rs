// src/styles/doom_style.rs
use eframe::egui::{Color32, Stroke, Visuals};

pub fn doom_mode_visuals() -> Visuals {
  let mut visuals = Visuals::dark();
  visuals.widgets.inactive.bg_fill = Color32::from_rgb(0x8b, 0x00, 0x00); // DarkRed
  visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(0xff, 0xff, 0xff)); // White
  visuals.widgets.hovered.bg_fill = Color32::from_rgb(0xb2, 0x22, 0x22); // Firebrick
  visuals.widgets.active.bg_fill = Color32::from_rgb(0xdc, 0x14, 0x3c); // Crimson
  visuals.override_text_color = Some(Color32::from_rgb(0xff, 0xff, 0xff)); // White
  visuals.window_fill = Color32::from_rgb(0x1a, 0x1a, 0x1a); // Very dark gray
  visuals
}