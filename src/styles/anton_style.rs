use crate::gui_style::GuiStyle;
use eframe::epaint::{ColorImage, TextureId};
use egui::{style::Visuals, *};
use parking_lot::Mutex;
use std::sync::Arc;

const IMAGE_WIDTH: f32 = 1024.0;
const IMAGE_HEIGHT: f32 = 682.0;

// UV coordinates for the knob from antonui.jpg
fn knob_uv() -> Rect {
  Rect::from_min_max(
    pos2(50.0 / IMAGE_WIDTH, 100.0 / IMAGE_HEIGHT),
    pos2(130.0 / IMAGE_WIDTH, 180.0 / IMAGE_HEIGHT),
  )
}

// UV coordinates for the button from antonui.jpg
fn button_uv() -> Rect {
  Rect::from_min_max(
    pos2(45.0 / IMAGE_WIDTH, 200.0 / IMAGE_HEIGHT),
    pos2(155.0 / IMAGE_WIDTH, 240.0 / IMAGE_HEIGHT),
  )
}

#[derive(Clone)]
struct AntonTextures {
  texture_id: TextureId,
}

/// A style that uses an image for UI elements.
pub struct AntonStyle {
  // Cache for loaded textures to avoid reloading every frame.
  textures: Arc<Mutex<Option<AntonTextures>>>,
  style: Style,
}

impl Default for AntonStyle {
  fn default() -> Self {
    Self {
      textures: Arc::new(Mutex::new(None)),
      style: {
        let mut style = Style::default();
        // Use dark visuals as a base.
        style.visuals = Visuals::dark();
        style
      },
    }
  }
}

impl AntonStyle {
  /// Lazily loads the UI texture atlas.
  fn get_textures(&self, ctx: &Context) -> AntonTextures {
    let mut textures_lock = self.textures.lock();
    if let Some(textures) = &*textures_lock {
      return textures.clone();
    }

    // Load texture for the first time.
    // This assumes `antonui.jpg` is in the root of your project folder.
    let image_bytes = include_bytes!("../../antonui.jpg");
    let image = image::load_from_memory_with_format(image_bytes, image::ImageFormat::Jpeg)
      .expect("Failed to load antonui.jpg. Make sure it's in the project root.");
    let image_rgba = image.to_rgba8();
    let size = [image_rgba.width() as _, image_rgba.height() as _];
    let color_image = ColorImage::from_rgba_unmultiplied(size, image_rgba.as_raw());

    let texture_id = ctx.load_texture("antonui_atlas", color_image, Default::default());

    let new_textures = AntonTextures { texture_id };
    *textures_lock = Some(new_textures.clone());
    new_textures
  }
}

impl GuiStyle for AntonStyle {
  fn name(&self) -> &'static str {
    "Anton"
  }

  fn style(&self) -> Style {
    self.style.clone()
  }

  fn is_image_based(&self) -> bool {
    true
  }

  fn knob_texture<'a>(&self, ctx: &'a Context) -> Option<(TextureId, Rect)> {
    let textures = self.get_textures(ctx);
    Some((textures.texture_id, knob_uv()))
  }

  fn button_texture<'a>(&self, ctx: &'a Context) -> Option<(TextureId, Rect)> {
    let textures = self.get_textures(ctx);
    Some((textures.texture_id, button_uv()))
  }

  fn panel_background_texture<'a>(&self, ctx: &'a Context) -> Option<TextureId> {
    let textures = self.get_textures(ctx);
    Some(textures.texture_id)
  }

  // Define fallback colors and styles for text and other elements.
  fn button_text_color(&self) -> Color32 {
    Color32::WHITE
  }
  fn button_text_color_disabled(&self) -> Color32 {
    Color32::GRAY
  }
  fn button_text_color_hovered(&self) -> Color32 {
    Color32::WHITE
  }
  fn button_text_color_clicked(&self) -> Color32 {
    Color32::WHITE
  }
  fn button_frame_color(&self) -> Color32 {
    Color32::TRANSPARENT
  }
  fn button_frame_color_disabled(&self) -> Color32 {
    Color32::TRANSPARENT
  }
  fn button_frame_color_hovered(&self) -> Color32 {
    Color32::TRANSPARENT
  }
  fn button_frame_color_clicked(&self) -> Color32 {
    Color32::TRANSPARENT
  }
  fn button_rounding(&self) -> f32 {
    12.0
  }
  fn knob_color(&self) -> Color32 {
    Color32::TRANSPARENT
  }
  fn knob_slot_color(&self) -> Color32 {
    Color32::TRANSPARENT
  }
  fn knob_slot_color_disabled(&self) -> Color32 {
    Color32::TRANSPARENT
  }
  fn text_color(&self) -> Color32 {
    Color32::WHITE
  }
  fn text_color_disabled(&self) -> Color32 {
    Color32::GRAY
  }
  fn text_color_strong(&self) -> Color32 {
    Color32::WHITE
  }
  fn text_color_weak(&self) -> Color32 {
    Color32::LIGHT_GRAY
  }
  fn background_color(&self) -> Color32 {
    Color32::from_rgb(27, 27, 27)
  }
  fn border_color(&self) -> Color32 {
    Color32::TRANSPARENT
  }
  fn border_stroke(&self) -> Stroke {
    Stroke::NONE
  }
  fn panel_background_color(&self) -> Color32 {
    Color32::TRANSPARENT
  }
  fn panel_border_color(&self) -> Color32 {
    Color32::TRANSPARENT
  }
  fn panel_border_stroke(&self) -> Stroke {
    Stroke::NONE
  }
  fn panel_rounding(&self) -> f32 {
    0.0
  }
  fn grid_spacing(&self) -> Vec2 {
    Vec2::new(16.0, 16.0)
  }
}
