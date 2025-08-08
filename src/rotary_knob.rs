// src/rotary_knob.rs

// This file contains the implementation for the rotary knob widget.

use eframe::egui::{
  Label, Rect, Response, RichText, Sense, TextStyle, Ui, Vec2, Widget,
};

/// A circular knob that can be dragged to change a value.
pub struct RotaryKnob<'a> {
  value: &'a mut f32,
  min: f32,
  max: f32,
  size: f32,
  label: Option<&'a str>,
  show_value: bool,
}

impl<'a> RotaryKnob<'a> {
  /// Creates a new `RotaryKnob`.
  pub fn new(value: &'a mut f32, min: f32, max: f32) -> Self {
    Self {
      value,
      min,
      max,
      size: 100.0,
      label: None,
      show_value: true,
    }
  }

  /// Sets the label for the knob.
  pub fn with_label(mut self, label: &'a str) -> Self {
    self.label = Some(label);
    self
  }

  /// Sets the size (diameter) of the knob.
  pub fn with_size(mut self, size: f32) -> Self {
    self.size = size;
    self
  }

  /// Sets whether to show the numeric value inside the knob.
  pub fn show_value(mut self, show: bool) -> Self {
    self.show_value = show;
    self
  }
}

impl<'a> Widget for RotaryKnob<'a> {
  fn ui(self, ui: &mut Ui) -> Response {
    let RotaryKnob {
      value,
      min,
      max,
      size,
      label,
      show_value,
    } = self;

    let desired_size = Vec2::splat(size);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::drag());
    let center = rect.center();
    let radius = size * 0.5;

    // Handle circular drag input
    if response.dragged() {
      if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
        let delta = pointer_pos - center;
        // Calculate angle and map it to the value range
        let angle = delta.y.atan2(delta.x);
        let t = (angle / std::f32::consts::TAU) + 0.5;
        *value = (min + t * (max - min)).clamp(min, max);
        response.mark_changed();
      }
    }

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
    if show_value {
      let val_str = format!("{:.2}", *value);
      let font = TextStyle::Small.resolve(ui.style());
      painter.text(
        center,
        eframe::egui::Align2::CENTER_CENTER,
        val_str,
        font,
        visuals.text_color(),
      );
    }

    // Draw label below knob
    if let Some(label) = label {
      let label_pos = center + Vec2::Y * (radius + 5.0);
      let label_rect = Rect::from_center_size(label_pos, Vec2::new(size, 10.0));
      ui.put(
        label_rect,
        Label::new(RichText::new(label).text_style(TextStyle::Body)).wrap(false),
      );
    }

    response
  }
}
