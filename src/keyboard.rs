use eframe::egui::{self, Color32, Pos2, Rect};

pub struct Keyboard {
  pressed_keys: Vec<u8>,
}

impl Keyboard {
  pub fn new() -> Self {
    Self {
      pressed_keys: Vec::new(),
    }
  }

  pub fn add_key(&mut self, key: u8) {
    if !self.pressed_keys.contains(&key) {
      self.pressed_keys.push(key);
    }
  }

  pub fn remove_key(&mut self, key: u8) {
    self.pressed_keys.retain(|&k| k != key);
  }

  pub fn ui(&self, ui: &mut egui::Ui) {
    let painter = ui.painter();
    let rect = ui.available_rect_before_wrap();
    let rect = Rect::from_min_size(rect.min, egui::vec2(rect.width(), rect.height() * 0.3));

    let white_key_count = 52;
    let white_key_width = rect.width() / white_key_count as f32;
    let black_key_width = white_key_width * 0.7;
    let white_key_height = rect.height();
    let black_key_height = white_key_height * 0.6;

    let mut white_key_x = rect.left();

    // Draw white keys
    for note in 21..=108 {
      let is_black = match note % 12 {
        1 | 3 | 6 | 8 | 10 => true,
        _ => false,
      };
      if !is_black {
        let key_rect = Rect::from_min_size(
          Pos2::new(white_key_x, rect.top()),
          egui::vec2(white_key_width, white_key_height),
        );
        let color = if self.pressed_keys.contains(&note) {
          Color32::from_rgb(100, 100, 255)
        } else {
          Color32::WHITE
        };
        painter.rect_filled(key_rect, 0.0, color);
        painter.rect_stroke(key_rect, 0.0, (1.0, Color32::BLACK));
        white_key_x += white_key_width;
      }
    }

    white_key_x = rect.left();
    // Draw black keys
    for note in 21..=108 {
      let is_black = match note % 12 {
        1 | 3 | 6 | 8 | 10 => true,
        _ => false,
      };

      if is_black {
        let x_pos = white_key_x - (black_key_width / 2.0);
        let key_rect = Rect::from_min_size(
          Pos2::new(x_pos, rect.top()),
          egui::vec2(black_key_width, black_key_height),
        );
        let color = if self.pressed_keys.contains(&note) {
          Color32::from_rgb(100, 100, 255)
        } else {
          Color32::BLACK
        };
        painter.rect_filled(key_rect, 0.0, color);
      } else {
        white_key_x += white_key_width;
      }
    }
  }
}