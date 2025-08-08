use crate::gui_style::GuiStyle;
use crate::keyboard::Keyboard;
use crate::rotary_knob::RotaryKnob;
use crate::styles::anton_style::AntonStyle;
use crate::styles::dark_style::DarkStyle;
use crate::styles::diagnostics_style::DiagnosticsStyle;
use crate::styles::orange_style::OrangeStyle;
use crate::styles::turquoise_style::TurquoiseStyle;
use egui::{Align2, Button, ComboBox, FontId, Frame, ImageButton, TextureOptions, Vec2};

pub struct DeltaApp {
  knob1: f32,
  knob2: f32,
  knob3: f32,
  knob4: f32,
  panic: bool,
  keyboard: Keyboard,
  styles: Vec<Box<dyn GuiStyle>>,
  selected_style: usize,
}

impl Default for DeltaApp {
  fn default() -> Self {
    let styles: Vec<Box<dyn GuiStyle>> = vec![
      Box::new(DarkStyle::default()),
      Box::new(OrangeStyle::default()),
      Box::new(TurquoiseStyle::default()),
      Box::new(DiagnosticsStyle::default()),
      Box::new(AntonStyle::default()),
    ];
    Self {
      knob1: 0.0,
      knob2: 0.0,
      knob3: 0.0,
      knob4: 0.0,
      panic: false,
      keyboard: Keyboard::new(),
      styles,
      selected_style: 0,
    }
  }
}

impl DeltaApp {
  pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
    Default::default()
  }

  fn style(&self) -> &dyn GuiStyle {
    self.styles[self.selected_style].as_ref()
  }
}

impl eframe::App for DeltaApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    let style = self.style();

    // Set the background frame. If the style provides a texture, use it.
    let frame = if let Some(texture_id) = style.panel_background_texture(ctx) {
      Frame::central_panel(&style.style()).fill_texture(
        texture_id,
        TextureOptions::default(),
      )
    } else {
      Frame::central_panel(&style.style()).fill(style.background_color())
    };

    egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
      ui.horizontal(|ui| {
        ui.label("Style:");
        ComboBox::from_id_source("style_combo")
          .selected_text(style.name())
          .show_ui(ui, |ui| {
            for (i, s) in self.styles.iter().enumerate() {
              ui.selectable_value(&mut self.selected_style, i, s.name());
            }
          });
      });

      ui.add_space(20.0);

      ui.horizontal(|ui| {
        ui.add(RotaryKnob::new(&mut self.knob1, 100.0, 0.0, 10.0, style));
        ui.add(RotaryKnob::new(&mut self.knob2, 100.0, 0.0, 10.0, style));
        ui.add(RotaryKnob::new(&mut self.knob3, 100.0, 0.0, 10.0, style));
        ui.add(RotaryKnob::new(&mut self.knob4, 100.0, 0.0, 10.0, style));
      });

      ui.add_space(20.0);

      ui.horizontal(|ui| {
        // If the style is image-based, use an ImageButton.
        if let Some((texture_id, uv)) = style.button_texture(ui.ctx()) {
          let button_size = Vec2::new(110.0, 40.0);
          let image_button = ImageButton::new(texture_id, button_size).uv(uv);
          let response = ui.add(image_button);

          // Manually draw text on top of the image button.
          ui.painter().text(
            response.rect.center(),
            Align2::CENTER_CENTER,
            "Panic!",
            FontId::proportional(16.0),
            style.button_text_color(),
          );
          if response.clicked() {
            self.panic = true;
          }
        } else {
          // Otherwise, use the standard egui::Button.
          let button = Button::new("Panic!").text_color(style.button_text_color());
          if ui.add(button).clicked() {
            self.panic = true;
          }
        }

        if ui.add(Button::new("Reset")).clicked() {
          self.knob1 = 0.0;
          self.knob2 = 0.0;
          self.knob3 = 0.0;
          self.knob4 = 0.0;
          self.panic = false;
        }
      });

      if self.panic {
        ui.colored_label(egui::Color32::RED, "PANICKING!");
      }

      ui.add_space(20.0);

      ui.add(&mut self.keyboard);
    });
  }
}
