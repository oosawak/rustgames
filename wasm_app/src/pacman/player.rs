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
    last_cell: (i32, i32),
}

// 0=上, 1=右, 2=下, 3=左
pub const DIR_DX: [i32; 4] = [0, 1, 0, -1];
pub const DIR_DY: [i32; 4] = [-1, 0, 1, 0];

impl Player {
    pub fn new(cell: f64) -> Self {
        Player {
            x: 13.0 * cell, y: 26.0 * cell,
            dir: 3, next_dir: 3,
            speed: cell * 7.5,
            alive: true, anim_frame: 0.0, cell,
            last_cell: (26, 13),
        }
    }

    pub fn reset(&mut self) {
        self.x = 13.0 * self.cell;
        self.y = 26.0 * self.cell;
        self.dir = 3; self.next_dir = 3;
        self.alive = true; self.anim_frame = 0.0;
        self.last_cell = (26, 13);
    }

    pub fn row(&self) -> i32 { (self.y / self.cell).round() as i32 }
    pub fn col(&self) -> i32 {
        let c = (self.x / self.cell).round() as i32;
        ((c % COLS as i32) + COLS as i32) % COLS as i32
    }
    pub fn grid_pos(&self) -> (i32, i32) { (self.row(), self.col()) }

    pub fn update(&mut self, dt: f32, map: &Map) {
        if !self.alive { return; }
        self.anim_frame += dt as f64 * 8.0;

        let dist = self.speed * dt as f64;
        let row = self.row();
        let col = self.col();
        let cx = col as f64 * self.cell;
        let cy = row as f64 * self.cell;

        // 新しいセルに入ったとき方向転換を試みる
        if (row, col) != self.last_cell {
            self.last_cell = (row, col);

            // 逆方向は即転換
            if self.next_dir == (self.dir + 2) % 4 {
                self.dir = self.next_dir;
                self.x = cx; self.y = cy;
            } else {
                // 直角方向: 先が通れるなら転換
                let nd = self.next_dir;
                let nr = row + DIR_DY[nd as usize];
                let nc = col + DIR_DX[nd as usize];
                if map.is_passable(nr, nc) {
                    self.dir = nd;
                    self.x = cx; self.y = cy;
                }
            }
        }

        // 前方の壁チェック
        let d = self.dir as usize;
        let next_r = row + DIR_DY[d];
        let next_c = col + DIR_DX[d];

        // セル境界を越えるか？
        let crossing = match d {
            0 => self.y - dist < cy,          // 上: 現在セルの上端を越える
            1 => self.x + dist > cx + self.cell - 1.0, // 右
            2 => self.y + dist > cy + self.cell - 1.0, // 下
            3 => self.x - dist < cx,          // 左
            _ => false,
        };

        if crossing && map.is_wall(next_r, next_c) {
            // 壁手前のセル中央で止まる
            self.x = cx; self.y = cy;
            return;
        }

        // トンネル
        if row == 13 || row == 14 {
            let w = COLS as f64 * self.cell;
            self.x = ((self.x + DIR_DX[d] as f64 * dist) % w + w) % w;
            self.y += DIR_DY[d] as f64 * dist;
            return;
        }

        self.x += DIR_DX[d] as f64 * dist;
        self.y += DIR_DY[d] as f64 * dist;
    }

    pub fn draw(&self, ctx: &CanvasRenderingContext2d, offset_y: f64) {
        if !self.alive { return; }
        let cx = self.x + self.cell / 2.0;
        let cy = self.y + self.cell / 2.0 + offset_y;
        let r  = self.cell * 0.45;
        let pi = std::f64::consts::PI;
        let mouth = ((self.anim_frame * pi).sin().abs() * pi * 0.25).max(0.05);
        let base = match self.dir { 0 => -pi/2.0, 1 => 0.0, 2 => pi/2.0, _ => pi };
        ctx.begin_path();
        ctx.set_fill_style_str("yellow");
        let _ = ctx.arc(cx, cy, r, base + mouth, base + 2.0*pi - mouth);
        let _ = ctx.line_to(cx, cy);
        ctx.close_path();
        ctx.fill();
    }
}
