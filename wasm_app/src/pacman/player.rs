use web_sys::CanvasRenderingContext2d;
use crate::pacman::map::{Map, COLS};

pub struct Player {
    pub x: f64,
    pub y: f64,
    pub dir: i32,
    pub next_dir: i32,
    pub speed: f64,
    pub alive: bool,
    pub anim_frame: f64,
    pub cell: f64,
}

// Direction vectors: 0=up, 1=right, 2=down, 3=left
pub const DIR_DX: [i32; 4] = [0, 1, 0, -1];
pub const DIR_DY: [i32; 4] = [-1, 0, 1, 0];

impl Player {
    pub fn new(cell: f64) -> Self {
        let start_col = 13.5;
        let start_row = 23.0;
        Player {
            x: start_col * cell,
            y: start_row * cell,
            dir: 3,
            next_dir: 3,
            speed: cell * 7.5,
            alive: true,
            anim_frame: 0.0,
            cell,
        }
    }

    pub fn reset(&mut self) {
        let start_col = 13.5;
        let start_row = 23.0;
        self.x = start_col * self.cell;
        self.y = start_row * self.cell;
        self.dir = 3;
        self.next_dir = 3;
        self.alive = true;
        self.anim_frame = 0.0;
    }

    pub fn row(&self) -> i32 {
        (self.y / self.cell).round() as i32
    }

    pub fn col(&self) -> i32 {
        let c = (self.x / self.cell).round() as i32;
        ((c % COLS as i32) + COLS as i32) % COLS as i32
    }

    pub fn grid_pos(&self) -> (i32, i32) {
        (self.row(), self.col())
    }

    fn near_center(&self) -> bool {
        let cx = (self.x / self.cell).round() * self.cell;
        let cy = (self.y / self.cell).round() * self.cell;
        let dx = (self.x - cx).abs();
        let dy = (self.y - cy).abs();
        dx < self.cell * 0.4 && dy < self.cell * 0.4
    }

    pub fn update(&mut self, dt: f32, map: &Map) {
        if !self.alive { return; }

        self.anim_frame += dt as f64 * 8.0;

        // Try to apply next_dir when near cell center
        if self.near_center() {
            let row = self.row();
            let col = self.col();
            let nd = self.next_dir;
            let nr = row + DIR_DY[nd as usize];
            let nc = col + DIR_DX[nd as usize];
            if map.is_passable(nr, nc) {
                self.dir = self.next_dir;
                self.x = col as f64 * self.cell;
                self.y = row as f64 * self.cell;
            }
        }

        let d = self.dir as usize;
        let dx = DIR_DX[d] as f64 * self.speed * dt as f64;
        let dy = DIR_DY[d] as f64 * self.speed * dt as f64;

        let new_x = self.x + dx;
        let new_y = self.y + dy;

        // Tunnel wrap-around
        let row = self.row();
        if row == 13 || row == 14 {
            self.x = ((new_x % (COLS as f64 * self.cell)) + COLS as f64 * self.cell) % (COLS as f64 * self.cell);
            self.y = new_y;
            return;
        }

        // Check if new position would hit a wall
        let next_row = ((new_y + self.cell * 0.4 * DIR_DY[d] as f64) / self.cell).floor() as i32;
        let next_col = ((new_x + self.cell * 0.4 * DIR_DX[d] as f64) / self.cell).floor() as i32;

        if !map.is_wall(next_row, next_col) {
            self.x = new_x;
            self.y = new_y;
        } else {
            self.x = self.col() as f64 * self.cell;
            self.y = self.row() as f64 * self.cell;
        }
    }

    pub fn draw(&self, ctx: &CanvasRenderingContext2d, offset_y: f64) {
        if !self.alive { return; }

        let cx = self.x + self.cell / 2.0;
        let cy = self.y + self.cell / 2.0 + offset_y;
        let r = self.cell * 0.45;

        let mouth_open = ((self.anim_frame * std::f64::consts::PI).sin().abs() * std::f64::consts::FRAC_PI_4).max(0.05);

        let angle_offset = match self.dir {
            0 => -std::f64::consts::FRAC_PI_2,
            1 => 0.0,
            2 => std::f64::consts::FRAC_PI_2,
            3 => std::f64::consts::PI,
            _ => 0.0,
        };

        ctx.begin_path();
        ctx.set_fill_style_str("yellow");
        let start_angle = angle_offset + mouth_open;
        let end_angle = angle_offset + std::f64::consts::TAU - mouth_open;
        let _ = ctx.arc(cx, cy, r, start_angle, end_angle);
        let _ = ctx.line_to(cx, cy);
        ctx.close_path();
        ctx.fill();
    }
}
