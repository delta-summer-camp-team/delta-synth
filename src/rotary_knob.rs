use eframe::egui::{
  Label, RichText, Response, Sense, Stroke, TextStyle, Ui, Vec2, Widget, Rect,
};

pub struct RotaryKnob<'a> {
  value: &'a mut f32,
  min: f32,
  max: f32,
  size: f32,
  label: Option<&'a str>,
}

impl<'a> RotaryKnob<'a> {
  pub fn new(value: &'a mut f32, min: f32, max: f32) -> Self {
    Self {
      value,
      min,
      max,
      size: 40.0,
      label: None,
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
}

impl<'a> Widget for RotaryKnob<'a> {
  fn ui(self, ui: &mut Ui) -> Response {
    let RotaryKnob {
      value,
      min,
      max,
      size,
      label,
    } = self;

    let desired_size = Vec2::splat(size);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::drag());

    if response.dragged() {
      let delta = response.drag_delta().y - response.drag_delta().x;
      let sensitivity = (max - min) / 100.0;
      *value = (*value - delta * sensitivity).clamp(min, max);
      response.mark_changed();
    }

    let painter = ui.painter();
    let center = rect.center();
    let radius = size * 0.5;

    painter.circle_filled(center, radius - 2.0, ui.visuals().widgets.inactive.bg_fill);

    let angle = ((*value - min) / (max - min)) * std::f32::consts::TAU * 0.75
      - std::f32::consts::PI * 0.625;
    let pointer = Vec2::angled(angle) * radius * 0.6;
    painter.line_segment(
      [center, center + pointer],
      Stroke::new(2.0, ui.visuals().widgets.inactive.fg_stroke.color),
    );

    if let Some(label) = label {
      let label_pos = center + Vec2::Y * (size * 0.65);
      let label_rect = Rect::from_center_size(label_pos, Vec2::new(size, 14.0));
      ui.put(
        label_rect,
        Label::new(RichText::new(label).text_style(TextStyle::Small)).wrap(false),
      );
    }

    response
  }
}
