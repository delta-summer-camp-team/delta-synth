// src/gui_style.rs
use eframe::egui::{Color32, Stroke, Visuals};


use crate::styles::{dark_style, diagnostics_style, orange_style, turquoise_style, anton_style, doom_style};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GUIStyle {
  OrangeMode,
  DarkMode,
  TurquoiseMode,
  Diagnostics,
  AntonMode,
  DoomMode,
}

impl GUIStyle {
  pub fn get_visuals(&self) -> Visuals {
    match self {
      GUIStyle::OrangeMode => orange_style::orange_mode_visuals(),
      GUIStyle::DarkMode => dark_style::dark_mode_visuals(),
      GUIStyle::TurquoiseMode => turquoise_style::turquoise_mode_visuals(),
      GUIStyle::Diagnostics => diagnostics_style::diagnostics_mode_visuals(),
      GUIStyle::AntonMode => anton_style::anton_mode_visuals(),
      GUIStyle::DoomMode => doom_style::doom_mode_visuals(),
    }
  }
}