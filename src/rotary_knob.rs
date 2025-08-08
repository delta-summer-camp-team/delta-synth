// src/rotary_knob.rs

// This file contains the implementation for the rotary knob widget.

use eframe::egui::{
  Response, Sense, Ui, Vec2, Widget,
};

/// A circular knob that can be dragged to change a value.
pub struct RotaryKnob<'a> {
  value: &'a mut f32,
  min: f32,
  max: f32,
  size: f32,
}

  /// Sets whether to show the numeric value inside the knob.
impl<'a> Widget for RotaryKnob<'a> {
  fn ui(self, ui: &mut Ui) -> Response {
    let RotaryKnob {
      value,
      min,
      max,
      size,

    } = self;

    let desired_size = Vec2::splat(size);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::drag());
    let center = rect.center();
    let radius = size * 0.5;

    // Handle circular drag input
    let painter = ui.painter();
    let visuals = ui.style().interact(&response);

    // Draw knob circle
    painter.circle(center, radius - 2.0, visuals.bg_fill, visuals.fg_stroke);

    // Draw pointer
    let normalized_value = (*value - min) / (max - min);
    let angle = (normalized_value * std::f32::consts::TAU) - std::f32::consts::PI;
    let pointer = Vec2::angled(angle) * radius * 0.7;
    painter.line_segment([center, center + pointer], visuals.fg_stroke);

    // Draw value text inside the knob
    // Draw label below knob

    response
  }
}
