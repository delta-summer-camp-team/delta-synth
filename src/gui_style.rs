use eframe::egui::Visuals;

use crate::styles::{dark_style, orange_style, turquoise_style};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GUIStyle {
  OrangeMode,
  DarkMode,
  TurquoiseMode,
}

impl GUIStyle {
  pub fn get_visuals(&self) -> Visuals {
    match self {
      GUIStyle::OrangeMode => orange_style::orange_mode_visuals(),
      GUIStyle::DarkMode => dark_style::dark_mode_visuals(),
      GUIStyle::TurquoiseMode => turquoise_style::turquoise_mode_visuals(),
    }
  }
}