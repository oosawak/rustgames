// cubeboy.rs — CubeBoy（rustgames 版）
//
// html-player 版 CubeBoy（JS）の移植。ダッシュと壁ジャンプの小さな
// プラットフォーマー。部屋を巡ってオーブを 3 つ集めるとボス出現、
// 最初の部屋の中心の出口へ逃げ切ればクリア。
// 部屋生成は JS 版と同一のシード規則（FNV-1a + mulberry32）で再現する。
// 描画は render2d（wgpu 2D バッチ層）、仮想解像度 128x128、30Hz 固定ステップ。

use std::collections::{HashMap, HashSet};
use wasm_bindgen::JsCast;

use crate::render2d::{Draw2d, Renderer2d, Rgba};
use crate::rng::{hash_seed, GameRng, JsRng};

pub const VW: f32 = 128.0;
pub const VH: f32 = 128.0;
const TS: f32 = 8.0;

// Pyxel パレット（部分）
const C_BG: Rgba = Rgba::hex(0x000000);
const C_WALL: Rgba = Rgba::hex(0x2B335F);
const C_WALLD: Rgba = Rgba::hex(0x5F574F);
const C_WHITE: Rgba = Rgba::hex(0xFFF1E8);
const C_PINK: Rgba = Rgba::hex(0xFF77A8);
const C_CYAN: Rgba = Rgba::hex(0x29ADFF);
const C_GRAYC: Rgba = Rgba::hex(0xC2C3C7);
const C_YEL: Rgba = Rgba::hex(0xFFEC27);
const C_RED: Rgba = Rgba::hex(0xFF004D);
const C_MTN1: Rgba = Rgba::hex(0x7E2553);
const C_MTN2: Rgba = Rgba::hex(0x5F574F);
const C_ROOM: Rgba = Rgba::hex(0x83769C);
const C_PORTAL: Rgba = Rgba::hex(0x00E436);
const C_BURST2: Rgba = Rgba::hex(0x00BFFF);
const C_BLACK: Rgba = Rgba::hex(0x000000);

// audio_event: 0=なし 1=ジャンプ 2=ダッシュ 3=オーブ 4=死亡
#[derive(Clone, Copy, PartialEq)]
pub enum Scene {
    Title = 0,
    Main = 1,
    OverSeq = 2,
    Over = 3,
    Clear = 4,
}

#[derive(Default, Clone, Copy)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub a: bool,
    pub b: bool,
}

#[derive(Clone, Copy)]
struct Particle {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    col: Rgba,
    life: i32,
}

#[derive(Clone, Copy)]
struct Orb {
    x: i32,
    y: i32,
    active: bool,
    timer: i32,
}

struct Player {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    w: f32,
    h: f32,
    on_ground: bool,
    on_wall: i32,
    can_dash: bool,
    dash_time: i32,
    dash_dir: (f32, f32),
    coyote: i32,
    buf: i32,
    sx: f32,
    sy: f32,
    facing: i32,
    dead: bool,
}

impl Player {
    fn new(x: f32, y: f32) -> Self {
        Player {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            w: 6.0,
            h: 6.0,
            on_ground: false,
            on_wall: 0,
            can_dash: true,
            dash_time: 0,
            dash_dir: (0.0, 0.0),
            coyote: 0,
            buf: 0,
            sx: 1.0,
            sy: 1.0,
            facing: 1,
            dead: false,
        }
    }
}

struct Boss {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    speed: f32,
    spawned: bool,
}

impl Boss {
    fn new() -> Self {
        Boss { x: -100.0, y: -100.0, w: 24.0, h: 24.0, speed: 0.8, spawned: false }
    }
}

// ── 部屋生成（JS 版 genRoom と同一規則） ────────────────────────────────────

fn gen_room(rx: i32, ry: i32) -> [u8; 256] {
    let mut rng = JsRng::new(hash_seed(&format!("{}_{}_cubeboy", rx, ry)));
    let mut tm = [0u8; 256];
    let s = |tm: &mut [u8; 256], x: i32, y: i32, v: u8| {
        if (0..16).contains(&x) && (0..16).contains(&y) {
            tm[(y * 16 + x) as usize] = v;
        }
    };
    for x in 0..16 {
        for y in 0..16 {
            let edge = x == 0 || x == 15 || y == 0 || y == 15;
            let exit = ((x == 7 || x == 8) && (y == 0 || y == 15))
                || ((y == 7 || y == 8) && (x == 0 || x == 15));
            if edge && !exit {
                s(&mut tm, x, y, 1);
            }
        }
    }
    let density = rng.rr(0.7, 0.9);
    for gy in 1..7 {
        for gx in 1..7 {
            if rng.next_f64() < density {
                let px = gx * 2;
                let py = gy * 2;
                let ty = rng.ri(0, 3);
                let size = rng.ri(2, 3);
                match ty {
                    0 => {
                        for i in 0..size {
                            if 0 < px + i && px + i < 15 {
                                s(&mut tm, px + i, py, 1);
                            }
                        }
                    }
                    1 => {
                        for i in 0..size {
                            if 0 < py + i && py + i < 15 {
                                s(&mut tm, px, py + i, 1);
                            }
                        }
                    }
                    2 => {
                        for i in 0..size {
                            if 0 < px + i && px + i < 15 {
                                s(&mut tm, px + i, py, 1);
                            }
                        }
                        for i in 0..size {
                            if 0 < py + i && py + i < 15 {
                                s(&mut tm, px, py + i, 1);
                            }
                        }
                    }
                    _ => {
                        s(&mut tm, px, py, 1);
                    }
                }
            }
        }
    }
    for i in 7..9 {
        for j in 0..2 {
            s(&mut tm, i, j, 0);
        }
        for j in 14..16 {
            s(&mut tm, i, j, 0);
        }
    }
    for j in 7..9 {
        for i in 0..2 {
            s(&mut tm, i, j, 0);
        }
        for i in 14..16 {
            s(&mut tm, i, j, 0);
        }
    }
    for i in 6..10 {
        for j in 6..10 {
            s(&mut tm, i, j, 0);
        }
    }
    for i in 11..15 {
        for j in 1..4 {
            s(&mut tm, i, j, 0);
        }
    }
    // トゲ（タイル 2）: 壁に隣接する空きへ
    for _ in 0..15 {
        let tx = rng.ri(1, 14);
        let ty = rng.ri(1, 14);
        if tx == 7 || tx == 8 || ty == 7 || ty == 8 {
            continue;
        }
        if (6..=9).contains(&tx) && (6..=9).contains(&ty) {
            continue;
        }
        if tx >= 11 && ty <= 3 {
            continue;
        }
        if tm[(ty * 16 + tx) as usize] == 0 {
            let mut adj = false;
            for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let ax = tx + dx;
                let ay = ty + dy;
                if (0..16).contains(&ax) && (0..16).contains(&ay) && tm[(ay * 16 + ax) as usize] == 1 {
                    adj = true;
                    break;
                }
            }
            if adj {
                s(&mut tm, tx, ty, 2);
            }
        }
    }
    tm
}

// ── ゲーム本体 ──────────────────────────────────────────────────────────────

pub struct CubeBoyGame {
    renderer: Renderer2d,
    draw: Draw2d,
    scene: Scene,
    boss_mode: bool,
    room_x: i32,
    room_y: i32,
    tm: [u8; 256],
    rooms_data: HashMap<(i32, i32), Vec<Orb>>,
    collected_rooms: HashSet<(i32, i32)>,
    collected: u32,
    orbs: Vec<Orb>,
    player: Player,
    boss: Boss,
    parts: Vec<Particle>,
    countdown: i32,
    death_timer: i32,
    shake: f32,
    frame: u64,
    pub input: Input,
    prev_a: bool,
    prev_b: bool,
    prev_up: bool,
    tap_pending: bool,
    pub audio_event: u8,
    rng: GameRng,
    prev_ts: f64,
    acc: f64,
}

impl CubeBoyGame {
    pub async fn new(canvas_id: &str) -> Result<Self, String> {
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(canvas_id)
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| "canvas cast failed")?;
        let renderer = Renderer2d::new(canvas, VW, VH).await?;
        Ok(CubeBoyGame {
            renderer,
            draw: Draw2d::new(),
            scene: Scene::Title,
            boss_mode: false,
            room_x: 0,
            room_y: 0,
            tm: gen_room(0, 0),
            rooms_data: HashMap::new(),
            collected_rooms: HashSet::new(),
            collected: 0,
            orbs: Vec::new(),
            player: Player::new(64.0, 64.0),
            boss: Boss::new(),
            parts: Vec::new(),
            countdown: 0,
            death_timer: 0,
            shake: 0.0,
            frame: 0,
            input: Input::default(),
            prev_a: false,
            prev_b: false,
            prev_up: false,
            tap_pending: false,
            audio_event: 0,
            rng: GameRng::new(),
            prev_ts: 0.0,
            acc: 0.0,
        })
    }

    pub fn scene_u8(&self) -> u8 {
        self.scene as u8
    }

    pub fn collected_count(&self) -> u32 {
        self.collected
    }

    pub fn start(&mut self) {
        if self.scene != Scene::Main {
            self.reset_game();
        }
    }

    pub fn press_a(&mut self) {
        self.tap_pending = true;
    }

    fn reset_game(&mut self) {
        self.room_x = 0;
        self.room_y = 0;
        self.rooms_data.clear();
        self.collected_rooms.clear();
        self.collected = 0;
        self.countdown = 0;
        self.death_timer = 0;
        self.shake = 0.0;
        self.parts.clear();
        self.player = Player::new(64.0, 64.0);
        self.boss = Boss::new();
        self.boss_mode = false;
        self.scene = Scene::Main;
        self.enter_room();
    }

    fn enter_room(&mut self) {
        self.tm = gen_room(self.room_x, self.room_y);
        self.orbs = self.room_orbs(self.room_x, self.room_y);
        self.parts.clear();
    }

    fn room_orbs(&mut self, rx: i32, ry: i32) -> Vec<Orb> {
        if rx == 0 && ry == 0 {
            return Vec::new();
        }
        if self.collected_rooms.contains(&(rx, ry)) {
            return Vec::new();
        }
        if let Some(o) = self.rooms_data.get(&(rx, ry)) {
            return o.clone();
        }
        let tm = self.tm;
        let mut rng = JsRng::new(hash_seed(&format!("{}_{}_cubeboy_orb", rx, ry)));
        let mut orbs: Vec<Orb> = Vec::new();
        for _ in 0..20 {
            if !orbs.is_empty() {
                break;
            }
            let ox = rng.ri(24, 104);
            let oy = rng.ri(24, 104);
            let tx = ox / 8;
            let ty = oy / 8;
            if tm[(ty * 16 + tx) as usize] == 0 && !((6..=9).contains(&tx) && (6..=9).contains(&ty)) {
                let mut near = false;
                'outer: for dx in -2..3 {
                    for dy in -2..3 {
                        let ax = tx + dx;
                        let ay = ty + dy;
                        if (0..16).contains(&ax)
                            && (0..16).contains(&ay)
                            && tm[(ay * 16 + ax) as usize] == 1
                        {
                            near = true;
                            break 'outer;
                        }
                    }
                }
                if near {
                    orbs = vec![Orb { x: ox, y: oy, active: true, timer: 0 }];
                }
            }
        }
        if orbs.is_empty() {
            orbs = vec![Orb { x: 64, y: 64, active: true, timer: 0 }];
        }
        self.rooms_data.insert((rx, ry), orbs.clone());
        orbs
    }

    fn is_wall(&self, x: f32, y: f32) -> bool {
        let p = &self.player;
        let x1 = (x / TS).floor() as i32;
        let y1 = (y / TS).floor() as i32;
        let x2 = ((x + p.w - 0.1) / TS).floor() as i32;
        let y2 = ((y + p.h - 0.1) / TS).floor() as i32;
        for ty in y1..=y2 {
            if ty < 0 || ty >= 16 {
                continue;
            }
            for tx in x1..=x2 {
                if tx < 0 || tx >= 16 {
                    continue;
                }
                if self.tm[(ty * 16 + tx) as usize] == 1 {
                    return true;
                }
            }
        }
        false
    }

    fn resolve_overlap(&mut self) {
        if self.is_wall(self.player.x, self.player.y) {
            for r in 1..16 {
                for dx in -r..=r {
                    for dy in -r..=r {
                        let (nx, ny) = (self.player.x + dx as f32, self.player.y + dy as f32);
                        if !self.is_wall(nx, ny) {
                            self.player.x = nx;
                            self.player.y = ny;
                            return;
                        }
                    }
                }
            }
        }
    }

    fn burst(&mut self, x: f32, y: f32, col: Rgba, n: i32, pow: f32) {
        for _ in 0..n {
            let dx = self.rng.range_f32(-pow, pow);
            let dy = self.rng.range_f32(-pow, pow);
            self.parts.push(Particle { x, y, dx, dy, col, life: 20 });
        }
    }

    // ── 更新 ────────────────────────────────────────────────────────────────

    pub fn tick(&mut self, ts: f64) {
        if self.prev_ts == 0.0 {
            self.prev_ts = ts;
        }
        let dt = ((ts - self.prev_ts) / 1000.0).min(0.25);
        self.prev_ts = ts;
        self.acc += dt;
        let h = 1.0 / 30.0;
        let mut n = 0;
        while self.acc >= h && n < 4 {
            self.step();
            self.acc -= h;
            n += 1;
        }
        if n == 4 {
            self.acc = 0.0;
        }
        self.draw();
        let clear = C_BG;
        self.renderer.render(&self.draw, clear);
    }

    fn step(&mut self) {
        self.frame += 1;
        let a_edge = self.input.a && !self.prev_a;
        let b_edge = self.input.b && !self.prev_b;
        self.prev_a = self.input.a;
        self.prev_b = self.input.b;
        let tap_now = self.tap_pending;
        self.tap_pending = false;
        let start_pressed = self.input.a || a_edge || tap_now;
        match self.scene {
            Scene::Title => {
                if start_pressed {
                    self.reset_game();
                }
            }
            Scene::Main => self.step_main(a_edge, b_edge, tap_now),
            Scene::OverSeq => self.step_seq(),
            Scene::Over | Scene::Clear => {
                if start_pressed {
                    self.reset_game();
                }
            }
        }
    }

    fn update_player(&mut self, a_edge: bool, b_edge: bool, tap_now: bool) {
        self.resolve_overlap();
        // プレイヤー状態を一時的に取り出して更新する（借用回避のため手続き的に書く）
        if self.player.coyote > 0 {
            self.player.coyote -= 1;
        }
        if self.player.buf > 0 {
            self.player.buf -= 1;
        }
        let dx_in: i32 = (self.input.right as i32) - (self.input.left as i32);
        let dy_in: i32 = (self.input.down as i32) - (self.input.up as i32);
        let mut dash_shake = false;

        if self.player.dash_time > 0 {
            self.player.vx = self.player.dash_dir.0 * 5.0;
            self.player.vy = self.player.dash_dir.1 * 5.0;
            self.player.dash_time -= 1;
            let (cx, cy) = (self.player.x + 3.0, self.player.y + 3.0);
            let dx = self.rng.range_f32(-1.0, 1.0);
            let dy = self.rng.range_f32(-1.0, 1.0);
            self.parts.push(Particle { x: cx, y: cy, dx, dy, col: C_CYAN, life: 10 });
            if self.player.dash_time == 0 {
                self.player.vx *= 0.5;
                self.player.vy *= 0.5;
            }
        } else {
            if dx_in != 0 {
                let target = dx_in as f32 * 2.5;
                self.player.vx += (target - self.player.vx) * 0.2;
                self.player.facing = dx_in;
            } else {
                self.player.vx *= 0.7;
            }
            if self.player.on_wall != 0 && self.player.vy > 0.0 {
                self.player.vy = (self.player.vy + 0.1).clamp(0.0, 0.8);
            } else {
                let grav = if self.input.a && self.player.vy < 0.0 { 0.3 } else { 0.5 };
                self.player.vy += grav;
            }
            let up_now = self.input.up;
            if a_edge || tap_now || (up_now && !self.prev_up) {
                self.player.buf = 4;
            }
            self.prev_up = up_now;
            if self.player.buf > 0 {
                if self.player.coyote > 0 {
                    self.player.vy = -4.5;
                    self.player.sx = 0.6;
                    self.player.sy = 1.4;
                    self.player.coyote = 0;
                    self.player.buf = 0;
                    self.audio_event = 1;
                } else if self.player.on_wall != 0 {
                    self.player.vy = -4.2;
                    self.player.vx = -(self.player.on_wall as f32) * 3.5;
                    self.player.sx = 0.6;
                    self.player.sy = 1.4;
                    self.player.buf = 0;
                    self.audio_event = 1;
                }
            }
            if b_edge && self.player.can_dash {
                let mut idx = dx_in;
                let idy = dy_in;
                if idx == 0 && idy == 0 {
                    idx = self.player.facing;
                }
                let mag = ((idx * idx + idy * idy) as f32).sqrt();
                self.player.dash_dir = (idx as f32 / mag, idy as f32 / mag);
                self.player.dash_time = 6;
                self.player.can_dash = false;
                self.player.vy = 0.0;
                dash_shake = true;
                self.audio_event = 2;
            }
        }
        if dash_shake {
            self.shake = 4.0;
        }

        // 軸分離で移動
        let old_y = self.player.y;
        let steps_x = ((self.player.vx.abs() / 0.5) as i32 + 1).max(1);
        let step_x = self.player.vx / steps_x as f32;
        for _ in 0..steps_x {
            let nx = self.player.x + step_x;
            if !self.is_wall(nx, self.player.y) {
                self.player.x = nx;
            } else {
                self.player.vx = 0.0;
                break;
            }
        }
        let steps_y = ((self.player.vy.abs() / 0.5) as i32 + 1).max(1);
        let step_y = self.player.vy / steps_y as f32;
        for _ in 0..steps_y {
            let ny = self.player.y + step_y;
            if !self.is_wall(self.player.x, ny) {
                self.player.y = ny;
            } else {
                self.player.vy = 0.0;
                break;
            }
        }
        self.player.on_ground = self.is_wall(self.player.x, self.player.y + 1.0);
        if self.player.on_ground {
            self.player.coyote = 5;
            self.player.can_dash = true;
            if old_y < self.player.y {
                self.player.sx = 1.4;
                self.player.sy = 0.6;
            }
        }
        if self.is_wall(self.player.x + 1.0, self.player.y) {
            self.player.on_wall = 1;
        } else if self.is_wall(self.player.x - 1.0, self.player.y) {
            self.player.on_wall = -1;
        } else {
            self.player.on_wall = 0;
        }
        self.player.sx += (1.0 - self.player.sx) * 0.2;
        self.player.sy += (1.0 - self.player.sy) * 0.2;
        let ctx = ((self.player.x + 3.0) / TS).floor() as i32;
        let cty = ((self.player.y + 3.0) / TS).floor() as i32;
        if (0..16).contains(&ctx) && (0..16).contains(&cty) && self.tm[(cty * 16 + ctx) as usize] == 2 {
            self.player.dead = true;
        }
    }

    fn update_boss(&mut self) {
        let (px, py) = (self.player.x + 3.0, self.player.y + 3.0);
        let dx = px - (self.boss.x + self.boss.w / 2.0);
        let dy = py - (self.boss.y + self.boss.h / 2.0);
        let dist = (dx * dx + dy * dy).sqrt();
        if dist > 0.0 {
            self.boss.x += dx / dist * self.boss.speed;
            self.boss.y += dy / dist * self.boss.speed;
        }
        let tx0 = (self.boss.x / TS).floor() as i32;
        let ty0 = (self.boss.y / TS).floor() as i32;
        let tx1 = ((self.boss.x + self.boss.w) / TS).floor() as i32;
        let ty1 = ((self.boss.y + self.boss.h) / TS).floor() as i32;
        for ty in ty0..=ty1 {
            for tx in tx0..=tx1 {
                if (0..16).contains(&tx) && (0..16).contains(&ty) && self.tm[(ty * 16 + tx) as usize] == 1 {
                    self.tm[(ty * 16 + tx) as usize] = 0;
                }
            }
        }
    }

    fn step_main(&mut self, a_edge: bool, b_edge: bool, tap_now: bool) {
        self.update_player(a_edge, b_edge, tap_now);
        if self.player.dead {
            self.shake = 10.0;
            self.audio_event = 4;
            let (bx, by) = (self.player.x + 4.0, self.player.y + 4.0);
            self.burst(bx, by, C_CYAN, 15, 3.0);
            self.player.x = 64.0;
            self.player.y = 64.0;
            self.player.vx = 0.0;
            self.player.vy = 0.0;
            self.player.dead = false;
        }
        if self.boss_mode && self.room_x == 0 && self.room_y == 0 {
            if (self.player.x - 64.0).abs() < 8.0 && (self.player.y - 64.0).abs() < 8.0 {
                self.scene = Scene::Clear;
                return;
            }
        }
        if self.boss_mode {
            if self.countdown > 0 {
                self.countdown -= 1;
                if self.countdown == 0 {
                    self.boss.x = self.player.x - 64.0;
                    self.boss.y = self.player.y - 64.0;
                    self.boss.spawned = true;
                }
            } else if self.boss.spawned {
                self.update_boss();
                let p = &self.player;
                let b = &self.boss;
                if p.x < b.x + b.w && p.x + p.w > b.x && p.y < b.y + b.h && p.y + p.h > b.y {
                    self.player.dead = true;
                    self.shake = 15.0;
                    self.scene = Scene::OverSeq;
                    self.death_timer = 150;
                    self.audio_event = 4;
                    let (bx, by) = (self.player.x + 2.0, self.player.y + 2.0);
                    self.burst(bx, by, C_WHITE, 40, 4.0);
                    return;
                }
            }
        }
        // 部屋遷移
        let m = 4.0;
        let mut changed = false;
        if self.player.x < -m {
            self.room_x -= 1;
            self.player.x = VW - self.player.w - 12.0;
            self.boss.x += VW;
            changed = true;
        } else if self.player.x > VW - self.player.w + m {
            self.room_x += 1;
            self.player.x = 12.0;
            self.boss.x -= VW;
            changed = true;
        }
        if self.player.y < -m {
            self.room_y -= 1;
            self.player.y = VH - self.player.h - 12.0;
            self.boss.y += VH;
            changed = true;
        } else if self.player.y > VH + m {
            self.room_y += 1;
            self.player.y = 12.0;
            self.boss.y -= VH;
            changed = true;
        }
        if changed {
            self.enter_room();
        }
        if self.shake > 0.0 {
            self.shake -= 1.0;
        }
        // オーブ
        for i in 0..self.orbs.len() {
            if !self.orbs[i].active {
                self.orbs[i].timer += 1;
                if self.orbs[i].timer > 90 {
                    self.orbs[i].active = true;
                    self.orbs[i].timer = 0;
                }
            } else {
                let o = self.orbs[i];
                let dx = (self.player.x + 3.0) - (o.x as f32 + 4.0);
                let dy = (self.player.y + 3.0) - (o.y as f32 + 4.0);
                if (dx * dx + dy * dy).sqrt() < 10.0 && !self.player.can_dash {
                    self.player.can_dash = true;
                    self.orbs[i].active = false;
                    self.orbs[i].timer = 0;
                    self.collected += 1;
                    self.collected_rooms.insert((self.room_x, self.room_y));
                    self.audio_event = 3;
                    self.burst(o.x as f32 + 4.0, o.y as f32 + 4.0, C_YEL, 5, 2.0);
                    if self.shake < 2.0 {
                        self.shake = 2.0;
                    }
                    if self.collected >= 3 && !self.boss_mode {
                        self.boss_mode = true;
                        self.countdown = 300;
                    }
                }
            }
        }
        self.update_particles();
    }

    fn step_seq(&mut self) {
        self.death_timer -= 1;
        self.shake = 5.0;
        if self.boss.spawned {
            self.update_boss();
        }
        let cols = [C_WHITE, C_CYAN, C_BURST2];
        for _ in 0..3 {
            let x = self.player.x + 2.0 + self.rng.range_f32(0.0, 4.0);
            let y = self.player.y + 2.0 + self.rng.range_f32(0.0, 4.0);
            let ci = (self.rng.next_f64() * 3.0) as usize % 3;
            let dx = self.rng.range_f32(-2.0, 2.0);
            let dy = self.rng.range_f32(-2.0, 2.0);
            self.parts.push(Particle { x, y, dx, dy, col: cols[ci], life: 20 });
        }
        self.update_particles();
        if self.death_timer <= 0 {
            self.scene = Scene::Over;
            self.shake = 0.0;
        }
    }

    fn update_particles(&mut self) {
        for p in self.parts.iter_mut() {
            p.x += p.dx;
            p.y += p.dy;
            p.life -= 1;
        }
        self.parts.retain(|p| p.life > 0);
    }

    // ── 描画 ────────────────────────────────────────────────────────────────

    fn draw(&mut self) {
        self.draw.clear();
        match self.scene {
            Scene::Title => self.draw_title(),
            _ => {
                self.draw_scene();
                self.draw_hud();
            }
        }
    }

    fn draw_title(&mut self) {
        let d = &mut self.draw;
        d.offset = (0.0, 0.0);
        d.rect(0.0, 0.0, VW, VH, C_BG);
        // 月
        d.circle_fill(105.0, 15.0, 6.0, C_WHITE);
        d.circle_fill(103.0, 13.0, 5.0, C_BG);
        // キューブ（上下にゆっくり揺れる）
        let bob = (self.frame as f32 * 6.0).to_radians().sin() * 1.5;
        d.rect(58.0, 44.0 + bob, 12.0, 12.0, C_WHITE);
        d.rect(66.0, 46.0 + bob, 1.0, 1.0, C_BLACK);
    }

    fn draw_scene(&mut self) {
        let (ox, oy) = if self.shake > 0.0 {
            (
                self.rng.range_f32(-self.shake, self.shake),
                self.rng.range_f32(-self.shake, self.shake),
            )
        } else {
            (0.0, 0.0)
        };
        self.draw.offset = (ox, oy);
        self.draw_bg();
        self.draw_tiles();
        if self.boss_mode && self.room_x == 0 && self.room_y == 0 {
            let r = 8.0 + (self.frame as f32 * 10.0).to_radians().sin() * 2.0;
            self.draw.circle_fill(64.0, 64.0, r, C_PORTAL);
            self.draw.circle_fill(64.0, 64.0, 4.0, C_WHITE);
            self.draw.text_centered("EXIT", 64.0, 48.0, 1.0, C_WHITE);
        }
        // オーブ
        for i in 0..self.orbs.len() {
            let o = self.orbs[i];
            if o.active {
                let bob = (((self.frame >> 2) * 20) as f32).to_radians().sin() * 2.0;
                let (cx, cy) = (o.x as f32 + 4.0, o.y as f32 + 4.0 + bob);
                self.draw.circle_fill(cx, cy, 3.0, C_YEL);
                self.draw.circle_outline(cx, cy, 3.0, 1.0, C_WHITE);
            } else {
                self.draw
                    .circle_outline(o.x as f32 + 4.0, o.y as f32 + 4.0, 2.0, 1.0, C_WALLD);
            }
        }
        // パーティクル
        for i in 0..self.parts.len() {
            let p = self.parts[i];
            self.draw.pixel(p.x, p.y, p.col);
        }
        if !self.player.dead {
            self.draw_player();
        }
        if self.boss.spawned && (self.boss_mode || self.scene == Scene::OverSeq) {
            let b = &self.boss;
            self.draw.rect(b.x, b.y, b.w, b.h, C_RED);
            self.draw.rect_outline(b.x, b.y, b.w, b.h, 1.0, C_WHITE);
            self.draw.rect(b.x + 4.0, b.y + 6.0, 4.0, 4.0, C_WHITE);
            self.draw.rect(b.x + 16.0, b.y + 6.0, 4.0, 4.0, C_WHITE);
        }
        self.draw.offset = (0.0, 0.0);
    }

    fn draw_bg(&mut self) {
        let d = &mut self.draw;
        d.rect(0.0, 0.0, VW, VH, C_BG);
        d.circle_fill(105.0, 15.0, 6.0, C_WHITE);
        d.circle_fill(103.0, 13.0, 5.0, C_BG);
        let cols = [C_MTN1, C_MTN2, C_MTN1];
        let hs = [15i32, 28, 45];
        for layer in 0..3 {
            let mut rng = JsRng::new(hash_seed(&format!(
                "{}_{}_mtn_{}",
                self.room_x, self.room_y, layer
            )));
            let h_max = hs[layer];
            d_rect_layer(d, &mut rng, h_max, cols[layer]);
        }
    }

    fn draw_tiles(&mut self) {
        for ty in 0..16 {
            for tx in 0..16 {
                let v = self.tm[ty * 16 + tx];
                let (x, y) = (tx as f32 * TS, ty as f32 * TS);
                match v {
                    1 => {
                        self.draw.rect(x, y, TS, TS, C_WALL);
                        self.draw.pixel(x + 2.0, y + 2.0, C_WALLD);
                    }
                    2 => {
                        let d = &mut self.draw;
                        d.tri((x, y + 7.0), (x + 2.0, y + 2.0), (x + 4.0, y + 7.0), C_CYAN);
                        d.tri((x + 3.0, y + 7.0), (x + 5.0, y + 1.0), (x + 7.0, y + 7.0), C_GRAYC);
                        d.tri((x + 5.0, y + 7.0), (x + 6.0, y + 3.0), (x + 7.0, y + 7.0), C_CYAN);
                        d.line(x + 2.0, y + 3.0, x + 2.0, y + 7.0, 1.0, C_WHITE);
                        d.line(x + 5.0, y + 2.0, x + 5.0, y + 7.0, 1.0, C_WHITE);
                    }
                    _ => {}
                }
            }
        }
    }

    fn draw_player(&mut self) {
        let p = &self.player;
        let sw = 8.0 * p.sx;
        let mut sh = 8.0 * p.sy;
        if p.on_ground && p.vx.abs() < 0.2 {
            sh += (self.frame as f32 * 15.0).to_radians().sin() * 0.4;
        }
        let col = if p.can_dash { C_WHITE } else { C_PINK };
        self.draw.rect(p.x + (p.w - sw) / 2.0, p.y + (p.h - sh), sw, sh, col);
        let eye_x = if p.facing > 0 { 4.0 } else { 0.0 };
        self.draw.rect(p.x + eye_x, p.y + 1.0, 1.0, 1.0, C_BLACK);
    }

    fn draw_hud(&mut self) {
        let d = &mut self.draw;
        let col = if self.collected < 3 { C_WHITE } else { C_YEL };
        d.text(&format!("ORBS: {}/3", self.collected), 4.0, 3.0, 1.0, col);
        d.text(
            &format!("ROOM: {},{}", self.room_x, self.room_y),
            4.0,
            12.0,
            1.0,
            C_ROOM,
        );
        if self.boss_mode {
            if self.countdown > 0 {
                let sec = self.countdown / 30;
                d.text(&format!("BOSS IN: {}S", sec), 76.0, 114.0, 1.0, C_RED);
                d.rect(
                    80.0,
                    124.0,
                    40.0 * (self.countdown as f32 / 300.0),
                    2.0,
                    C_RED,
                );
            } else {
                d.text_centered("WARNING: BOSS ACTIVE!", 64.0, 118.0, 1.0, C_RED);
            }
        }
    }
}

/// 山のレイヤー（32px ごとに滑らかに高さを変える）を描く
fn d_rect_layer(d: &mut Draw2d, rng: &mut JsRng, h_max: i32, col: Rgba) {
    let mut last_h = rng.ri(h_max - 10, h_max + 10);
    let mut x = 0;
    while x < 160 {
        let next_h = rng.ri(h_max - 10, h_max + 10);
        for i in 0..32 {
            let t = i as f32 / 32.0;
            let st = t * t * (3.0 - 2.0 * t);
            let h = (last_h as f32 * (1.0 - st) + next_h as f32 * st).round();
            d.rect((x + i) as f32, VH - h, 1.0, h + 10.0, col);
        }
        last_h = next_h;
        x += 32;
    }
}
