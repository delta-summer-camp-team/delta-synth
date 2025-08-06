use eframe::egui::{
  Label, Rect, RichText, Response, Sense, Stroke, TextStyle, Ui, Vec2, Widget,
};

pub struct RotaryKnob<'a> {
  value: &'a mut f32,
  min: f32,
  max: f32,
  size: f32,
  label: Option<&'a str>,
  show_value: bool,
}

impl<'a> RotaryKnob<'a> {
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

  pub fn with_label(mut self, label: &'a str) -> Self {
    self.label = Some(label);
    self
  }

  pub fn with_size(mut self, size: f32) -> Self {
    self.size = size;
    self
  }

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
    let radius = size * 2.0;

    // Handle circular drag input
    if response.dragged() {
      if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
        let delta = pointer_pos - center;
        let mut angle = delta.angle(); // from -π to π

        // Convert to 0..=2π range
        if angle < 0.0 {
          angle += std::f32::consts::TAU;
        }

        // Normalize angle to 0.0..=1.0
        let t = angle / std::f32::consts::TAU;

        // Map to value
        *value = min + t * (max - min);
        response.mark_changed();
      }
    }

    let painter = ui.painter();

    // Draw knob circle
    painter.circle_filled(center, radius - 2.0, ui.visuals().widgets.inactive.bg_fill);

    // Draw pointer
    let normalized_value = (*value - min) / (max - min);
    let angle = normalized_value * std::f32::consts::TAU;
    let pointer = Vec2::angled(angle) * radius * 0.6;

    painter.line_segment(
      [center, center + pointer],
      Stroke::new(2.0, ui.visuals().widgets.inactive.fg_stroke.color),
    );

    // Draw value text inside the knob
    if show_value {
      let val_str = format!("{:.2}", *value);
      let font = TextStyle::Small.resolve(ui.style());
      painter.text(center, eframe::egui::Align2::CENTER_CENTER, val_str, font, ui.visuals().text_color());
    }

    // Draw label below knob
    if let Some(label) = label {
      let label_pos = center + Vec2::Y * (size * 0.65);
      let label_rect = Rect::from_center_size(label_pos, Vec2::new(size, 10.0));
      ui.put(
        label_rect,
        Label::new(RichText::new(label).text_style(TextStyle::Small)).wrap(false),
      );
    }

    response
  }
}
