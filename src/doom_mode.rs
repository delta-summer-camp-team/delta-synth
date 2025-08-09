#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
// src/doom_mode.rs
// Enhanced Doom-mode implementation with:
// - Procedural map generation
// - Visibility checking
// - Monster AI improvements
// - Animated effects

use crate::app::MyApp;
use eframe::egui::{self, Color32, Key, Pos2, Rect, Stroke, Vec2};
use std::time::{Duration, Instant};
use rand::{Rng, thread_rng};
use std::f32::consts::PI;
use rand::seq::SliceRandom;

// ----------------------------- Constants ---------------------------------
const MAP_WIDTH: usize = 24;
const MAP_HEIGHT: usize = 24;
const DOOR_TILE: u8 = 2;
const TROPHY_TILE: u8 = 3;
const WALL_TILE: u8 = 1;
const EMPTY_TILE: u8 = 0;
const PLAYER_RADIUS: f32 = 0.2; // Small radius for player collision checks
const ENEMY_RADIUS: f32 = 0.3; // Small radius for enemy collision checks

// ----------------------------- Data Types ---------------------------------
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

  // Game state
  pub game_over: bool,
  pub game_won: bool,
  pub blood_splatters: Vec<(Vec2, f32)>,
  pub last_spawn_time: Instant,
  pub last_shot_time: Option<Instant>,
}

// ----------------------------- Map Generation ----------------------------
fn generate_dungeon() -> Vec<Vec<u8>> {
  let mut rng = thread_rng();
  let mut map = vec![vec![WALL_TILE; MAP_WIDTH]; MAP_HEIGHT];

  // Random walk algorithm for corridors
  let (mut x, mut y) = (MAP_WIDTH / 2, MAP_HEIGHT / 2);
  for _ in 0..500 {
    map[y][x] = EMPTY_TILE;
    match rng.gen_range(0..4) {
      0 => x = x.saturating_add(1).min(MAP_WIDTH - 1),
      1 => x = x.saturating_sub(1),
      2 => y = y.saturating_add(1).min(MAP_HEIGHT - 1),
      _ => y = y.saturating_sub(1),
    }
  }

  // Cellular automata pass for natural-looking caves
  for _ in 0..4 {
    let mut new_map = map.clone();
    for y in 1..MAP_HEIGHT - 1 {
      for x in 1..MAP_WIDTH - 1 {
        let neighbors = count_wall_neighbors(&map, x, y);
        new_map[y][x] = if neighbors > 4 { WALL_TILE } else { EMPTY_TILE };
      }
    }
    map = new_map;
  }

  // Place special tiles
  place_special_tiles(&mut map, &mut rng);

  map
}

fn count_wall_neighbors(map: &Vec<Vec<u8>>, x: usize, y: usize) -> u8 {
  let mut count = 0;
  for dy in -1..=1 {
    for dx in -1..=1 {
      if dx == 0 && dy == 0 { continue; }
      let nx = (x as isize + dx) as usize;
      let ny = (y as isize + dy) as usize;
      if nx < MAP_WIDTH && ny < MAP_HEIGHT && map[ny][nx] == WALL_TILE {
        count += 1;
      }
    }
  }
  count
}

fn place_special_tiles(map: &mut Vec<Vec<u8>>, rng: &mut impl Rng) {
  let mut placed_door = false;
  let mut placed_trophy = false;

  // Scan map in random order to place special tiles
  let mut positions: Vec<(usize, usize)> = (1..MAP_HEIGHT - 1)
    .flat_map(|y| (1..MAP_WIDTH - 1).map(move |x| (x, y)))
    .collect();

  positions.shuffle(rng);

  for (x, y) in positions {
    if map[y][x] == EMPTY_TILE {
      // Place door if suitable location found
      if !placed_door && rng.gen_bool(0.05) && has_wall_neighbor(map, x, y) {
        map[y][x] = DOOR_TILE;
        placed_door = true;
      }
      // Place trophy if suitable location found
      else if !placed_trophy && rng.gen_bool(0.03) {
        map[y][x] = TROPHY_TILE;
        placed_trophy = true;
      }

      if placed_door && placed_trophy {
        break;
      }
    }
  }

  // Fallback for door placement
  if !placed_door {
    let mut found = false;
    for y in 1..MAP_HEIGHT - 1 {
      for x in 1..MAP_WIDTH - 1 {
        if map[y][x] == EMPTY_TILE {
          map[y][x] = DOOR_TILE;
          found = true;
          break;
        }
      }
      if found { break; }
    }
  }
  // Fallback for trophy placement
  if !placed_trophy {
    let mut found = false;
    for y in (1..MAP_HEIGHT - 1).rev() {
      for x in (1..MAP_WIDTH - 1).rev() {
        if map[y][x] == EMPTY_TILE {
          map[y][x] = TROPHY_TILE;
          found = true;
          break;
        }
      }
      if found { break; }
    }
  }
}

fn has_wall_neighbor(map: &Vec<Vec<u8>>, x: usize, y: usize) -> bool {
  let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
  directions.iter().any(|(dx, dy)| {
    let nx = (x as isize + dx) as usize;
    let ny = (y as isize + dy) as usize;
    nx < MAP_WIDTH && ny < MAP_HEIGHT && map[ny][nx] == WALL_TILE
  })
}

// ----------------------------- Drawing Functions --------------------------

fn draw_enemy(painter: &egui::Painter, pos: Pos2, size: Vec2, enemy: &Enemy) {
  let center = Pos2::new(pos.x, pos.y - size.y * 0.3);
  let frame = enemy.animation_frame.sin().abs();

  match enemy.monster_type {
    MonsterType::Demon => {
      // Draw demon body with pulsing animation
      let pulse_size = size * (1.0 + frame * 0.1);
      painter.rect_filled(
        Rect::from_center_size(center, pulse_size),
        5.0,
        Color32::from_rgb(150 + (frame * 50.0).min(105.0) as u8, 0, 0)
      );

      // Draw demon features
      let head_pos = Pos2::new(center.x, center.y - pulse_size.y * 0.25);
      painter.circle_filled(head_pos, pulse_size.x * 0.3, Color32::from_rgb(200, 0, 0));
      painter.circle_filled(
        Pos2::new(head_pos.x - pulse_size.x * 0.15, head_pos.y),
        pulse_size.x * 0.08,
        Color32::YELLOW
      );
      painter.circle_filled(
        Pos2::new(head_pos.x + pulse_size.x * 0.15, head_pos.y),
        pulse_size.x * 0.08,
        Color32::YELLOW
      );
    },
    MonsterType::Cacodemon => {
      // Draw floating eye monster
      let body_size = size * 0.7;
      painter.circle_filled(center, body_size.x * 0.5, Color32::from_rgb(200, 100, 100));

      // Animated mouth
      let mouth_width = body_size.x * 0.6 * (0.5 + frame * 0.5);
      painter.rect_filled(
        Rect::from_center_size(
          Pos2::new(center.x, center.y + body_size.y * 0.15),
          Vec2::new(mouth_width, body_size.y * 0.1)
        ),
        2.0,
        Color32::from_rgb(50, 0, 0)
      );
    },
    MonsterType::Imp => {
      // Draw agile imp enemy
      let body_size = size * 0.8;
      let body_rect = Rect::from_center_size(center, body_size);
      painter.rect_filled(body_rect, 4.0, Color32::from_rgb(100, 50, 0));

      // Claw attack animation
      let claw_offset = body_size.x * 0.3 * (1.0 + frame * 0.2);
      painter.line_segment(
        [Pos2::new(center.x - claw_offset, center.y),
          Pos2::new(center.x - claw_offset * 1.5, center.y - body_size.y * 0.25)],
        Stroke::new(3.0, Color32::from_rgb(150, 100, 0))
      );
      painter.line_segment(
        [Pos2::new(center.x + claw_offset, center.y),
          Pos2::new(center.x + claw_offset * 1.5, center.y - body_size.y * 0.25)],
        Stroke::new(3.0, Color32::from_rgb(150, 100, 0))
      );
    }
  }
}

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

fn draw_walls(painter: &egui::Painter, rect: Rect, state: &DoomState) {
  let width = rect.width() as i32;

  for x in 0..width {
    let camera_x = 2.0 * x as f32 / width as f32 - 1.0;
    let ray_dir = state.player_dir + state.camera_plane * camera_x;

    let mut map_pos = Vec2::new(
      state.player_pos.x.floor(),
      state.player_pos.y.floor()
    );

    let mut side_dist = Vec2::new(0.0, 0.0);
    let delta_dist = Vec2::new(
      if ray_dir.x == 0.0 { f32::INFINITY } else { ray_dir.x.abs().recip() },
      if ray_dir.y == 0.0 { f32::INFINITY } else { ray_dir.y.abs().recip() }
    );

    let step = Vec2::new(
      ray_dir.x.signum(),
      ray_dir.y.signum()
    );

    // DDA algorithm
    let mut hit = false;
    let mut side = 0; // NS or EW wall
    let mut perp_wall_dist = 0.0;

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

      match map_tile_at(&state.map, map_pos.x as isize, map_pos.y as isize) {
        Some(tile) if tile != EMPTY_TILE => {
          hit = true;
          perp_wall_dist = if side == 0 {
            (map_pos.x - state.player_pos.x + (1.0 - step.x) / 2.0) / ray_dir.x
          } else {
            (map_pos.y - state.player_pos.y + (1.0 - step.y) / 2.0) / ray_dir.y
          };

          let line_height = (rect.height() / perp_wall_dist) as i32;
          let draw_start = (-line_height / 2 + (rect.height() / 2.0) as i32).max(0);
          let draw_end = (line_height / 2 + (rect.height() / 2.0) as i32).min(rect.height() as i32);

          let color = match tile {
            WALL_TILE => if side == 0 {
              Color32::from_rgb(160, 160, 160) // NS wall
            } else {
              Color32::from_rgb(140, 140, 140) // EW wall
            },
            DOOR_TILE => if side == 0 {
              Color32::from_rgb(180, 120, 60)
            } else {
              Color32::from_rgb(150, 100, 50)
            },
            _ => Color32::TRANSPARENT
          };

          if color != Color32::TRANSPARENT {
            painter.line_segment(
              [
                Pos2::new(x as f32, draw_start as f32),
                Pos2::new(x as f32, draw_end as f32)
              ],
              Stroke::new(1.0, color)
            );
          }
        },
        _ => ()
      }
    }
  }
}

// ----------------------------- Game Logic ---------------------------------

impl DoomState {
  fn spawn_enemies(&mut self) {
    let mut rng = thread_rng();
    if Instant::now().duration_since(self.last_spawn_time) > Duration::from_secs(10)
      && self.enemies.len() < 6 {

      // Find valid spawn positions
      let mut candidates = vec![];
      for y in 1..self.map.len()-1 {
        for x in 1..self.map[0].len()-1 {
          if self.map[y][x] == EMPTY_TILE {
            let pos = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            if vec2_distance(pos, self.player_pos) > 5.0 {
              candidates.push((x, y));
            }
          }
        }
      }

      if !candidates.is_empty() {
        let (x, y) = candidates[rng.gen_range(0..candidates.len())];
        let monster_type = match rng.gen_range(0..3) {
          0 => MonsterType::Demon,
          1 => MonsterType::Cacodemon,
          _ => MonsterType::Imp,
        };

        self.enemies.push(Enemy {
          pos: Vec2::new(x as f32 + 0.5, y as f32 + 0.5),
          health: match monster_type {
            MonsterType::Demon => 120.0,
            MonsterType::Cacodemon => 80.0,
            MonsterType::Imp => 60.0
          },
          last_attack: None,
          monster_type,
          animation_frame: 0.0,
        });
        self.last_spawn_time = Instant::now();
      }
    }
  }

  fn update_animations(&mut self, dt: f32) {
    for enemy in &mut self.enemies {
      enemy.animation_frame += dt * 4.0;
    }
    self.trophy.glow_intensity += dt * 2.0;

    // Update blood splatters
    self.blood_splatters = self.blood_splatters.iter()
      .filter(|(_, t)| *t < 5.0)
      .map(|(pos, t)| (*pos, t + dt))
      .collect();
  }

  fn try_open_door(&mut self) -> bool {
    let front_pos = self.player_pos + self.player_dir * (PLAYER_RADIUS + 0.1);
    let dx = front_pos.x.floor() as isize;
    let dy = front_pos.y.floor() as isize;

    if let Some(DOOR_TILE) = map_tile_at(&self.map, dx, dy) {
      set_map_tile(&mut self.map, dx, dy, EMPTY_TILE);
      true
    } else {
      false
    }
  }

  fn fire_weapon(&mut self) -> bool {
    if self.ammo <= 0 { return false; }

    self.ammo -= 1;
    self.last_shot_time = Some(Instant::now());
    let mut hit_any = false;

    // Find the closest visible enemy and apply damage to it.
    if let Some(mut enemy_to_hit) = self.enemies.iter_mut()
      .filter(|enemy| is_visible(self.player_pos, self.player_dir, enemy.pos, &self.map))
      .min_by(|a, b| {
        vec2_distance(self.player_pos, a.pos)
          .partial_cmp(&vec2_distance(self.player_pos, b.pos))
          .unwrap_or(std::cmp::Ordering::Less)
      }) {

      let dist = vec2_distance(enemy_to_hit.pos, self.player_pos);
      if dist <= 4.0 {
        let damage = match enemy_to_hit.monster_type {
          MonsterType::Demon => 25.0,
          MonsterType::Cacodemon => 35.0,
          MonsterType::Imp => 40.0,
        };

        enemy_to_hit.health -= damage;
        enemy_to_hit.last_attack = Some(Instant::now());
        hit_any = true;

        if enemy_to_hit.health <= 0.0 {
          self.blood_splatters.push((enemy_to_hit.pos, 0.0));
        }
      }
    }

    self.enemies.retain(|e| e.health > 0.0);
    hit_any
  }

  fn update_enemies(&mut self, dt: f32) {
    for enemy in &mut self.enemies {
      let to_player = self.player_pos - enemy.pos;
      let dist = to_player.length();

      // Movement
      if dist < 8.0 && dist > 1.5 {
        let move_speed = 0.8 * dt;
        let dir = to_player.normalized();
        let new_pos = enemy.pos + dir * move_speed;

        // Improved collision detection for enemies
        if !check_collision(&self.map, new_pos, ENEMY_RADIUS) {
          enemy.pos = new_pos;
        }
      }

      // Attack
      if dist < 1.5 {
        let now = Instant::now();
        if enemy.last_attack.map_or(true, |t| now.duration_since(t) > Duration::from_secs(1)) {
          let damage = match enemy.monster_type {
            MonsterType::Demon => 8.0,
            MonsterType::Cacodemon => 5.0,
            MonsterType::Imp => 3.0
          };

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

  fn check_trophy(&mut self) {
    let px = self.player_pos.x.floor() as isize;
    let py = self.player_pos.y.floor() as isize;
    if let Some(TROPHY_TILE) = map_tile_at(&self.map, px, py) {
      if !self.trophy.collected {
        self.trophy.collected = true;
        self.game_won = true;
      }
    }
  }
}

// ----------------------------- Helper Functions --------------------------
fn check_collision(map: &Vec<Vec<u8>>, pos: Vec2, radius: f32) -> bool {
  let (x, y) = (pos.x, pos.y);
  let (min_x, max_x) = ((x - radius).floor() as isize, (x + radius).floor() as isize);
  let (min_y, max_y) = ((y - radius).floor() as isize, (y + radius).floor() as isize);

  for iy in min_y..=max_y {
    for ix in min_x..=max_x {
      if let Some(tile) = map_tile_at(map, ix, iy) {
        if tile == WALL_TILE || tile == DOOR_TILE {
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
      if tile == WALL_TILE || tile == DOOR_TILE {
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
    draw_game_over_screen(&mut app.doom_state, ctx);
    return;
  }
  if app.doom_state.game_won {
    draw_victory_screen(&mut app.doom_state, ctx);
    return;
  }

  egui::CentralPanel::default().show(ctx, |ui| {
    // Cast dt to f32 once here to avoid multiple casts later
    let dt = ui.input(|i| i.unstable_dt) as f32;
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter();

    // Moved these lines out of the closure to fix the borrowing error.
    let player_dir = app.doom_state.player_dir;
    let camera_plane = app.doom_state.camera_plane;

    // Handle input first to ensure state is up-to-date
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
      if i.key_pressed(Key::F) {
        app.doom_state.try_open_door();
      }
      if i.key_pressed(Key::Space) {
        app.doom_state.fire_weapon();
      }
    });

    // Update game state with the latest `dt`
    app.doom_state.spawn_enemies();
    app.doom_state.update_animations(dt);
    app.doom_state.update_enemies(dt);
    app.doom_state.check_trophy();

    // Get the most up-to-date state variables for rendering
    let player_pos = app.doom_state.player_pos;
    let player_dir = app.doom_state.player_dir;
    let camera_plane = app.doom_state.camera_plane;

    // Rendering
    draw_background(painter, rect);
    draw_walls(painter, rect, &app.doom_state);

    // Collect and sort all visible sprites (enemies and trophy) by distance
    let mut sprites_to_draw = Vec::new();

    // Enemies
    for enemy in &app.doom_state.enemies {
      let dist = vec2_distance(player_pos, enemy.pos);
      if is_visible(player_pos, player_dir, enemy.pos, &app.doom_state.map) {
        sprites_to_draw.push(("enemy", dist, enemy.pos));
      }
    }
    // Trophy
    if !app.doom_state.trophy.collected {
      let dist = vec2_distance(player_pos, app.doom_state.trophy.pos);
      if is_visible(player_pos, player_dir, app.doom_state.trophy.pos, &app.doom_state.map) {
        sprites_to_draw.push(("trophy", dist, app.doom_state.trophy.pos));
      }
    }

    // Sort sprites from farthest to nearest
    sprites_to_draw.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Less));

    // Draw sprites
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

    // Draw blood splatters
    for (pos, timer) in &app.doom_state.blood_splatters {
      if let Some((screen_pos, size_scale)) = world_to_screen(player_pos, rect, *pos, player_dir, camera_plane) {
        let alpha = (1.0 - timer / 5.0).max(0.0);
        let size = (size_scale * 0.5).min(50.0);
        painter.circle_filled(
          screen_pos,
          size * (1.0 + timer / 5.0),
          Color32::from_rgba_unmultiplied(150, 0, 0, (alpha * 150.0) as u8)
        );
      }
    }

    // Draw weapon
    draw_player_weapon(painter, rect, &app.doom_state);

    // Draw HUD
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
  // Fix: Correctly use size_scale for proportional scaling
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
        DOOR_TILE => Color32::from_rgb(150, 100, 50),
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

fn draw_game_over_screen(state: &mut DoomState, ctx: &egui::Context) {
  egui::CentralPanel::default().show(ctx, |ui| {
    ui.vertical_centered(|ui| {
      ui.add_space(50.0);
      ui.heading(egui::RichText::new("GAME OVER").color(Color32::RED));
      ui.label("Your journey ends here...");
      ui.add_space(20.0);

      if ui.button("Try Again").clicked() {
        *state = DoomState::default();
      }
    });
  });
}

fn draw_victory_screen(state: &mut DoomState, ctx: &egui::Context) {
  egui::CentralPanel::default().show(ctx, |ui| {
    ui.vertical_centered(|ui| {
      ui.add_space(50.0);
      ui.heading(egui::RichText::new("VICTORY!").color(Color32::GOLD));
      ui.label("You escaped with the sacred artifact!");
      ui.add_space(20.0);

      if ui.button("Play Again").clicked() {
        *state = DoomState::default();
      }
    });
  });
}

impl Default for DoomState {
  fn default() -> Self {
    let map = generate_dungeon();

    let mut player_pos = Vec2::ZERO;
    let mut trophy_pos = Vec2::ZERO;

    let mut empty_positions: Vec<(usize, usize)> = map.iter().enumerate()
      .flat_map(|(y, row)| row.iter().enumerate().filter(|(_, &tile)| tile == EMPTY_TILE).map(move |(x, _)| (x, y)))
      .collect();

    empty_positions.shuffle(&mut thread_rng());

    if let Some((x, y)) = empty_positions.pop() {
      player_pos = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
    }
    if let Some((x, y)) = empty_positions.pop() {
      trophy_pos = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
    }

    DoomState {
      health: 100.0,
      armor: 0.0,
      ammo: 20,
      player_pos,
      player_dir: Vec2::new(-1.0, 0.0),
      camera_plane: Vec2::new(0.0, 0.66),
      enemies: Vec::new(),
      map,
      trophy: Trophy {
        pos: trophy_pos,
        collected: false,
        glow_intensity: 0.0,
      },
      game_over: false,
      game_won: false,
      blood_splatters: Vec::new(),
      last_spawn_time: Instant::now(),
      last_shot_time: None,
    }
  }
}
