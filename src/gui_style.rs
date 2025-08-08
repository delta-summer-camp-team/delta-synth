use egui::{Color32, Context, Rect, Stroke, Style, TextureId, Vec2};

pub trait GuiStyle: Send + Sync {
  fn name(&self) -> &'static str;
  fn style(&self) -> Style;
  fn button_text_color(&self) -> Color32;
  fn button_text_color_disabled(&self) -> Color32;
  fn button_text_color_hovered(&self) -> Color32;
  fn button_text_color_clicked(&self) -> Color32;
  fn button_frame_color(&self) -> Color32;
  fn button_frame_color_disabled(&self) -> Color32;
  fn button_frame_color_hovered(&self) -> Color32;
  fn button_frame_color_clicked(&self) -> Color32;
  fn button_rounding(&self) -> f32;
  fn knob_color(&self) -> Color32;
  fn knob_slot_color(&self) -> Color32;
  fn knob_slot_color_disabled(&self) -> Color32;
  fn text_color(&self) -> Color32;
  fn text_color_disabled(&self) -> Color32;
  fn text_color_strong(&self) -> Color32;
  fn text_color_weak(&self) -> Color32;
  fn background_color(&self) -> Color32;
  fn border_color(&self) -> Color32;
  fn border_stroke(&self) -> Stroke;
  fn panel_background_color(&self) -> Color32;
  fn panel_border_color(&self) -> Color32;
  fn panel_border_stroke(&self) -> Stroke;
  fn panel_rounding(&self) -> f32;
  fn grid_spacing(&self) -> Vec2;

  // New methods for image-based styles

  /// Returns true if the style uses images for widgets.
  fn is_image_based(&self) -> bool {
    false
  }

  /// Provides a texture and UV coordinates for the knob widget.
  fn knob_texture<'a>(&self, _ctx: &'a Context) -> Option<(TextureId, Rect)> {
    None
  }

  /// Provides a texture and UV coordinates for button widgets.
  fn button_texture<'a>(&self, _ctx: &'a Context) -> Option<(TextureId, Rect)> {
    None
  }

  /// Provides a texture for the main panel background.
  fn panel_background_texture<'a>(&self, _ctx: &'a Context) -> Option<TextureId> {
    None
  }
}
