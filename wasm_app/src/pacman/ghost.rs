use web_sys::CanvasRenderingContext2d;
use crate::pacman::map::{Map, COLS, ROWS};
use crate::pacman::player::{Player, DIR_DX, DIR_DY};

#[derive(Clone, Copy, PartialEq)]
pub enum GhostType { Blinky, Pinky, Inky, Clyde }

#[derive(Clone, Copy, PartialEq)]
pub enum GhostMode { Chase, Scatter, Frightened, Eaten }

pub struct Ghost {
    pub ghost_type: GhostType,
    pub mode: GhostMode,
    pub x: f64,
    pub y: f64,
    pub dir: i32,
    pub speed: f64,
    pub cell: f64,
    scatter_target: (i32, i32),
    pub in_house: bool,
    house_timer: f32,
    last_cell: (i32, i32),
}

impl Ghost {
    pub fn new(ghost_type: GhostType, cell: f64) -> Self {
        let (sr, sc, scatter) = match ghost_type {
            GhostType::Blinky => (11, 13, (0, 25)),
            GhostType::Pinky  => (13, 13, (0, 2)),
            GhostType::Inky   => (13, 11, (30, 27)),
            GhostType::Clyde  => (13, 15, (30, 0)),
        };
        let in_house = ghost_type != GhostType::Blinky;
        let house_timer = match ghost_type {
            GhostType::Blinky => 0.0,
            GhostType::Pinky  => 3.0,
            GhostType::Inky   => 6.0,
            GhostType::Clyde  => 9.0,
        };
        Ghost {
            ghost_type, mode: GhostMode::Chase,
            x: sc as f64 * cell, y: sr as f64 * cell,
            dir: 0, speed: cell * 6.0, cell,
            scatter_target: scatter, in_house, house_timer,
            last_cell: (sr, sc),
        }
    }

    pub fn reset(&mut self) {
        let (sr, sc) = match self.ghost_type {
            GhostType::Blinky => (11, 13),
            GhostType::Pinky  => (13, 13),
            GhostType::Inky   => (13, 11),
            GhostType::Clyde  => (13, 15),
        };
        self.x = sc as f64 * self.cell;
        self.y = sr as f64 * self.cell;
        self.dir = 0; self.mode = GhostMode::Chase;
        self.in_house = self.ghost_type != GhostType::Blinky;
        self.house_timer = match self.ghost_type {
            GhostType::Blinky => 0.0, GhostType::Pinky => 3.0,
            GhostType::Inky => 6.0, GhostType::Clyde => 9.0,
        };
        self.last_cell = (sr, sc);
    }

    pub fn frighten(&mut self) {
        if self.mode != GhostMode::Eaten {
            self.mode = GhostMode::Frightened;
            self.dir = (self.dir + 2) % 4;
        }
    }

    pub fn row(&self) -> i32 { (self.y / self.cell).round() as i32 }
    pub fn col(&self) -> i32 {
        let c = (self.x / self.cell).round() as i32;
        ((c % COLS as i32) + COLS as i32) % COLS as i32
    }

    pub fn update(&mut self, dt: f32, player: &Player, blinky_pos: (f64, f64), map: &Map) {
        // ゴーストハウス内でバウンス
        if self.in_house {
            self.house_timer -= dt;
            if self.house_timer <= 0.0 {
                self.in_house = false;
                self.x = 13.0 * self.cell;
                self.y = 11.0 * self.cell;
                self.dir = 1;
                self.last_cell = (11, 13);
            } else {
                let bounce = if (self.house_timer * 2.0) as i32 % 2 == 0 { 1.0 } else { -1.0 };
                self.y += self.speed * 0.3 * dt as f64 * bounce;
                self.y = self.y.clamp(12.0 * self.cell, 14.0 * self.cell);
                return;
            }
        }

        let spd = match self.mode {
            GhostMode::Frightened => self.speed * 0.5,
            GhostMode::Eaten => self.speed * 2.0,
            _ => self.speed,
        };
        let dist = spd * dt as f64;

        let row = self.row();
        let col = self.col();
        let cx = col as f64 * self.cell;
        let cy = row as f64 * self.cell;

        // 新しいセルに入ったときに方向決定（スナップ付き）
        if (row, col) != self.last_cell {
            self.last_cell = (row, col);
            // セル中央にスナップ
            self.x = cx; self.y = cy;
            let target = self.get_target(player, blinky_pos, row, col);
            self.dir = self.choose_direction(row, col, target, map);
        }

        // 進行方向に壁があれば止まる（行き詰まり防止）
        let d = self.dir as usize;
        let next_r = row + DIR_DY[d];
        let next_c = col + DIR_DX[d];
        let tile = map.tiles.get(next_r as usize).and_then(|r| r.get(next_c as usize)).copied().unwrap_or(1);
        let is_house = tile == 4 && !self.in_house && self.mode != GhostMode::Eaten;

        let crossing = match d {
            0 => self.y - dist < cy,
            1 => self.x + dist > cx + self.cell - 1.0,
            2 => self.y + dist > cy + self.cell - 1.0,
            3 => self.x - dist < cx,
            _ => false,
        };
        if crossing && (map.is_wall(next_r, next_c) || is_house) {
            self.x = cx; self.y = cy;
            return;
        }

        // トンネルラップ
        let w = COLS as f64 * self.cell;
        self.x = ((self.x + DIR_DX[d] as f64 * dist) % w + w) % w;
        self.y += DIR_DY[d] as f64 * dist;
    }

    fn get_target(&self, player: &Player, blinky_pos: (f64, f64), _row: i32, _col: i32) -> (i32, i32) {
        match self.mode {
            GhostMode::Scatter => self.scatter_target,
            GhostMode::Frightened => {
                // 疑似ランダムでバラバラの方向へ
                let seed = (self.x as i32).wrapping_mul(7) ^ (self.y as i32).wrapping_mul(13);
                let t = seed.unsigned_abs() as usize % (ROWS * COLS);
                ((t / COLS) as i32, (t % COLS) as i32)
            }
            GhostMode::Eaten => (11, 13),
            GhostMode::Chase => {
                let pr = player.row(); let pc = player.col();
                let pd = player.dir as usize;
                match self.ghost_type {
                    GhostType::Blinky => (pr, pc),
                    GhostType::Pinky => {
                        let tr = (pr + DIR_DY[pd] * 4).clamp(0, ROWS as i32 - 1);
                        let tc = (pc + DIR_DX[pd] * 4).clamp(0, COLS as i32 - 1);
                        (tr, tc)
                    }
                    GhostType::Inky => {
                        let mid_r = pr + DIR_DY[pd] * 2;
                        let mid_c = pc + DIR_DX[pd] * 2;
                        let br = (blinky_pos.1 / self.cell).round() as i32;
                        let bc = (blinky_pos.0 / self.cell).round() as i32;
                        ((mid_r + (mid_r - br)).clamp(0, ROWS as i32 - 1),
                         (mid_c + (mid_c - bc)).clamp(0, COLS as i32 - 1))
                    }
                    GhostType::Clyde => {
                        let dr = (self.row() - pr) as f64;
                        let dc = (self.col() - pc) as f64;
                        if (dr * dr + dc * dc).sqrt() > 8.0 { (pr, pc) } else { self.scatter_target }
                    }
                }
            }
        }
    }

    fn choose_direction(&self, row: i32, col: i32, target: (i32, i32), map: &Map) -> i32 {
        let reverse = (self.dir + 2) % 4;
        let mut best_dir = -1i32;
        let mut best_dist = if self.mode == GhostMode::Frightened { i64::MIN } else { i64::MAX };

        for d in 0..4i32 {
            if d == reverse { continue; }
            let nr = row + DIR_DY[d as usize];
            let nc = col + DIR_DX[d as usize];
            if !map.is_passable(nr, nc) { continue; }
            let tile = map.tiles.get(nr as usize).and_then(|r| r.get(nc as usize)).copied().unwrap_or(1);
            if tile == 4 && !self.in_house && self.mode != GhostMode::Eaten { continue; }
            let dr = (nr - target.0) as i64;
            let dc = (nc - target.1) as i64;
            let dist = dr * dr + dc * dc;
            let better = if self.mode == GhostMode::Frightened { dist > best_dist } else { dist < best_dist };
            if better || best_dir < 0 {
                best_dist = dist;
                best_dir = d;
            }
        }
        // fallback: reverse direction
        if best_dir < 0 { reverse } else { best_dir }
    }

    pub fn pos(&self) -> (f64, f64) { (self.x, self.y) }

    pub fn draw(&self, ctx: &CanvasRenderingContext2d, offset_y: f64, time: f64) {
        let cx = self.x + self.cell / 2.0;
        let cy = self.y + self.cell / 2.0 + offset_y;
        let r = self.cell * 0.45;
        let pi = std::f64::consts::PI;

        let color = match self.mode {
            GhostMode::Frightened => if time % 1.0 < 0.5 { "#0000ff" } else { "#ffffff" },
            GhostMode::Eaten => "#aaaaaa",
            _ => match self.ghost_type {
                GhostType::Blinky => "#ff0000",
                GhostType::Pinky  => "#ffb8ff",
                GhostType::Inky   => "#00ffff",
                GhostType::Clyde  => "#ffb852",
            }
        };
        ctx.set_fill_style_str(color);

        // 頭（半円）
        ctx.begin_path();
        let _ = ctx.arc(cx, cy - r * 0.1, r, pi, 0.0);
        // スカートの波形
        let steps = 4i32;
        let wave_r = r / steps as f64;
        ctx.line_to(cx + r, cy + r * 0.7);
        for i in (0..steps).rev() {
            let x1 = cx + r * (2.0 * (i + 1) as f64 / steps as f64 - 1.0);
            let x0 = cx + r * (2.0 * i as f64 / steps as f64 - 1.0);
            let mid = (x0 + x1) / 2.0;
            let peak = if i % 2 == 0 { cy + r * 0.7 } else { cy + r * 0.9 };
            let _ = ctx.arc(mid, peak - wave_r / 2.0, wave_r, 0.0, pi);
            let _ = ctx.line_to(x0, cy + r * 0.7);
        }
        ctx.close_path();
        ctx.fill();

        // 目
        ctx.set_fill_style_str("white");
        for side in [-1.0f64, 1.0] {
            ctx.begin_path();
            let ex = cx + side * r * 0.33;
            let ey = cy - r * 0.25;
            let _ = ctx.arc(ex, ey, r * 0.22, 0.0, 2.0 * pi);
            ctx.fill();
        }
        if self.mode != GhostMode::Eaten {
            ctx.set_fill_style_str("#00f");
            for side in [-1.0f64, 1.0] {
                ctx.begin_path();
                let ex = cx + side * r * 0.33 + DIR_DX[self.dir as usize] as f64 * r * 0.1;
                let ey = cy - r * 0.25 + DIR_DY[self.dir as usize] as f64 * r * 0.1;
                let _ = ctx.arc(ex, ey, r * 0.12, 0.0, 2.0 * pi);
                ctx.fill();
            }
        }
    }
}
