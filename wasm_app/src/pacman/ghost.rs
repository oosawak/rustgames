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
}

impl Ghost {
    pub fn new(ghost_type: GhostType, cell: f64) -> Self {
        let (start_row, start_col, scatter) = match ghost_type {
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
            ghost_type,
            mode: GhostMode::Chase,
            x: start_col as f64 * cell,
            y: start_row as f64 * cell,
            dir: 0,
            speed: cell * 6.0,
            cell,
            scatter_target: scatter,
            in_house,
            house_timer,
        }
    }

    pub fn reset(&mut self) {
        let (start_row, start_col) = match self.ghost_type {
            GhostType::Blinky => (11, 13),
            GhostType::Pinky  => (13, 13),
            GhostType::Inky   => (13, 11),
            GhostType::Clyde  => (13, 15),
        };
        self.x = start_col as f64 * self.cell;
        self.y = start_row as f64 * self.cell;
        self.dir = 0;
        self.mode = GhostMode::Chase;
        self.in_house = self.ghost_type != GhostType::Blinky;
        self.house_timer = match self.ghost_type {
            GhostType::Blinky => 0.0,
            GhostType::Pinky  => 3.0,
            GhostType::Inky   => 6.0,
            GhostType::Clyde  => 9.0,
        };
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

    fn near_center(&self) -> bool {
        let cx = (self.x / self.cell).round() * self.cell;
        let cy = (self.y / self.cell).round() * self.cell;
        (self.x - cx).abs() < self.cell * 0.3 && (self.y - cy).abs() < self.cell * 0.3
    }

    pub fn update(&mut self, dt: f32, player: &Player, blinky_pos: (f64, f64), map: &Map) {
        if self.in_house {
            self.house_timer -= dt;
            if self.house_timer <= 0.0 {
                self.in_house = false;
                self.x = 13.0 * self.cell;
                self.y = 11.0 * self.cell;
                self.dir = 1;
            } else {
                let bounce = if (self.house_timer * 2.0) as i32 % 2 == 0 { 1.0 } else { -1.0 };
                self.y += self.speed * 0.3 * dt as f64 * bounce;
                if self.y < 12.0 * self.cell { self.y = 12.0 * self.cell; }
                if self.y > 14.0 * self.cell { self.y = 14.0 * self.cell; }
                return;
            }
        }

        let spd = match self.mode {
            GhostMode::Frightened => self.speed * 0.5,
            GhostMode::Eaten => self.speed * 2.0,
            _ => self.speed,
        };

        if self.near_center() {
            let row = self.row();
            let col = self.col();
            let target = self.get_target(player, blinky_pos, row, col);
            let best_dir = self.choose_direction(row, col, target, map);
            self.dir = best_dir;
            self.x = col as f64 * self.cell;
            self.y = row as f64 * self.cell;
        }

        let d = self.dir as usize;
        self.x += DIR_DX[d] as f64 * spd * dt as f64;
        self.y += DIR_DY[d] as f64 * spd * dt as f64;

        // Tunnel wrap
        self.x = ((self.x % (COLS as f64 * self.cell)) + COLS as f64 * self.cell) % (COLS as f64 * self.cell);
    }

    fn get_target(&self, player: &Player, blinky_pos: (f64, f64), _row: i32, _col: i32) -> (i32, i32) {
        match self.mode {
            GhostMode::Scatter => self.scatter_target,
            GhostMode::Frightened => {
                let t = (self.x as i32 * 7 + self.y as i32 * 3).abs() % (ROWS as i32 * COLS as i32);
                (t / COLS as i32, t % COLS as i32)
            }
            GhostMode::Eaten => (11, 13),
            GhostMode::Chase => {
                let pr = player.row();
                let pc = player.col();
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
                        let tr = (mid_r + (mid_r - br)).clamp(0, ROWS as i32 - 1);
                        let tc = (mid_c + (mid_c - bc)).clamp(0, COLS as i32 - 1);
                        (tr, tc)
                    }
                    GhostType::Clyde => {
                        let dr = (self.row() - pr) as f64;
                        let dc = (self.col() - pc) as f64;
                        let dist = (dr * dr + dc * dc).sqrt();
                        if dist > 8.0 { (pr, pc) } else { self.scatter_target }
                    }
                }
            }
        }
    }

    fn choose_direction(&self, row: i32, col: i32, target: (i32, i32), map: &Map) -> i32 {
        let reverse = (self.dir + 2) % 4;
        let mut best_dir = self.dir;
        let mut best_dist = i64::MAX;

        for d in 0..4i32 {
            if d == reverse && self.mode != GhostMode::Frightened { continue; }
            let nr = row + DIR_DY[d as usize];
            let nc = col + DIR_DX[d as usize];
            if !map.is_passable(nr, nc) { continue; }
            // Ghosts can't enter ghost house from outside
            if map.tiles.get(nr as usize).and_then(|r| r.get(nc as usize)).copied().unwrap_or(1) == 4
                && !self.in_house && self.mode != GhostMode::Eaten {
                continue;
            }
            let dr = (nr - target.0) as i64;
            let dc = (nc - target.1) as i64;
            let dist = dr * dr + dc * dc;
            if self.mode == GhostMode::Frightened {
                if dist > best_dist || best_dist == i64::MAX {
                    best_dist = dist;
                    best_dir = d;
                }
            } else if dist < best_dist {
                best_dist = dist;
                best_dir = d;
            }
        }
        best_dir
    }

    pub fn pos(&self) -> (f64, f64) { (self.x, self.y) }

    pub fn draw(&self, ctx: &CanvasRenderingContext2d, offset_y: f64, time: f64) {
        let cx = self.x + self.cell / 2.0;
        let cy = self.y + self.cell / 2.0 + offset_y;
        let r = self.cell * 0.45;

        let color = match self.mode {
            GhostMode::Frightened => {
                if time % 1.0 < 0.5 { "#0000ff" } else { "#ffffff" }
            }
            GhostMode::Eaten => "#aaaaaa",
            _ => match self.ghost_type {
                GhostType::Blinky => "#ff0000",
                GhostType::Pinky  => "#ffb8ff",
                GhostType::Inky   => "#00ffff",
                GhostType::Clyde  => "#ffb852",
            }
        };

        ctx.begin_path();
        ctx.set_fill_style_str(color);
        let _ = ctx.arc(cx, cy, r, std::f64::consts::PI, 0.0);
        let bottom = cy + r;
        let wave_w = r * 2.0 / 3.0;
        ctx.line_to(cx + r, bottom);
        ctx.line_to(cx + r - wave_w / 2.0, bottom - r * 0.2);
        ctx.line_to(cx + r / 6.0, bottom);
        ctx.line_to(cx - r / 6.0, bottom - r * 0.2);
        ctx.line_to(cx - r + wave_w / 2.0, bottom);
        ctx.line_to(cx - r, bottom);
        ctx.close_path();
        ctx.fill();

        if self.mode != GhostMode::Frightened && self.mode != GhostMode::Eaten {
            ctx.begin_path();
            ctx.set_fill_style_str("white");
            let _ = ctx.arc(cx - r * 0.3, cy - r * 0.1, r * 0.25, 0.0, std::f64::consts::TAU);
            ctx.fill();
            ctx.begin_path();
            let _ = ctx.arc(cx + r * 0.3, cy - r * 0.1, r * 0.25, 0.0, std::f64::consts::TAU);
            ctx.fill();
            ctx.begin_path();
            ctx.set_fill_style_str("blue");
            let _ = ctx.arc(cx - r * 0.3, cy - r * 0.1, r * 0.12, 0.0, std::f64::consts::TAU);
            ctx.fill();
            ctx.begin_path();
            let _ = ctx.arc(cx + r * 0.3, cy - r * 0.1, r * 0.12, 0.0, std::f64::consts::TAU);
            ctx.fill();
        }
    }
}
