#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
// src/doom_mode.rs
// Immersive application based on the original Doom source code style.
// This code has been restructured to be more modular and function like the original C codebase.

use crate::app::{MyApp, AppState};
use eframe::egui::{self, Color32, Key, Pos2, Rect, Stroke, Vec2};
use std::time::{Duration, Instant};
use rand::{Rng, rng};
use std::f32::consts::PI;
use rand::seq::SliceRandom;

// ----------------------------- Constants ---------------------------------
const MAP_WIDTH: usize = 64;
const MAP_HEIGHT: usize = 64;
const TROPHY_TILE: u8 = 2; // Winning tile
const WALL_TILE: u8 = 1;
const EMPTY_TILE: u8 = 0;
const PLAYER_RADIUS: f32 = 0.2;
const ENEMY_RADIUS: f32 = 0.3;

// ----------------------------- Data Types ---------------------------------
// Mimics the original mobj_t (moving object) structure
#[derive(Clone)]
pub struct Enemy {
  pub pos: Vec2,
  pub health: f32,
  pub last_attack: Option<Instant>,
  pub monster_type: MonsterType,
  pub animation_frame: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MonsterType {
  Demon,
  Cacodemon,
  Imp,
}

#[derive(Clone)]
pub struct Trophy {
  pub pos: Vec2,
  pub collected: bool,
  pub glow_intensity: f32,
}

// Removed the Door struct as doors no longer exist in this simplified map.

pub struct DoomState {
  // Player state
  pub health: f32,
  pub armor: f32,
  pub ammo: i32,

  // View state
  pub player_pos: Vec2,
  pub player_dir: Vec2,
  pub camera_plane: Vec2,

  // Game objects
  pub enemies: Vec<Enemy>,
  pub map: Vec<Vec<u8>>,
  pub trophy: Trophy,
  // Removed `doors`, `health_items`, and `armor_items` fields from the struct.

  // Game state
  pub game_over: bool,
  pub game_won: bool,
  pub blood_splatters: Vec<(Vec2, f32)>,
  pub last_shot_time: Option<Instant>,
}

// ----------------------------- Map Generation ----------------------------
// This function creates a large, detailed static map.
fn generate_static_dungeon() -> Vec<Vec<u8>> {
  let mut map = vec![vec![WALL_TILE; MAP_WIDTH]; MAP_HEIGHT];
  let mut rng = rng();

  // Fill with empty space
  for y in 1..MAP_HEIGHT - 1 {
    for x in 1..MAP_WIDTH - 1 {
      map[y][x] = EMPTY_TILE;
    }
  }

  // Generate a labyrinth by placing random walls
  for _ in 0..6000 {
    let x = rng.random_range(1..MAP_WIDTH - 1);
    let y = rng.random_range(1..MAP_HEIGHT - 1);
    map[y][x] = WALL_TILE;
  }

  // Create random, open corridors to make a navigable maze
  for _ in 0..1500 {
    let start_x = rng.random_range(1..MAP_WIDTH - 1);
    let start_y = rng.random_range(1..MAP_HEIGHT - 1);
    let len = rng.random_range(3..10);
    let dir = rng.random_range(0..4);

    for i in 0..len {
      match dir {
        0 => { // right
          if start_x + i < MAP_WIDTH - 1 { map[start_y][start_x + i] = EMPTY_TILE; }
        },
        1 => { // left
          if start_x > i { map[start_y][start_x - i] = EMPTY_TILE; }
        },
        2 => { // down
          if start_y + i < MAP_HEIGHT - 1 { map[start_y + i][start_x] = EMPTY_TILE; }
        },
        3 => { // up
          if start_y > i { map[start_y - i][start_x] = EMPTY_TILE; }
        },
        _ => {}
      }
    }
  }

  // Set fixed start and end tiles
  map[1][1] = EMPTY_TILE;
  map[MAP_HEIGHT - 2][MAP_WIDTH - 2] = TROPHY_TILE;

  map
}

// ----------------------------- Drawing Functions --------------------------
// Mimics the sprite drawing functions from the original codebase
fn draw_enemy(painter: &egui::Painter, pos: Pos2, size: Vec2, enemy: &Enemy) {
  let center = Pos2::new(pos.x, pos.y - size.y * 0.3);
  let frame = enemy.animation_frame.sin().abs();

  // Unified drawing logic for a single enemy type (Demon) with a pixel-art style
  let body_color = Color32::from_rgb(200, 0, 0);
  let head_color = Color32::from_rgb(220, 0, 0);
  let eye_color = Color32::from_rgba_unmultiplied(255, 255, 0, (200.0 + frame * 55.0) as u8);

  // Draw body
  painter.rect_filled(
    Rect::from_center_size(center, size),
    2.0,
    body_color
  );

  // Draw head
  let head_size = Vec2::new(size.x * 0.7, size.y * 0.5);
  let head_rect = Rect::from_center_size(Pos2::new(center.x, center.y - size.y * 0.4), head_size);
  painter.rect_filled(head_rect, 2.0, head_color);

  // Draw horns
  painter.line_segment(
    [Pos2::new(head_rect.min.x, head_rect.min.y), Pos2::new(head_rect.min.x - head_size.x * 0.3, head_rect.min.y - head_size.y * 0.5)],
    Stroke::new(2.0, Color32::from_rgb(150, 150, 150))
  );
  painter.line_segment(
    [Pos2::new(head_rect.max.x, head_rect.min.y), Pos2::new(head_rect.max.x + head_size.x * 0.3, head_rect.min.y - head_size.y * 0.5)],
    Stroke::new(2.0, Color32::from_rgb(150, 150, 150))
  );

  // Draw eyes
  painter.circle_filled(
    Pos2::new(head_rect.center().x - head_size.x * 0.2, head_rect.center().y),
    head_size.x * 0.1,
    eye_color
  );
  painter.circle_filled(
    Pos2::new(head_rect.center().x + head_size.x * 0.2, head_rect.center().y),
    head_size.x * 0.1,
    eye_color
  );
}

// Draws the sky and floor textures
fn draw_background(painter: &egui::Painter, rect: Rect) {
  // Draw gradient sky with a loop
  let sky_rect = Rect::from_min_max(
    rect.min,
    Pos2::new(rect.max.x, rect.center().y)
  );
  let height = sky_rect.height() as usize;

  for i in 0..height {
    let t = i as f32 / height as f32;
    let color = Color32::from_rgb(
      (20.0 + (40.0 * t)) as u8,
      (30.0 + (60.0 * t)) as u8,
      (60.0 + (60.0 * t)) as u8
    );
    painter.rect_filled(
      Rect::from_min_max(
        Pos2::new(sky_rect.min.x, sky_rect.min.y + i as f32),
        Pos2::new(sky_rect.max.x, sky_rect.min.y + (i + 1) as f32)
      ),
      0.0,
      color
    );
  }

  // Draw floor
  let floor_rect = Rect::from_min_max(
    Pos2::new(rect.min.x, rect.center().y),
    rect.max
  );
  painter.rect_filled(
    floor_rect,
    0.0,
    Color32::from_rgb(40, 35, 30)
  );
}

// Raycasting engine for walls and objects
fn draw_walls(painter: &egui::Painter, rect: Rect, state: &DoomState) {
  let width = rect.width() as i32;
  let player_x_floor = state.player_pos.x.floor();
  let player_y_floor = state.player_pos.y.floor();

  for x in 0..width {
    let camera_x = 2.0 * x as f32 / width as f32 - 1.0;
    let ray_dir = state.player_dir + state.camera_plane * camera_x;

    let mut map_pos = Vec2::new(player_x_floor, player_y_floor);
    let delta_dist = Vec2::new(
      if ray_dir.x == 0.0 { f32::INFINITY } else { ray_dir.x.abs().recip() },
      if ray_dir.y == 0.0 { f32::INFINITY } else { ray_dir.y.abs().recip() }
    );

    let mut side_dist = Vec2::new(0.0, 0.0);
    let step = Vec2::new(
      ray_dir.x.signum(),
      ray_dir.y.signum()
    );

    if ray_dir.x < 0.0 {
      side_dist.x = (state.player_pos.x - map_pos.x) * delta_dist.x;
    } else {
      side_dist.x = (map_pos.x + 1.0 - state.player_pos.x) * delta_dist.x;
    }
    if ray_dir.y < 0.0 {
      side_dist.y = (state.player_pos.y - map_pos.y) * delta_dist.y;
    } else {
      side_dist.y = (map_pos.y + 1.0 - state.player_pos.y) * delta_dist.y;
    }

    let mut hit = false;
    let mut side = 0;
    let mut perp_wall_dist = 0.0;
    let mut current_tile_type = WALL_TILE;
    let _ = perp_wall_dist;

    while !hit {
      if side_dist.x < side_dist.y {
        side_dist.x += delta_dist.x;
        map_pos.x += step.x;
        side = 0;
      } else {
        side_dist.y += delta_dist.y;
        map_pos.y += step.y;
        side = 1;
      }

      if let Some(tile) = map_tile_at(&state.map, map_pos.x as isize, map_pos.y as isize) {
        if tile != EMPTY_TILE {
          hit = true;
          current_tile_type = tile;
        }
      } else {
        hit = true;
        current_tile_type = WALL_TILE;
      }
    }

    if current_tile_type != EMPTY_TILE {
      perp_wall_dist = if side == 0 {
        (map_pos.x - state.player_pos.x + (1.0 - step.x) / 2.0) / ray_dir.x
      } else {
        (map_pos.y - state.player_pos.y + (1.0 - step.y) / 2.0) / ray_dir.y
      };

      let line_height = (rect.height() / perp_wall_dist) as i32;
      let draw_start = (-line_height / 2 + (rect.height() / 2.0) as i32).max(0);
      let draw_end = (line_height / 2 + (rect.height() / 2.0) as i32).min(rect.height() as i32);

      let color = match current_tile_type {
        WALL_TILE | TROPHY_TILE => {
          let wall_hit_x = if side == 0 { state.player_pos.y + perp_wall_dist * ray_dir.y } else { state.player_pos.x + perp_wall_dist * ray_dir.x };
          let wall_hit_x = wall_hit_x - wall_hit_x.floor();

          if (wall_hit_x * 8.0).floor() as i32 % 2 == 0 {
            if side == 0 { Color32::from_rgb(160, 160, 160) } else { Color32::from_rgb(140, 140, 140) }
          } else {
            if side == 0 { Color32::from_rgb(180, 180, 180) } else { Color32::from_rgb(160, 160, 160) }
          }
        },
        _ => Color32::TRANSPARENT,
      };

      if color != Color32::TRANSPARENT {
        painter.line_segment(
          [Pos2::new(x as f32, draw_start as f32), Pos2::new(x as f32, draw_end as f32)],
          Stroke::new(1.0, color)
        );
      }
    }
  }
}

// ----------------------------- Game Logic ---------------------------------
// These functions are based on the original Doom game logic modules.

impl DoomState {
  fn update_animations(&mut self, dt: f32) {
    for enemy in &mut self.enemies {
      enemy.animation_frame += dt * 4.0;
    }
    self.trophy.glow_intensity += dt * 2.0;

    self.blood_splatters = self.blood_splatters.iter()
      .filter(|(_, t)| *t < 5.0)
      .map(|(pos, t)| (*pos, t + dt))
      .collect();
  }

  fn fire_weapon(&mut self) -> bool {
    if self.ammo <= 0 { return false; }

    self.ammo -= 1;
    self.last_shot_time = Some(Instant::now());
    let mut hit_any = false;

    if let Some(enemy_to_hit) = self.enemies.iter_mut()
      .filter(|enemy| is_visible(self.player_pos, self.player_dir, enemy.pos, &self.map))
      .min_by(|a, b| {
        vec2_distance(self.player_pos, a.pos)
          .partial_cmp(&vec2_distance(self.player_pos, b.pos))
          .unwrap_or(std::cmp::Ordering::Less)
      }) {

      let damage = 100.0;

      enemy_to_hit.health -= damage;
      enemy_to_hit.last_attack = Some(Instant::now());
      hit_any = true;

      if enemy_to_hit.health <= 0.0 {
        self.blood_splatters.push((enemy_to_hit.pos, 0.0));
      }
    }

    self.enemies.retain(|e| e.health > 0.0);
    hit_any
  }

  fn update_enemies(&mut self, dt: f32) {
    for enemy in &mut self.enemies {
      let to_player = self.player_pos - enemy.pos;
      let dist = to_player.length();

      if dist < 8.0 && dist > 1.5 {
        let move_speed = 0.8 * dt;
        let dir = to_player.normalized();
        let new_pos = enemy.pos + dir * move_speed;

        if !check_collision(&self.map, new_pos, ENEMY_RADIUS) {
          enemy.pos = new_pos;
        }
      }

      if dist < 1.5 {
        let now = Instant::now();
        if enemy.last_attack.map_or(true, |t| now.duration_since(t) > Duration::from_secs(1)) {
          let damage = 8.0;

          if self.armor > 0.0 {
            let armor_take = f32::min(damage * 0.5, self.armor);
            self.armor -= armor_take;
            self.health -= damage - armor_take;
          } else {
            self.health -= damage;
          }

          enemy.last_attack = Some(now);
          self.blood_splatters.push((self.player_pos, 0.0));
        }
      }
    }

    if self.health <= 0.0 {
      self.game_over = true;
    }
  }

  fn collect_items(&mut self) {
    let px = self.player_pos.x.floor() as isize;
    let py = self.player_pos.y.floor() as isize;

    if let Some(tile) = map_tile_at(&self.map, px, py) {
      match tile {
        TROPHY_TILE => {
          if !self.trophy.collected {
            self.trophy.collected = true;
            self.game_won = true;
          }
        },
        _ => {}
      }
    }
  }
}

// ----------------------------- Helper Functions --------------------------
// These functions are analogous to the utility functions in Doom's original source.
fn check_collision(map: &Vec<Vec<u8>>, pos: Vec2, radius: f32) -> bool {
  let (x, y) = (pos.x, pos.y);
  let (min_x, max_x) = ((x - radius).floor() as isize, (x + radius).floor() as isize);
  let (min_y, max_y) = ((y - radius).floor() as isize, (y + radius).floor() as isize);

  for iy in min_y..=max_y {
    for ix in min_x..=max_x {
      if let Some(tile) = map_tile_at(map, ix, iy) {
        if tile == WALL_TILE || tile == TROPHY_TILE { // Collide with trophy tile
          return true;
        }
      }
    }
  }
  false
}


fn map_tile_at(map: &Vec<Vec<u8>>, x: isize, y: isize) -> Option<u8> {
  if y < 0 || x < 0 { return None; }
  let yu = y as usize;
  let xu = x as usize;
  if yu >= map.len() || xu >= map[0].len() { return None; }
  Some(map[yu][xu])
}

fn set_map_tile(map: &mut Vec<Vec<u8>>, x: isize, y: isize, val: u8) {
  if y < 0 || x < 0 { return; }
  let yu = y as usize;
  let xu = x as usize;
  if yu >= map.len() || xu >= map[0].len() { return; }
  map[yu][xu] = val;
}

fn vec2_distance(a: Vec2, b: Vec2) -> f32 {
  (a - b).length()
}

fn angle_between(v1: Vec2, v2: Vec2) -> f32 {
  let dot = v1.dot(v2);
  let magnitude_product = v1.length() * v2.length();
  if magnitude_product == 0.0 {
    return 0.0;
  }
  (dot / magnitude_product).acos()
}


fn is_visible(player_pos: Vec2, player_dir: Vec2, target_pos: Vec2, map: &Vec<Vec<u8>>) -> bool {
  let to_target = target_pos - player_pos;
  let dist = to_target.length();
  let ray_dir = to_target.normalized();
  let angle = angle_between(player_dir, ray_dir).abs();

  if angle > PI / 2.0 {
    return false;
  }

  let steps = (dist * 10.0).max(1.0) as i32;

  for i in 0..steps {
    let t = i as f32 / steps as f32;
    let check_pos = player_pos + to_target * t;

    if let Some(tile) = map_tile_at(map, check_pos.x.floor() as isize, check_pos.y.floor() as isize) {
      if tile == WALL_TILE || tile == TROPHY_TILE {
        return false;
      }
    }
  }
  true
}

fn rotate_vec(v: Vec2, angle: f32) -> Vec2 {
  let (sin_a, cos_a) = angle.sin_cos();
  Vec2::new(v.x * cos_a - v.y * sin_a, v.x * sin_a + v.y * cos_a)
}

fn world_to_screen(player_pos: Vec2, rect: Rect, world_pos: Vec2, player_dir: Vec2, camera_plane: Vec2) -> Option<(Pos2, f32)> {
  let to_world_pos = world_pos - player_pos;

  let inv_det = 1.0 / (camera_plane.x * player_dir.y - player_dir.x * camera_plane.y);

  let transform_x = inv_det * (player_dir.y * to_world_pos.x - player_dir.x * to_world_pos.y);
  let transform_y = inv_det * (-camera_plane.y * to_world_pos.x + camera_plane.x * to_world_pos.y);

  if transform_y <= 0.1 {
    return None;
  }

  let screen_x = rect.center().x + (rect.width() * 0.5) * transform_x / transform_y;
  let screen_y = rect.center().y;

  let size_scale = rect.height() / transform_y;

  Some((Pos2::new(screen_x, screen_y), size_scale))
}

// ----------------------------- Main Rendering ----------------------------

pub fn draw_doom_screen(app: &mut MyApp, ctx: &egui::Context) {
  if app.doom_state.game_over {
    draw_game_over_screen(app, ctx);
    return;
  }
  if app.doom_state.game_won {
    draw_victory_screen(app, ctx);
    return;
  }

  egui::Area::new("doom_top_left_corner".into())
    .anchor(egui::Align2::LEFT_TOP, [10.0, 10.0])
    .show(ctx, |ui| {
      if ui.button("Return to Main Menu").clicked() {
        app.app_state = AppState::StartScreen;
        app.doom_state = DoomState::default();
      }
    });


  egui::CentralPanel::default().show(ctx, |ui| {
    let dt = ui.input(|i| i.unstable_dt);
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter();

    let player_dir = app.doom_state.player_dir;
    let camera_plane = app.doom_state.camera_plane;

    let move_speed = 0.05;
    let rot_speed = 0.03;
    ui.input(|i| {
      if i.key_down(Key::W) {
        try_move_player(&mut app.doom_state, player_dir * move_speed);
      }
      if i.key_down(Key::S) {
        try_move_player(&mut app.doom_state, -player_dir * move_speed);
      }
      if i.key_down(Key::A) {
        app.doom_state.player_dir = rotate_vec(player_dir, rot_speed);
        app.doom_state.camera_plane = rotate_vec(camera_plane, rot_speed);
      }
      if i.key_down(Key::D) {
        app.doom_state.player_dir = rotate_vec(player_dir, -rot_speed);
        app.doom_state.camera_plane = rotate_vec(camera_plane, -rot_speed);
      }
      if i.key_pressed(Key::Space) {
        app.doom_state.fire_weapon();
      }
    });

    app.doom_state.update_animations(dt);
    app.doom_state.update_enemies(dt);
    app.doom_state.collect_items();

    let player_pos = app.doom_state.player_pos;
    let player_dir = app.doom_state.player_dir;
    let camera_plane = app.doom_state.camera_plane;

    draw_background(painter, rect);
    draw_walls(painter, rect, &app.doom_state);

    let mut sprites_to_draw = Vec::new();

    for enemy in &app.doom_state.enemies {
      let dist = vec2_distance(player_pos, enemy.pos);
      if is_visible(player_pos, player_dir, enemy.pos, &app.doom_state.map) {
        sprites_to_draw.push(("enemy", dist, enemy.pos));
      }
    }
    if !app.doom_state.trophy.collected {
      let dist = vec2_distance(player_pos, app.doom_state.trophy.pos);
      if is_visible(player_pos, player_dir, app.doom_state.trophy.pos, &app.doom_state.map) {
        sprites_to_draw.push(("trophy", dist, app.doom_state.trophy.pos));
      }
    }

    sprites_to_draw.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Less));

    for (sprite_type, _dist, pos) in sprites_to_draw {
      if let Some((screen_pos, size_scale)) = world_to_screen(player_pos, rect, pos, player_dir, camera_plane) {
        match sprite_type {
          "enemy" => {
            if let Some(enemy) = app.doom_state.enemies.iter().find(|e| vec2_distance(e.pos, pos) < 0.1) {
              let size = (size_scale * 0.5).min(150.0);
              draw_enemy(painter, screen_pos, Vec2::new(size * 0.6, size), enemy);
            }
          },
          "trophy" => {
            let size = (size_scale * 0.3).min(100.0);
            draw_trophy(painter, screen_pos, size);
          },
          _ => {}
        }
      }
    }

    draw_blood_splatters(painter, rect, &app.doom_state);
    draw_player_weapon(painter, rect, &app.doom_state);
    draw_crosshair(painter, rect);
    draw_minimap(painter, rect, &app.doom_state);
    draw_hud(ui, &app.doom_state);
  });
}

fn try_move_player(state: &mut DoomState, delta: Vec2) {
  let new_pos = state.player_pos + delta;

  if !check_collision(&state.map, new_pos, PLAYER_RADIUS) {
    state.player_pos = new_pos;
  }
}

fn draw_player_weapon(painter: &egui::Painter, rect: Rect, state: &DoomState) {
  let weapon_pos = Pos2::new(rect.center().x, rect.max.y - 60.0);
  let weapon_size = Vec2::new(120.0, 140.0);

  painter.rect_filled(
    Rect::from_center_size(weapon_pos, weapon_size),
    6.0,
    Color32::from_rgb(50, 50, 50)
  );

  if let Some(last_shot) = state.last_shot_time {
    if last_shot.elapsed() < Duration::from_millis(100) {
      painter.circle_filled(
        Pos2::new(weapon_pos.x, weapon_pos.y - weapon_size.y * 0.4),
        weapon_size.y * 0.6,
        Color32::from_rgba_unmultiplied(255, 200, 0, 180)
      );
    }
  }
}

fn draw_trophy(painter: &egui::Painter, pos: Pos2, size_scale: f32) {
  let size = (size_scale * 0.3).min(100.0);
  let center = Pos2::new(pos.x, pos.y);

  painter.circle_filled(center, size, Color32::GOLD);
  painter.rect_filled(
    Rect::from_center_size(
      Pos2::new(center.x, center.y + size * 0.5),
      Vec2::new(size * 1.2, size * 0.3)
    ),
    3.0,
    Color32::from_rgb(150, 150, 150)
  );
}

fn draw_crosshair(painter: &egui::Painter, rect: Rect) {
  let crosshair_pos = rect.center();
  let crosshair_size = 8.0;
  painter.line_segment(
    [Pos2::new(crosshair_pos.x - crosshair_size, crosshair_pos.y),
      Pos2::new(crosshair_pos.x + crosshair_size, crosshair_pos.y)],
    Stroke::new(2.0, Color32::RED));
  painter.line_segment(
    [Pos2::new(crosshair_pos.x, crosshair_pos.y - crosshair_size),
      Pos2::new(crosshair_pos.x, crosshair_pos.y + crosshair_size)],
    Stroke::new(2.0, Color32::RED));
}

fn draw_blood_splatters(painter: &egui::Painter, rect: Rect, state: &DoomState) {
  for (pos, timer) in &state.blood_splatters {
    if let Some((screen_pos, size_scale)) = world_to_screen(state.player_pos, rect, *pos, state.player_dir, state.camera_plane) {
      let alpha = (1.0 - timer / 5.0).max(0.0);
      let size = (size_scale * 0.5).min(50.0);
      painter.circle_filled(
        screen_pos,
        size * (1.0 + timer / 5.0),
        Color32::from_rgba_unmultiplied(150, 0, 0, (alpha * 150.0) as u8)
      );
    }
  }
}


fn draw_minimap(painter: &egui::Painter, rect: Rect, state: &DoomState) {
  let size = 180.0;
  let mm_rect = Rect::from_min_size(
    Pos2::new(rect.min.x + 10.0, rect.min.y + 10.0),
    Vec2::new(size, size)
  );

  painter.rect_filled(mm_rect, 5.0, Color32::from_rgba_unmultiplied(20, 20, 20, 200));

  let cell_size = Vec2::new(
    (size - 10.0) / MAP_WIDTH as f32,
    (size - 10.0) / MAP_HEIGHT as f32
  );

  for y in 0..MAP_HEIGHT {
    for x in 0..MAP_WIDTH {
      let pos = Pos2::new(
        mm_rect.min.x + 5.0 + x as f32 * cell_size.x + cell_size.x * 0.5,
        mm_rect.min.y + 5.0 + y as f32 * cell_size.y + cell_size.y * 0.5
      );

      let color = match state.map[y][x] {
        WALL_TILE => Color32::from_rgb(120, 120, 120),
        TROPHY_TILE => Color32::GOLD,
        _ => continue
      };

      painter.rect_filled(
        Rect::from_center_size(pos, Vec2::new(cell_size.x - 2.0, cell_size.y - 2.0)),
        0.0,
        color
      );
    }
  }

  let player_pos_on_map = Pos2::new(
    mm_rect.min.x + 5.0 + (state.player_pos.x / MAP_WIDTH as f32) * (size - 10.0),
    mm_rect.min.y + 5.0 + (state.player_pos.y / MAP_HEIGHT as f32) * (size - 10.0)
  );
  painter.circle_filled(player_pos_on_map, 4.0, Color32::BLUE);

  for enemy in &state.enemies {
    let enemy_pos_on_map = Pos2::new(
      mm_rect.min.x + 5.0 + (enemy.pos.x / MAP_WIDTH as f32) * (size - 10.0),
      mm_rect.min.y + 5.0 + (enemy.pos.y / MAP_HEIGHT as f32) * (size - 10.0)
    );
    painter.circle_filled(enemy_pos_on_map, 3.0, Color32::RED);
  }
}

fn draw_hud(ui: &mut egui::Ui, state: &DoomState) {
  egui::Area::new("status_panel".into())
    .anchor(egui::Align2::RIGHT_TOP, [-20.0, 20.0])
    .show(ui.ctx(), |ui| {
      ui.vertical(|ui| {
        ui.label(egui::RichText::new(format!("HEALTH: {:.1}", state.health))
          .color(if state.health < 30.0 { Color32::RED } else { Color32::WHITE })
          .heading());
        ui.label(egui::RichText::new(format!("ARMOR: {:.1}", state.armor))
          .color(Color32::from_rgb(100, 150, 255))
          .heading());
        ui.label(egui::RichText::new(format!("AMMO: {}", state.ammo))
          .color(if state.ammo < 5 { Color32::RED } else { Color32::YELLOW })
          .heading());
      });
    });
}

fn draw_game_over_screen(app: &mut MyApp, ctx: &egui::Context) {
  egui::CentralPanel::default().show(ctx, |ui| {
    ui.vertical_centered(|ui| {
      ui.add_space(50.0);
      ui.heading(egui::RichText::new("GAME OVER").color(Color32::RED));
      ui.label("Your journey ends here...");
      ui.add_space(20.0);

      if ui.button("Try Again").clicked() {
        app.doom_state = DoomState::default();
      }
      if ui.button("Return to Main Menu").clicked() {
        app.app_state = AppState::StartScreen;
        app.doom_state = DoomState::default();
      }
    });
  });
}

fn draw_victory_screen(app: &mut MyApp, ctx: &egui::Context) {
  egui::CentralPanel::default().show(ctx, |ui| {
    ui.vertical_centered(|ui| {
      ui.add_space(50.0);
      ui.heading(egui::RichText::new("VICTORY!").color(Color32::GOLD));
      ui.label("You escaped with the sacred artifact!");
      ui.add_space(20.0);

      if ui.button("Play Again").clicked() {
        app.doom_state = DoomState::default();
      }
      if ui.button("Return to Main Menu").clicked() {
        app.app_state = AppState::StartScreen;
        app.doom_state = DoomState::default();
      }
    });
  });
}

impl Default for DoomState {
  fn default() -> Self {
    let mut map = generate_static_dungeon();

    let mut player_pos = Vec2::new(1.5, 1.5);
    let trophy_pos = Vec2::new((MAP_WIDTH - 2) as f32 + 0.5, (MAP_HEIGHT - 2) as f32 + 0.5);

    let mut enemies = Vec::new();
    let mut rng = rng();

    // Spawn 20 enemies randomly, avoiding the start and end tiles
    let mut empty_positions: Vec<(usize, usize)> = (0..MAP_HEIGHT)
      .flat_map(|y| (0..MAP_WIDTH).map(move |x| (x, y)))
      .filter(|(x, y)| map[*y][*x] == EMPTY_TILE && (*x, *y) != (1,1) && (*x, *y) != (MAP_WIDTH-2, MAP_HEIGHT-2))
      .collect();
    empty_positions.shuffle(&mut rng);

    for _ in 0..20 {
      if let Some((x, y)) = empty_positions.pop() {
        let monster_type = MonsterType::Demon;

        enemies.push(Enemy {
          pos: Vec2::new(x as f32 + 0.5, y as f32 + 0.5),
          health: 120.0,
          last_attack: None,
          monster_type,
          animation_frame: 0.0,
        });
      }
    }

    DoomState {
      health: 100.0,
      armor: 0.0,
      ammo: 20,
      player_pos,
      player_dir: Vec2::new(-1.0, 0.0),
      camera_plane: Vec2::new(0.0, 0.66),
      enemies,
      map,
      trophy: Trophy {
        pos: trophy_pos,
        collected: false,
        glow_intensity: 0.0,
      },
      game_over: false,
      game_won: false,
      blood_splatters: Vec::new(),
      last_shot_time: None,
    }
  }
}