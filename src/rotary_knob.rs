use crate::gui_style::GuiStyle;
use egui::{vec2, Align2, Color32, Pos2, Rect, Response, Sense, Shape, Ui, Vec2, Widget};
use std::f32::consts::TAU;

const ANGLE_OFFSET: f32 = -TAU / 4.0;

pub struct RotaryKnob<'a> {
  value: &'a mut f32,
  diameter: f32,
  min: f32,
  max: f32,
  style: &'a dyn GuiStyle,
}

impl<'a> RotaryKnob<'a> {
  pub fn new(
    value: &'a mut f32,
    diameter: f32,
    min: f32,
    max: f32,
    style: &'a dyn GuiStyle,
  ) -> Self {
    Self {
      value,
      diameter,
      min,
      max,
      style,
    }
  }
}

impl<'a> Widget for RotaryKnob<'a> {
  fn ui(self, ui: &mut Ui) -> Response {
    let (rect, mut response) =
      ui.allocate_exact_size(Vec2::splat(self.diameter), Sense::click_and_drag());

    if response.dragged() {
      let delta = response.drag_delta().y;
      let range = self.max - self.min;
      let change = delta / self.diameter * range;
      *self.value = (*self.value - change).clamp(self.min, self.max);
      response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
      rotary_knob_ui(
        ui,
        rect,
        self.diameter,
        self.min,
        self.max,
        *self.value,
        self.style,
      );
    }

    response
  }
}

fn rotary_knob_ui(
  ui: &mut Ui,
  rect: Rect,
  diameter: f32,
  min: f32,
  max: f32,
  value: f32,
  style: &dyn GuiStyle,
) {
  let angle_min = -5.0 * TAU / 8.0 + ANGLE_OFFSET;
  let angle_max = -1.0 * TAU / 8.0 + ANGLE_OFFSET;

  if let Some((texture_id, uv)) = style.knob_texture(ui.ctx()) {
    // Draw the knob using the provided image texture.
    let image = egui::Image::new(texture_id, Vec2::splat(diameter)).uv(uv);
    image.paint_at(ui, rect);

    // Draw an indicator on top of the image.
    let angle = (value - min) / (max - min) * (angle_max - angle_min) + angle_min;
    let indicator_radius = diameter * 0.05;
    let indicator_center =
      rect.center() + Vec2::from_angle(angle) * (diameter * 0.5 - indicator_radius * 2.5);
    ui.painter()
      .circle_filled(indicator_center, indicator_radius, Color32::RED);
  } else {
    // Original drawing code for non-image-based styles.
    let center = rect.center();
    let radius = diameter / 2.0;
    let n_points = 32;

    let slot_color = if ui.is_enabled() {
      style.knob_slot_color()
    } else {
      style.knob_slot_color_disabled()
    };

    let slot_points: Vec<Pos2> = (0..=n_points)
      .map(|i| {
        let angle = egui::remap(i as f32, 0.0..=n_points as f32, angle_min..=angle_max);
        center + vec2(angle.cos(), angle.sin()) * radius
      })
      .collect();
    ui.painter()
      .add(Shape::line(slot_points, (3.0, slot_color)));

    let value_angle = egui::remap(value, min..=max, angle_min..=angle_max);
    let value_points: Vec<Pos2> = (0..=n_points)
      .filter_map(|i| {
        let angle = egui::remap(i as f32, 0.0..=n_points as f32, angle_min..=value_angle);
        if angle <= value_angle {
          Some(center + vec2(angle.cos(), angle.sin()) * radius)
        } else {
          None
        }
      })
      .collect();

    if value_points.len() > 1 {
      ui.painter()
        .add(Shape::line(value_points, (3.0, style.knob_color())));
    }

    let indicator_center = center + vec2(value_angle.cos(), value_angle.sin()) * radius * 0.8;
    ui.painter()
      .circle_filled(indicator_center, 3.0, style.knob_color());
  }

  let text_pos = rect.center() - vec2(0.0, diameter * 0.6);
  ui.painter().text(
    text_pos,
    Align2::CENTER_CENTER,
    format!("{:.1}", value),
    egui::FontId::proportional(14.0),
    style.text_color(),
  );
}
