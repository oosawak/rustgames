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
        }
    }

    pub fn reset(&mut self) {
        self.x = 13.0 * self.cell;
        self.y = 26.0 * self.cell;
        self.dir = 3; self.next_dir = 3;
        self.alive = true; self.anim_frame = 0.0;
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
        let thresh = self.cell * 0.45; // セル中心への吸着閾値

        // 毎フレーム: next_dir への転換を試みる
        let nd = self.next_dir;
        if nd != self.dir {
            let is_reverse = nd == (self.dir + 2) % 4;
            let nd_is_horiz = nd == 1 || nd == 3;

            let can_turn = if is_reverse {
                // 逆方向: 両軸ともセル中心近くなら即転換
                (self.x - cx).abs() < thresh && (self.y - cy).abs() < thresh
            } else {
                // 直角方向: 垂直軸がセル中心近くで、かつ進行方向が通れる
                let aligned = if nd_is_horiz {
                    (self.y - cy).abs() < thresh
                } else {
                    (self.x - cx).abs() < thresh
                };
                if aligned {
                    let nr = row + DIR_DY[nd as usize];
                    let nc = (col + DIR_DX[nd as usize]).rem_euclid(COLS as i32);
                    map.is_passable(nr, nc)
                } else {
                    false
                }
            };

            if can_turn {
                self.dir = nd;
                // 垂直軸をセル中心にスナップ
                if nd_is_horiz { self.y = cy; } else { self.x = cx; }
            }
        }

        // 現在方向への移動: 次のセルが壁なら止まる
        let d = self.dir as usize;
        let next_r = row + DIR_DY[d];
        let next_c = (col + DIR_DX[d]).rem_euclid(COLS as i32);

        // セル境界を越えるかどうかチェック
        let half = self.cell * 0.5;
        let will_cross = match d {
            0 => self.y - dist < cy - half,   // 上
            1 => self.x + dist > cx + half,   // 右
            2 => self.y + dist > cy + half,   // 下
            3 => self.x - dist < cx - half,   // 左
            _ => false,
        };

        if will_cross && map.is_wall(next_r, next_c) {
            // 壁手前のセル中央にスナップして停止
            self.x = cx;
            self.y = cy;
            return;
        }

        // トンネル (row 13)
        if row == 13 {
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
