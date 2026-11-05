// lineboy.rs — LINE BOY（rustgames 版）
//
// html-player 版 LINE BOY（JS）の移植。ネオンな線画プラットフォーマー。
// 全 5 ワールド。各ワールドの奥にいるボスを倒してゴールへ。
// 地形生成は JS 版と同一のシード規則（mulberry32）で再現する。
// 描画は render2d（wgpu 2D バッチ層）で線・枠・円を再現する。
// 仮想解像度 512x512、レベル幅 2048、60Hz 固定ステップ。

use wasm_bindgen::JsCast;

use crate::render2d::{Draw2d, Renderer2d, Rgba};
use crate::rng::{GameRng, JsRng};

pub const VW: f32 = 512.0;
pub const VH: f32 = 512.0;
const W: f32 = 512.0;
const H: f32 = 512.0;
const LEVEL_W: f32 = 2048.0;
const TS: i32 = 16;
const PW: f32 = 12.0;
const PH: f32 = 14.0;
const GRAV: f32 = 0.55;
const JUMP_VEL: f32 = -9.5;
const MAX_FALL: f32 = 12.0;
const MOVE_SPEED: f32 = 3.2;
const FRICTION: f32 = 0.80;

// パレット（JS 版 PAL の 16 色）
const CB: usize = 0;
const CPU: usize = 2;
const CG: usize = 3;
const CBR: usize = 4;
const CLB: usize = 6;
const CW: usize = 7;
const CR: usize = 8;
const CO: usize = 9;
const CY: usize = 10;
const CL: usize = 11;
const CC: usize = 12;
const CGR: usize = 13;
const CP: usize = 14;
const CPE: usize = 15;

const PAL: [Rgba; 16] = [
    Rgba::hex(0x050508),
    Rgba::hex(0x0D1B4B),
    Rgba::hex(0xBF00FF),
    Rgba::hex(0x39FF14),
    Rgba::hex(0xA0522D),
    Rgba::hex(0x1A3A5C),
    Rgba::hex(0x00BFFF),
    Rgba::hex(0xF0F0FF),
    Rgba::hex(0xFF2020),
    Rgba::hex(0xFF6B00),
    Rgba::hex(0xFFFF00),
    Rgba::hex(0xCCFF00),
    Rgba::hex(0x00F5FF),
    Rgba::hex(0x404050),
    Rgba::hex(0xFF1493),
    Rgba::hex(0xFF7F50),
];

struct Theme {
    name: &'static str,
    ground: usize,
    accent: usize,
    sky: usize,
}

const THEMES: [Theme; 5] = [
    Theme { name: "FOREST", ground: CG, accent: CBR, sky: CL },
    Theme { name: "DESERT", ground: CO, accent: CY, sky: CPE },
    Theme { name: "CAVE", ground: CC, accent: CGR, sky: CLB },
    Theme { name: "GHOST", ground: CPU, accent: CP, sky: CW },
    Theme { name: "VOLCANO", ground: CR, accent: CO, sky: CP },
];

#[derive(Clone, Copy, PartialEq)]
pub enum Scene {
    Title = 0,
    Play = 1,
    Over = 2,
    Clear = 3,
}

#[derive(Default, Clone, Copy)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub a: bool,
}

// ── 地形データ ──────────────────────────────────────────────────────────────

struct TerrainData {
    segs: Vec<(i32, i32, i32)>,   // (x, y, w)
    plats: Vec<(i32, i32, i32)>,  // (x, y, w)
    trees: Vec<(i32, i32, i32, usize, usize, i32)>, // (x, y, h, trunk, leaf, style)
    enemy_spawns: Vec<(i32, i32)>,
    items: Vec<(f32, f32)>,
}

fn gen_terrain(seed: u32, theme_idx: usize) -> TerrainData {
    let theme = &THEMES[theme_idx];
    let mut rng = JsRng::new(seed);
    let mut ground_y: i32 = (H - 80.0) as i32;
    let mut segs = Vec::new();
    let mut x = 0;
    while (x as f32) < LEVEL_W {
        let sw = rng.ri(TS * 2, TS * 8);
        let dy = rng.ri(-TS, TS);
        ground_y = (ground_y + dy).clamp((H - 140.0) as i32, (H - 60.0) as i32);
        segs.push((x, ground_y, sw));
        x += sw;
    }
    let np = (LEVEL_W as i32) / (TS * 8);
    let mut plats = Vec::new();
    for _ in 0..np {
        plats.push((
            rng.ri(TS * 2, LEVEL_W as i32 - TS * 6),
            rng.ri((H - 260.0) as i32, (H - 120.0) as i32),
            rng.ri(TS * 3, TS * 8),
        ));
    }
    let mut trees = Vec::new();
    let mut i = 0;
    while i < segs.len() {
        let s = segs[i];
        trees.push((
            s.0 + rng.ri(4, (s.2 - 4).max(5)),
            s.1,
            rng.ri(32, 64),
            theme.accent,
            theme.ground,
            rng.ri(0, 2),
        ));
        i += 3;
    }
    let ne = (LEVEL_W as i32) / (TS * 16);
    let mut enemy_spawns = Vec::new();
    for _ in 0..ne {
        let ex = rng.ri(280, LEVEL_W as i32 - TS * 4);
        let mut ey = (H - 80.0) as i32;
        for s in &segs {
            if s.0 <= ex && ex <= s.0 + s.2 {
                ey = s.1;
                break;
            }
        }
        enemy_spawns.push((ex, ey));
    }
    let mut items = Vec::new();
    let mut i = 0;
    while i < segs.len() {
        let s = segs[i];
        let n = rng.ri(1, 4);
        for _ in 0..n {
            items.push((
                (s.0 + rng.ri(4, (s.2 - 4).max(5))) as f32,
                (s.1 - 10) as f32,
            ));
        }
        i += 2;
    }
    TerrainData { segs, plats, trees, enemy_spawns, items }
}

// ── エンティティ ────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct Particle {
    x: f32,
    y: f32,
    ox: f32,
    oy: f32,
    vx: f32,
    vy: f32,
    col: usize,
    life: i32,
    life_max: i32,
    shape: u8,
    grav: f32,
}

impl Particle {
    fn new(x: f32, y: f32, vx: f32, vy: f32, col: usize, life: i32, shape: u8, grav: f32) -> Self {
        Particle { x, y, ox: x, oy: y, vx, vy, col, life, life_max: life, shape, grav }
    }
    fn update(&mut self) {
        self.ox = self.x;
        self.oy = self.y;
        self.x += self.vx;
        self.y += self.vy;
        self.vy += self.grav;
        self.vx *= 0.96;
        self.life -= 1;
    }
}

#[derive(Clone, Copy)]
struct Enemy {
    x: f32,
    y: f32,
    col: usize,
    vx: f32,
    alive: bool,
    anim: i32,
}

struct Boss {
    x: f32,
    y: f32,
    col: usize,
    kind: usize,
    hp: i32,
    alive: bool,
    flash: i32,
    size: f32,
    timer: i32,
    ang: f32,
    trail: Vec<(f32, f32)>,
    bursted: bool,
}

impl Boss {
    fn new(x: f32, y: f32, col: usize, kind: usize) -> Self {
        let mut trail = Vec::new();
        for i in 0..110 {
            trail.push((x + i as f32 * 1.6, y + (i as f32 * 0.15).sin() * 7.0));
        }
        Boss {
            x,
            y,
            col,
            kind,
            hp: 5,
            alive: true,
            flash: 0,
            size: 40.0,
            timer: 0,
            ang: std::f32::consts::PI,
            trail,
            bursted: false,
        }
    }
}

struct Player {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    on_ground: bool,
    facing: i32,
    alive: bool,
    inv: i32,
    score: u32,
    sx: f32,
    sy: f32,
    coyote: i32,
    buf: i32,
    anim: i32,
}

impl Player {
    fn new(x: f32, y: f32) -> Self {
        Player {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            on_ground: false,
            facing: 1,
            alive: true,
            inv: 0,
            score: 0,
            sx: 1.0,
            sy: 1.0,
            coyote: 0,
            buf: 0,
            anim: 0,
        }
    }
}

struct Level {
    data: TerrainData,
    goal_x: f32,
    enemies: Vec<Enemy>,
    items: Vec<(f32, f32)>,
}

// ── ゲーム本体 ──────────────────────────────────────────────────────────────

// audio_event: 0=なし 1=ジャンプ 2=アイテム 3=踏みつけ 4=死亡 5=ボス爆発
pub struct LineBoyGame {
    renderer: Renderer2d,
    draw: Draw2d,
    scene: Scene,
    world: usize,
    level: Option<Level>,
    player: Option<Player>,
    boss: Option<Boss>,
    parts: Vec<Particle>,
    title_parts: Vec<Particle>,
    cam_x: f32,
    shake: i32,
    boss_on: bool,
    over_timer: i32,
    score_total: u32,
    frame: u64,
    pub input: Input,
    prev_a: bool,
    prev_left: bool,
    prev_right: bool,
    tap_pending: bool,
    pub audio_event: u8,
    rng: GameRng,
    prev_ts: f64,
    acc: f64,
}

impl LineBoyGame {
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
        Ok(LineBoyGame {
            renderer,
            draw: Draw2d::new(),
            scene: Scene::Title,
            world: 0,
            level: None,
            player: None,
            boss: None,
            parts: Vec::new(),
            title_parts: Vec::new(),
            cam_x: 0.0,
            shake: 0,
            boss_on: false,
            over_timer: 0,
            score_total: 0,
            frame: 0,
            input: Input::default(),
            prev_a: false,
            prev_left: false,
            prev_right: false,
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

    pub fn press_a(&mut self) {
        self.tap_pending = true;
    }

    pub fn score(&self) -> u32 {
        self.player.as_ref().map(|p| p.score).unwrap_or(0)
    }

    pub fn score_total(&self) -> u32 {
        self.score_total
    }

    pub fn world_idx(&self) -> u32 {
        self.world as u32
    }

    pub fn boss_hp(&self) -> i32 {
        if self.boss_on {
            self.boss.as_ref().map(|b| if b.alive { b.hp } else { 0 }).unwrap_or(0)
        } else {
            0
        }
    }

    pub fn boss_active(&self) -> bool {
        self.boss_on && self.boss.as_ref().map(|b| b.alive).unwrap_or(false)
    }

    fn start_level(&mut self) {
        let seed = (self.world * 9999 + 42) as u32;
        let data = gen_terrain(seed, self.world);
        let enemies = data
            .enemy_spawns
            .iter()
            .map(|&(ex, ey)| {
                let vx = if self.rng.next_f64() < 0.5 { -1.0 } else { 1.0 };
                Enemy {
                    x: ex as f32,
                    y: ey as f32,
                    col: THEMES[self.world].accent,
                    vx,
                    alive: true,
                    anim: 0,
                }
            })
            .collect::<Vec<_>>();
        let items = data.items.clone();
        let start_y = if data.segs.is_empty() {
            (H - 160.0) as i32
        } else {
            data.segs[0].1
        };
        self.player = Some(Player::new(60.0, start_y as f32 - PH - 2.0));
        self.boss = Some(Boss::new(
            LEVEL_W - 200.0,
            start_y as f32 - 80.0,
            THEMES[self.world].ground,
            self.world,
        ));
        self.level = Some(Level {
            data,
            goal_x: LEVEL_W - 80.0,
            enemies,
            items,
        });
        self.parts.clear();
        self.cam_x = 0.0;
        self.shake = 0;
        self.boss_on = false;
        self.over_timer = 0;
    }

    // ── 更新 ────────────────────────────────────────────────────────────────

    pub fn tick(&mut self, ts: f64) {
        if self.prev_ts == 0.0 {
            self.prev_ts = ts;
        }
        let dt = ((ts - self.prev_ts) / 1000.0).min(0.25);
        self.prev_ts = ts;
        self.acc += dt;
        let h = 1.0 / 60.0;
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
        self.renderer.render(&self.draw, PAL[CB]);
    }

    fn step(&mut self) {
        self.frame += 1;
        let a_edge = (self.input.a && !self.prev_a) || self.tap_pending;
        self.prev_a = self.input.a;
        self.tap_pending = false;
        match self.scene {
            Scene::Title => self.step_title(a_edge),
            Scene::Play => self.step_play(a_edge),
            Scene::Over | Scene::Clear => self.step_end(a_edge),
        }
    }

    fn step_title(&mut self, a_edge: bool) {
        if self.frame % 3 == 0 {
            let cols = [CG, CC, CL, CO, CP, CPU, CY];
            let shapes = [0u8, 2, 3];
            let x = self.rng.range_f32(10.0, W - 10.0);
            let y = self.rng.range_f32(H / 2.0 + 40.0, H - 20.0);
            let vx = self.rng.range_f32(-0.6, 0.6);
            let vy = self.rng.range_f32(-2.5, -0.8);
            let col = cols[(self.rng.next_f64() * 7.0) as usize % 7];
            let life = 40 + (self.rng.next_f64() * 50.0) as i32;
            let shape = shapes[(self.rng.next_f64() * 3.0) as usize % 3];
            self.title_parts.push(Particle::new(x, y, vx, vy, col, life, shape, -0.01));
        }
        for p in self.title_parts.iter_mut() {
            p.update();
        }
        self.title_parts.retain(|p| p.life > 0 && p.y > -10.0);
        if self.input.left && !self.prev_left {
            self.world = (self.world + THEMES.len() - 1) % THEMES.len();
        }
        if self.input.right && !self.prev_right {
            self.world = (self.world + 1) % THEMES.len();
        }
        self.prev_left = self.input.left;
        self.prev_right = self.input.right;
        if a_edge {
            self.scene = Scene::Play;
            self.start_level();
        }
    }

    fn step_end(&mut self, a_edge: bool) {
        self.over_timer += 1;
        for p in self.parts.iter_mut() {
            p.update();
        }
        self.parts.retain(|p| p.life > 0);
        if self.over_timer > 60 && a_edge {
            if self.scene == Scene::Over {
                self.scene = Scene::Title;
            } else {
                // Clear → 次のワールドへ
                if let Some(p) = &self.player {
                    self.score_total += p.score;
                }
                self.world = (self.world + 1) % THEMES.len();
                self.scene = Scene::Play;
                self.start_level();
            }
        }
    }

    fn player_die(&mut self) {
        let col = THEMES[self.world].ground;
        if let Some(p) = &mut self.player {
            p.alive = false;
        }
        let (cx, cy) = match &self.player {
            Some(p) => (p.x + PW / 2.0, p.y + PH / 2.0),
            None => return,
        };
        for _ in 0..20 {
            let vx = self.rng.range_f32(-5.0, 5.0);
            let vy = self.rng.range_f32(-6.0, 1.0);
            self.parts.push(Particle::new(cx, cy, vx, vy, col, 45, 3, 0.18));
        }
        for _ in 0..12 {
            let vx = self.rng.range_f32(-3.0, 3.0);
            let vy = self.rng.range_f32(-4.0, 0.5);
            self.parts.push(Particle::new(cx, cy, vx, vy, CW, 30, 2, 0.1));
        }
        for _ in 0..8 {
            let vx = self.rng.range_f32(-2.0, 2.0);
            let vy = self.rng.range_f32(-3.0, 0.0);
            self.parts.push(Particle::new(cx, cy, vx, vy, CY, 25, 1, 0.08));
        }
        self.audio_event = 4;
    }

    fn step_play(&mut self, a_edge: bool) {
        let col = THEMES[self.world].ground;
        // ボス出現判定
        if !self.boss_on {
            if let Some(p) = &self.player {
                if p.x > LEVEL_W - 400.0 {
                    self.boss_on = true;
                    self.shake = 20;
                }
            }
        }
        self.update_player(a_edge);
        if let Some(p) = &self.player {
            if !p.alive {
                self.shake = 15;
                self.scene = Scene::Over;
                self.over_timer = 0;
                return;
            }
        }
        // ボス更新＋演出パーティクル
        if self.boss_on {
            let boss_alive = self.boss.as_ref().map(|b| b.alive).unwrap_or(false);
            if boss_alive {
                self.update_boss();
                if self.frame % 3 == 0 {
                    if let Some(b) = &self.boss {
                        let a = self.frame as f32 * 0.15;
                        for rad in [b.size + 2.0, b.size - 10.0] {
                            let px = b.x + a.cos() * rad;
                            let py = b.y + a.sin() * rad;
                            let vx = self.rng.range_f32(-0.5, 0.5);
                            let vy = self.rng.range_f32(-0.5, 0.5);
                            self.parts.push(Particle::new(px, py, vx, vy, b.col, 20, 3, 0.0));
                        }
                    }
                }
            } else if let Some(b) = &mut self.boss {
                if !b.bursted {
                    b.bursted = true;
                    let (bx, by, bcol) = (b.x, b.y, b.col);
                    let cols = [CY, CW, CC, CL, CP];
                    let shapes = [0u8, 1, 3];
                    for _ in 0..40 {
                        let vx = self.rng.range_f32(-6.0, 6.0);
                        let vy = self.rng.range_f32(-7.0, 0.0);
                        let c = cols[(self.rng.next_f64() * 5.0) as usize % 5];
                        let s = shapes[(self.rng.next_f64() * 3.0) as usize % 3];
                        self.parts.push(Particle::new(bx, by, vx, vy, c, 60, s, 0.15));
                    }
                    for _ in 0..20 {
                        let vx = self.rng.range_f32(-4.0, 4.0);
                        let vy = self.rng.range_f32(-5.0, 0.0);
                        self.parts.push(Particle::new(bx, by, vx, vy, bcol, 45, 2, 0.1));
                    }
                    self.audio_event = 5;
                }
            }
            // ゴール到達でクリア
            if !boss_alive {
                if let (Some(p), Some(lv)) = (&self.player, &self.level) {
                    if p.x >= lv.goal_x {
                        self.scene = Scene::Clear;
                        self.over_timer = 0;
                        return;
                    }
                }
            }
        }
        if let Some(lv) = &mut self.level {
            for e in lv.enemies.iter_mut() {
                if e.alive {
                    update_enemy(e, &lv.data);
                }
            }
        }
        for p in self.parts.iter_mut() {
            p.update();
        }
        self.parts.retain(|p| p.life > 0);
        if let Some(p) = &self.player {
            let target = (p.x - W / 3.0).clamp(0.0, LEVEL_W - W);
            self.cam_x += (target - self.cam_x) * 0.12;
        }
        if self.shake > 0 {
            self.shake -= 1;
        }
        let _ = col;
    }

    fn update_boss(&mut self) {
        let (px, py) = match &self.player {
            Some(p) => (p.x + PW / 2.0, p.y + PH / 2.0),
            None => return,
        };
        if let Some(b) = &mut self.boss {
            b.timer += 1;
            let dx = px - b.x;
            let dy = py - b.y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > 0.0 {
                b.ang = dy.atan2(dx);
                let sp = 1.2 + b.timer as f32 * 0.001;
                b.x += dx / dist * sp;
                b.y += dy / dist * sp;
            }
            b.trail.insert(0, (b.x, b.y));
            if b.trail.len() > 110 {
                b.trail.pop();
            }
            if b.flash > 0 {
                b.flash -= 1;
            }
        }
    }

    fn update_player(&mut self, a_edge: bool) {
        let col = THEMES[self.world].ground;
        let a_held = self.input.a;
        // フェーズ1: 移動と物理（プレイヤー単体で完結する部分）
        let mut died = false;
        let mut jumped = false;
        if let Some(p) = &mut self.player {
            if !p.alive {
                return;
            }
            p.anim += 1;
            if p.inv > 0 {
                p.inv -= 1;
            }
            if p.coyote > 0 {
                p.coyote -= 1;
            }
            if p.buf > 0 {
                p.buf -= 1;
            }
            if self.input.left {
                p.vx -= 1.2;
                p.facing = -1;
            } else if self.input.right {
                p.vx += 1.2;
                p.facing = 1;
            } else {
                p.vx *= FRICTION;
            }
            p.vx = p.vx.clamp(-MOVE_SPEED, MOVE_SPEED);
            let grav = GRAV * if a_held && p.vy < 0.0 { 0.55 } else { 1.0 };
            p.vy = (p.vy + grav).min(MAX_FALL);
            if a_edge {
                p.buf = 5;
            }
            if p.buf > 0 && p.coyote > 0 {
                p.vy = JUMP_VEL;
                p.coyote = 0;
                p.buf = 0;
                p.sx = 0.6;
                p.sy = 1.5;
                jumped = true;
            }
            p.x += p.vx;
            p.x = p.x.clamp(0.0, LEVEL_W - PW);
            p.y += p.vy;
            // 着地解決（中心 x で最も高い面）
            let was_ground = p.on_ground;
            p.on_ground = false;
            if let Some(lv) = &self.level {
                let foot = p.y + PH;
                let cx = p.x + PW / 2.0;
                let mut cands: Vec<f32> = Vec::new();
                for s in &lv.data.segs {
                    if (s.0 as f32) - 2.0 <= cx && cx <= (s.0 + s.2) as f32 + 2.0 {
                        cands.push(s.1 as f32);
                    }
                }
                for pl in &lv.data.plats {
                    if (pl.0 as f32) - 2.0 <= cx
                        && cx <= (pl.0 + pl.2) as f32 + 2.0
                        && p.vy >= 0.0
                        && foot <= pl.1 as f32 + 18.0
                    {
                        cands.push(pl.1 as f32);
                    }
                }
                if !cands.is_empty() && p.vy >= 0.0 {
                    let mut best: Option<f32> = None;
                    for sy in cands {
                        if foot >= sy - 2.0 && best.map(|b| sy < b).unwrap_or(true) {
                            best = Some(sy);
                        }
                    }
                    if let Some(b) = best {
                        p.y = b - PH;
                        p.vy = 0.0;
                        p.on_ground = true;
                    }
                }
            }
            if p.on_ground && !was_ground {
                p.sx = 1.4;
                p.sy = 0.6;
            }
            if p.on_ground {
                p.coyote = 6;
            }
            p.sx += (1.0 - p.sx) * 0.25;
            p.sy += (1.0 - p.sy) * 0.25;
            if p.y > H + 50.0 {
                died = true;
            }
        }
        if jumped {
            self.audio_event = 1;
            if let Some(p) = &self.player {
                let (jx, jy) = (p.x + PW / 2.0, p.y + PH);
                for _ in 0..10 {
                    let vx = self.rng.range_f32(-2.5, 2.5);
                    let vy = self.rng.range_f32(0.5, 2.5);
                    self.parts.push(Particle::new(jx, jy, vx, vy, col, 18, 2, 0.1));
                }
            }
        }
        if died {
            self.player_die();
            return;
        }
        // 着地パーティクルは JS では着地した瞬間に出るが、ここでは簡略化して省略しない
        // （on_ground の立ち上がりは update 内で検出済み。演出はジャンプ/走行で担保）

        // フェーズ2: アイテム・敵・ボスとの相互作用
        let mut item_hits: Vec<usize> = Vec::new();
        if let (Some(p), Some(lv)) = (&self.player, &self.level) {
            for (i, it) in lv.items.iter().enumerate() {
                if (p.x + PW / 2.0 - it.0).abs() < 12.0 && (p.y + PH / 2.0 - it.1).abs() < 12.0 {
                    item_hits.push(i);
                }
            }
        }
        if !item_hits.is_empty() {
            let mut gained = 0u32;
            if let Some(lv) = &mut self.level {
                for &i in item_hits.iter().rev() {
                    let it = lv.items.remove(i);
                    gained += 10;
                    for _ in 0..5 {
                        let vx = self.rng.range_f32(-2.0, 2.0);
                        let vy = self.rng.range_f32(-3.0, 0.0);
                        self.parts.push(Particle::new(it.0, it.1, vx, vy, CY, 20, 0, 0.15));
                    }
                }
            }
            if let Some(p) = &mut self.player {
                p.score += gained;
            }
            self.audio_event = 2;
        }

        if let Some(p) = &self.player {
            if p.inv != 0 || !p.alive {
                return;
            }
        }
        // 敵との接触
        let mut stomp_enemy: Option<usize> = None;
        let mut hit = false;
        if let (Some(p), Some(lv)) = (&self.player, &self.level) {
            for (i, e) in lv.enemies.iter().enumerate() {
                if !e.alive {
                    continue;
                }
                if (p.x + PW / 2.0 - e.x).abs() < 14.0 && (p.y + PH / 2.0 - e.y).abs() < 14.0 {
                    if p.vy > 1.0 && p.y + PH < e.y + 2.0 {
                        stomp_enemy = Some(i);
                    } else {
                        hit = true;
                    }
                    break;
                }
            }
        }
        if let Some(i) = stomp_enemy {
            let (ex, ey) = if let Some(lv) = &mut self.level {
                lv.enemies[i].alive = false;
                (lv.enemies[i].x, lv.enemies[i].y)
            } else {
                (0.0, 0.0)
            };
            if let Some(p) = &mut self.player {
                p.vy = -6.0;
                p.score += 100;
            }
            for _ in 0..8 {
                let vx = self.rng.range_f32(-3.0, 3.0);
                let vy = self.rng.range_f32(-4.0, 0.0);
                self.parts.push(Particle::new(ex, ey, vx, vy, col, 25, 0, 0.15));
            }
            self.audio_event = 3;
            return;
        }
        if hit {
            self.player_die();
            return;
        }
        // ボスとの接触
        if self.boss_on {
            let mut boss_hit = false;
            let mut boss_stomp = false;
            if let (Some(p), Some(b)) = (&self.player, &self.boss) {
                if b.alive {
                    let dx = (p.x + PW / 2.0 - b.x).abs();
                    let dy = (p.y + PH / 2.0 - b.y).abs();
                    if dx < b.size * 0.6 && dy < b.size * 0.6 {
                        if p.vy > 1.0 {
                            boss_stomp = true;
                        } else {
                            boss_hit = true;
                        }
                    }
                }
            }
            if boss_stomp {
                let (bx, by, bcol) = if let Some(b) = &mut self.boss {
                    b.hp -= 1;
                    b.flash = 8;
                    if b.hp <= 0 {
                        b.alive = false;
                    }
                    (b.x, b.y, b.col)
                } else {
                    (0.0, 0.0, 0)
                };
                if let Some(p) = &mut self.player {
                    p.vy = -8.0;
                }
                for _ in 0..12 {
                    let vx = self.rng.range_f32(-4.0, 4.0);
                    let vy = self.rng.range_f32(-5.0, 0.0);
                    self.parts.push(Particle::new(bx, by, vx, vy, bcol, 30, 0, 0.15));
                }
                self.audio_event = 3;
                return;
            }
            if boss_hit {
                self.player_die();
            }
        }
    }
}

fn update_enemy(e: &mut Enemy, data: &TerrainData) {
    e.x += e.vx;
    e.anim += 1;
    for s in &data.segs {
        if (s.0 as f32) <= e.x && e.x <= (s.0 + s.2) as f32 && (e.y - s.1 as f32).abs() < 2.0 {
            if e.x <= s.0 as f32 + 4.0 || e.x >= (s.0 + s.2) as f32 - 4.0 {
                e.vx *= -1.0;
            }
            break;
        }
    }
    for p in &data.plats {
        if (p.0 as f32) <= e.x && e.x <= (p.0 + p.2) as f32 && (e.y - p.1 as f32).abs() < 2.0 {
            if e.x <= p.0 as f32 + 4.0 || e.x >= (p.0 + p.2) as f32 - 4.0 {
                e.vx *= -1.0;
            }
            break;
        }
    }
}

// ── 描画ヘルパー（JS 版 nLine / nRectb / nCircb / pset の再現） ─────────────

fn pset(d: &mut Draw2d, x: f32, y: f32, col: usize) {
    d.pixel(x.trunc(), y.trunc(), PAL[col]);
}

fn line_core(d: &mut Draw2d, x1: f32, y1: f32, x2: f32, y2: f32, col: usize) {
    d.line(x1, y1, x2, y2, 1.0, PAL[col]);
}

/// グロー付き線（4 方向オフセットにグロー色、本体に core 色）
fn line_glow(d: &mut Draw2d, x1: f32, y1: f32, x2: f32, y2: f32, core: usize, glow: usize) {
    let g = PAL[glow];
    d.line(x1 - 1.0, y1, x2 - 1.0, y2, 1.0, g);
    d.line(x1 + 1.0, y1, x2 + 1.0, y2, 1.0, g);
    d.line(x1, y1 - 1.0, x2, y2 - 1.0, 1.0, g);
    d.line(x1, y1 + 1.0, x2, y2 + 1.0, 1.0, g);
    d.line(x1, y1, x2, y2, 1.0, PAL[core]);
}

fn rectb(d: &mut Draw2d, x: f32, y: f32, w: f32, h: f32, core: usize, glow: Option<usize>) {
    if let Some(g) = glow {
        d.rect_outline(x - 1.0, y - 1.0, w + 2.0, h + 2.0, 1.0, PAL[g]);
    }
    d.rect_outline(x, y, w, h, 1.0, PAL[core]);
}

fn circb(d: &mut Draw2d, cx: f32, cy: f32, r: f32, core: usize, glow: Option<usize>) {
    if let Some(g) = glow {
        d.circle_outline(cx, cy, r + 1.0, 1.0, PAL[g]);
    }
    d.circle_outline(cx, cy, r, 1.0, PAL[core]);
}

fn spiral(d: &mut Draw2d, cx: f32, cy: f32, max_r: f32, col: usize, t_off: f32) {
    let step = 4.0f32;
    let dirs = [(step, 0.0), (0.0, step), (-step, 0.0), (0.0, -step)];
    let mut dir = (t_off as usize) % 4;
    let mut len = 2.0f32;
    let mut drawn = 0.0f32;
    let (mut px, mut py) = (cx, cy);
    while drawn < max_r * 2.0 {
        let (dx, dy) = dirs[dir % 4];
        let nx = (px + dx * len).clamp(cx - max_r, cx + max_r);
        let ny = (py + dy * len).clamp(cy - max_r, cy + max_r);
        d.line(px, py, nx, ny, 1.0, PAL[col]);
        px = nx;
        py = ny;
        dir += 1;
        len += 1.0;
        drawn += len;
        if drawn > max_r * 6.0 {
            break;
        }
    }
}

fn dash_line(d: &mut Draw2d, x1: f32, y1: f32, x2: f32, y2: f32, col: usize) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    if len == 0.0 {
        return;
    }
    let mut i = 0.0f32;
    while i < len {
        let t1 = i / len;
        let t2 = ((i + 3.0) / len).min(1.0);
        d.line(x1 + dx * t1, y1 + dy * t1, x1 + dx * t2, y1 + dy * t2, 1.0, PAL[col]);
        i += 6.0;
    }
}

fn draw_tree(d: &mut Draw2d, x: i32, y: i32, h: i32, trunk: usize, leaf: usize, style: i32) {
    let (x, y) = (x as f32, y as f32);
    let th = h / 3;
    let thf = th as f32;
    // 幹
    d.rect_outline(x - 3.0, y - thf, 6.0, thf, 1.0, PAL[trunk]);
    match style {
        0 => {
            let r = thf;
            let half = (th / 2) as f32;
            circb(d, x, y - thf - r + 2.0, r, leaf, None);
            circb(d, x - half, y - thf - half, r - 4.0, leaf, None);
            circb(d, x + half, y - thf - half, r - 4.0, leaf, None);
        }
        1 => {
            let ch = th * 2;
            let chf = ch as f32;
            d.rect_outline(x - 3.0, y - chf, 6.0, chf, 1.0, PAL[trunk]);
            d.rect_outline(x - 10.0, y - (ch / 2) as f32 - 4.0, 7.0, 6.0, 1.0, PAL[trunk]);
            d.rect_outline(x + 3.0, y - (ch / 2) as f32 - 4.0, 7.0, 6.0, 1.0, PAL[trunk]);
        }
        _ => {
            let h3 = th * 3;
            let h3f = h3 as f32;
            d.rect_outline(x - 3.0, y - h3f, 6.0, h3f, 1.0, PAL[trunk]);
            let mut yy = 0;
            while yy < h3 {
                let yf = yy as f32;
                d.line(x - 3.0, y - h3f + yf, x - 8.0, y - h3f + yf + 3.0, 1.0, PAL[leaf]);
                d.line(x + 3.0, y - h3f + yf, x + 8.0, y - h3f + yf + 3.0, 1.0, PAL[leaf]);
                yy += 6;
            }
        }
    }
}

// ── 描画本体 ────────────────────────────────────────────────────────────────

impl LineBoyGame {
    fn draw(&mut self) {
        self.draw.clear();
        match self.scene {
            Scene::Title => self.draw_title_canvas(),
            _ => self.draw_play_scene(),
        }
    }

    fn draw_title_canvas(&mut self) {
        // 背景はクリア色（CB）。浮遊パーティクルと渦巻き装飾を描く
        let parts = std::mem::take(&mut self.title_parts);
        for p in &parts {
            draw_particle(&mut self.draw, p, 0.0);
        }
        self.title_parts = parts;
        let deco: [(f32, f32, f32, usize); 5] = [
            (80.0, 100.0, 30.0, CG),
            (80.0, 230.0, 20.0, CL),
            (W - 80.0, 120.0, 24.0, CO),
            (W - 120.0, 280.0, 20.0, CC),
            (W / 2.0 + 60.0, 160.0, 18.0, CPU),
        ];
        for (i, s) in deco.iter().enumerate() {
            let cx = s.0 + (self.frame as f32 * 0.03 + i as f32).sin() * 10.0;
            let cx = cx.round();
            spiral(&mut self.draw, cx, s.1, s.2, s.3, (self.frame / 20) as f32);
        }
    }

    fn draw_play_scene(&mut self) {
        let (ox, oy) = if self.scene == Scene::Play && self.shake > 0 {
            (
                self.rng.range_f32(-(self.shake as f32), self.shake as f32) / 4.0,
                self.rng.range_f32(-(self.shake as f32), self.shake as f32) / 4.0,
            )
        } else {
            (0.0, 0.0)
        };
        self.draw.offset = (ox, oy);
        self.draw_bg();
        self.draw_terrain();
        let cam_x = self.cam_x;
        if let Some(lv) = &self.level {
            for e in &lv.enemies {
                if e.alive {
                    draw_enemy(&mut self.draw, e, cam_x);
                }
            }
        }
        let boss_draw = self.boss_on || self.scene != Scene::Play;
        if boss_draw {
            if let Some(b) = &self.boss {
                if b.alive {
                    draw_boss(&mut self.draw, b, cam_x, self.frame);
                }
            }
        }
        self.draw_player();
        let parts = std::mem::take(&mut self.parts);
        for p in &parts {
            draw_particle(&mut self.draw, p, cam_x);
        }
        self.parts = parts;
        self.draw.offset = (0.0, 0.0);
    }

    fn draw_bg(&mut self) {
        let theme = &THEMES[self.world];
        self.draw.rect(0.0, 0.0, W, H, PAL[CB]);
        let seed = (self.world * 9999 + 42) as u32;
        let cam_x = self.cam_x;
        let frame = self.frame;
        // 渦巻き
        {
            let mut rng = JsRng::new(seed + 777);
            for _ in 0..6 {
                let sx = rng.next_f64() as f32 * LEVEL_W;
                let sy = rng.ri(30, (H - 220.0) as i32) as f32;
                let r = rng.ri(8, 18) as f32;
                let scx = wrapf(sx - cam_x * 0.25, W + 60.0) - 30.0;
                spiral(&mut self.draw, scx, sy, r, theme.sky, ((frame / 30) % 4) as f32);
            }
        }
        // ダッシュライン
        {
            let mut rng = JsRng::new(seed + 333);
            for _ in 0..8 {
                let lx = rng.next_f64() as f32 * LEVEL_W;
                let ly = rng.ri(15, (H / 3.0) as i32) as f32;
                let ln = rng.ri(20, 60) as f32;
                let lsx = wrapf(lx - cam_x * 0.15, W + 80.0) - 40.0;
                dash_line(&mut self.draw, lsx, ly, lsx + ln, ly, theme.sky);
            }
        }
        // 星
        {
            let mut rng = JsRng::new(seed + 999);
            for _ in 0..20 {
                let px2 = rng.next_f64() as f32 * LEVEL_W;
                let py2 = rng.ri(30, (H - 200.0) as i32) as f32;
                let spx = wrapf(px2 - cam_x * 0.18 + frame as f32 * 0.2, W + 40.0) - 20.0;
                self.draw.rect_outline(spx, py2, 3.0, 3.0, 1.0, PAL[theme.sky]);
            }
        }
    }

    fn draw_terrain(&mut self) {
        let theme = &THEMES[self.world];
        let (col, acc) = (theme.ground, theme.accent);
        let cam_x = self.cam_x;
        let frame = self.frame;
        let lv = match &self.level {
            Some(l) => l,
            None => return,
        };
        // 木
        for t in &lv.data.trees {
            let sx = (t.0 as f32 - cam_x) as i32;
            if -80 < sx && sx < (W + 80.0) as i32 {
                draw_tree(&mut self.draw, sx, t.1, t.2, t.3, t.4, t.5);
            }
        }
        // 足場
        for p in &lv.data.plats {
            let sx = (p.0 as f32 - cam_x) as i32;
            if sx + p.2 < 0 || sx > W as i32 {
                continue;
            }
            let (sxf, py) = (sx as f32, p.1 as f32);
            let mut i = 0;
            while i < p.2 {
                let tooth = if (i / 8) % 2 == 0 { 4.0 } else { 2.0 };
                self.draw.line(sxf + i as f32, py, sxf + i as f32 + 8.0, py, 1.0, PAL[col]);
                pset(&mut self.draw, sxf + i as f32, py - tooth, col);
                i += 8;
            }
            line_core(&mut self.draw, sxf, py + 4.0, sxf + p.2 as f32, py + 4.0, acc);
        }
        // 地面
        for s in &lv.data.segs {
            let sx = (s.0 as f32 - cam_x) as i32;
            if sx + s.2 < 0 || sx > W as i32 {
                continue;
            }
            let (sxf, syf) = (sx as f32, s.1 as f32);
            let mut i = 0;
            while i < s.2 {
                let tooth = if (i / 8) % 2 == 0 { 5.0 } else { 3.0 };
                let seg_end = (i + 8).min(s.2);
                self.draw.line(sxf + i as f32, syf, sxf + seg_end as f32, syf, 1.0, PAL[col]);
                if i + 4 < s.2 {
                    pset(&mut self.draw, sxf + i as f32 + 4.0, syf - tooth, col);
                }
                i += 8;
            }
            let ay = (syf + 8.0).min(H - 2.0);
            line_core(&mut self.draw, sxf, ay, sxf + s.2 as f32, ay, acc);
        }
        // アイテム
        for it in &lv.items {
            let sx = (it.0 - cam_x) as i32;
            if -10 < sx && sx < (W + 10.0) as i32 {
                let bob = ((frame as f32 * 0.12 + it.0 * 0.05).sin() * 2.0).round();
                self.draw
                    .rect_outline(sx as f32 - 3.0, it.1 + bob - 3.0, 6.0, 6.0, 1.0, PAL[CY]);
                pset(&mut self.draw, sx as f32, it.1 + bob, CW);
            }
        }
        // ゴール
        let gx = (lv.goal_x - cam_x) as i32;
        if -20 < gx && gx < (W + 20.0) as i32 {
            let gxf = gx as f32;
            line_core(&mut self.draw, gxf, H - 180.0, gxf, H - 100.0, CW);
            self.draw.rect_outline(gxf, H - 180.0, 30.0, 16.0, 1.0, PAL[CY]);
            self.draw.text("GO", gxf + 4.0, H - 178.0, 1.5, PAL[CY]);
        }
    }

    fn draw_player(&mut self) {
        let cam_x = self.cam_x;
        let p = match &self.player {
            Some(p) => p,
            None => return,
        };
        if !p.alive {
            return;
        }
        if p.inv > 0 && (p.inv / 4) % 2 == 1 {
            return;
        }
        let col = THEMES[self.world].ground;
        let sx = (p.x - cam_x) as i32;
        let mut sy = p.y as i32;
        let sw = (PW * p.sx) as i32;
        let sh = (PH * p.sy) as i32;
        let ox = ((PW - sw as f32) / 2.0) as i32;
        let oy = (PH - sh as f32) as i32;
        let (sxf, syf) = (sx as f32, sy as f32);
        rectb(
            &mut self.draw,
            (sx + ox) as f32,
            (sy + oy) as f32,
            sw as f32,
            sh as f32,
            col,
            Some(CGR),
        );
        pset(
            &mut self.draw,
            (sx + if p.facing > 0 { 8 } else { 2 }) as f32,
            (sy + oy + 3) as f32,
            CW,
        );
        if p.on_ground && p.vx.abs() < 0.3 {
            sy += ((p.anim as f32 * 0.15).sin() * 1.5).round() as i32;
        }
        let syf2 = sy as f32;
        if !p.on_ground {
            line_core(&mut self.draw, sxf + 2.0, syf2 + PH, sxf + 2.0 + (p.facing * 3) as f32, syf2 + PH + 5.0, col);
            line_core(&mut self.draw, sxf + PW - 3.0, syf2 + PH, sxf + PW - 3.0 - (p.facing * 3) as f32, syf2 + PH + 5.0, col);
        } else {
            let leg = ((p.anim as f32 * 0.35) as i32) % 4;
            let base_y = syf2 + sh as f32 + oy as f32;
            line_core(&mut self.draw, sxf + 3.0, base_y, sxf + 3.0 + leg as f32, base_y + 5.0, col);
            line_core(&mut self.draw, sxf + sw as f32 - 4.0, base_y, sxf + sw as f32 - 4.0 - leg as f32, base_y + 5.0, col);
        }
    }
}

fn wrapf(v: f32, m: f32) -> f32 {
    ((v % m) + m) % m
}

fn draw_particle(d: &mut Draw2d, p: &Particle, cam_x: f32) {
    let a = p.life as f32 / p.life_max as f32;
    if a <= 0.0 {
        return;
    }
    let sx = (p.x - cam_x) as i32 as f32;
    let sy = p.y as i32 as f32;
    let col = if a > 0.4 { p.col } else { CGR };
    match p.shape {
        0 => pset(d, sx, sy, col),
        1 => {
            let s = ((a * 3.0) as i32).max(1) as f32;
            d.rect_outline(sx - s, sy - s, s * 2.0 + 1.0, s * 2.0 + 1.0, 1.0, PAL[col]);
        }
        _ => {
            line_core(d, p.ox - cam_x, p.oy, sx, sy, col);
            if p.shape == 3 {
                pset(d, sx, sy, CW);
            }
        }
    }
}

fn draw_enemy(d: &mut Draw2d, e: &Enemy, cam_x: f32) {
    let s = 10.0f32;
    let sx = (e.x - cam_x) as i32 as f32;
    let sy = (e.y - s + (e.anim as f32 * 0.2).sin() * 2.0) as i32 as f32;
    rectb(d, sx - 5.0, sy, s, s, e.col, Some(CB));
    let ex = if e.vx > 0.0 { sx - 2.0 } else { sx + 2.0 };
    pset(d, ex, sy + 3.0, CW);
    pset(d, sx, sy + 3.0, CW);
    let leg = (((e.anim as f32 * 0.3) as i32) % 4) as f32;
    line_core(d, sx - 3.0, sy + s, sx - 3.0 + leg, sy + s + 4.0, e.col);
    line_core(d, sx + 3.0, sy + s, sx + 3.0 - leg, sy + s + 4.0, e.col);
}

/// 4 隅の点から矩形の枠を描く（回転矩形用）
fn outline4(d: &mut Draw2d, pts: [(f32, f32); 4], col: usize) {
    for i in 0..4 {
        let a = pts[i];
        let b = pts[(i + 1) % 4];
        line_core(d, a.0, a.1, b.0, b.1, col);
    }
}

fn draw_boss(d: &mut Draw2d, b: &Boss, cam_x: f32, _frame: u64) {
    let sx = (b.x - cam_x) as i32 as f32;
    let sy = b.y as i32 as f32;
    let flash = b.flash % 2 == 1;
    let col = if flash { CW } else { b.col };
    let timer = b.timer as f32;
    match b.kind {
        0 => {
            // 巨木の精（FOREST）
            let trunk = if flash { CW } else { CBR };
            let leaf = if flash { CW } else { CG };
            let arm = |d: &mut Draw2d, sgn: f32| {
                line_core(d, sx + 8.0 * sgn, sy + 2.0, sx + 26.0 * sgn, sy - 8.0, trunk);
                line_core(d, sx + 26.0 * sgn, sy - 8.0, sx + 33.0 * sgn, sy - 14.0, trunk);
                line_core(d, sx + 26.0 * sgn, sy - 8.0, sx + 31.0 * sgn, sy - 2.0, trunk);
                line_core(d, sx + 9.0 * sgn, sy - 14.0, sx + 11.0 * sgn, sy + 24.0, trunk);
                line_core(d, sx + 11.0 * sgn, sy + 24.0, sx + 17.0 * sgn, sy + 30.0, trunk);
            };
            arm(d, -1.0);
            arm(d, 1.0);
            line_core(d, sx - 11.0, sy + 24.0, sx + 11.0, sy + 24.0, trunk);
            line_core(d, sx - 3.0, sy - 8.0, sx - 4.0, sy + 16.0, CGR);
            line_core(d, sx + 4.0, sy - 4.0, sx + 3.0, sy + 12.0, CGR);
            circb(d, sx, sy - 30.0, 17.0, leaf, Some(CL));
            circb(d, sx - 15.0, sy - 22.0, 12.0, leaf, Some(CL));
            circb(d, sx + 15.0, sy - 22.0, 12.0, leaf, Some(CL));
            circb(d, sx, sy - 20.0, 13.0, leaf, None);
            pset(d, sx - 5.0, sy + 2.0, CY);
            pset(d, sx + 5.0, sy + 2.0, CY);
            line_core(d, sx - 4.0, sy + 10.0, sx + 4.0, sy + 10.0, CY);
        }
        1 => {
            // 太陽の化身（DESERT）
            for i in 0..12 {
                let a = (i as f32) / 12.0 * std::f32::consts::TAU + timer * 0.02;
                let len = if i % 2 == 0 { 20.0 } else { 12.0 };
                line_glow(
                    d,
                    sx + a.cos() * 30.0,
                    sy + a.sin() * 30.0,
                    sx + a.cos() * (30.0 + len),
                    sy + a.sin() * (30.0 + len),
                    CO,
                    CY,
                );
            }
            circb(d, sx, sy, 30.0, col, Some(CY));
            circb(d, sx, sy, 23.0, CY, None);
            line_core(d, sx - 15.0, sy - 8.0, sx - 5.0, sy - 4.0, col);
            line_core(d, sx + 15.0, sy - 8.0, sx + 5.0, sy - 4.0, col);
            pset(d, sx - 10.0, sy - 2.0, CW);
            pset(d, sx + 10.0, sy - 2.0, CW);
            for i in 0..4 {
                let zx = sx - 12.0 + (i * 8) as f32;
                line_core(d, zx, sy + 10.0, zx + 4.0, sy + 15.0, col);
                line_core(d, zx + 4.0, sy + 15.0, zx + 8.0, sy + 10.0, col);
            }
        }
        2 => {
            // CAVE: 円盤ボス
            let r = b.size;
            circb(d, sx, sy, r + 6.0, CGR, None);
            circb(d, sx, sy, r + 4.0, if flash { col } else { CB }, None);
            circb(d, sx, sy, r, col, Some(CGR));
            line_core(d, sx - r + 4.0, sy, sx + r - 4.0, sy, col);
            line_core(d, sx, sy - r + 4.0, sx, sy + r - 4.0, col);
            for i in 0..8 {
                let a = (i as f32) / 8.0 * std::f32::consts::TAU + timer * 0.05;
                d.rect_outline(
                    sx + a.cos() * (r - 8.0) - 4.0,
                    sy + a.sin() * (r - 8.0) - 4.0,
                    8.0,
                    8.0,
                    1.0,
                    PAL[col],
                );
            }
            for i in 0..6 {
                let a2 = (i as f32) / 6.0 * std::f32::consts::TAU - timer * 0.08;
                pset(d, sx + a2.cos() * (r - 20.0), sy + a2.sin() * (r - 20.0), CW);
            }
        }
        3 => {
            // 大おばけ（GHOST）
            let body = if flash { CP } else { CW };
            let wob = (timer * 0.15).sin() * 2.0;
            let ghost_fill = PAL[CW].with_alpha(0.20);
            d.circle_fill(sx, sy - 2.0, 24.0, ghost_fill);
            d.rect(sx - 24.0, sy - 2.0, 48.0, 28.0, ghost_fill);
            // 輪郭: 左辺 → 頭の弧 → 右辺 → 波打つ裾
            line_core(d, sx - 24.0, sy + 26.0, sx - 24.0, sy - 2.0, body);
            d.arc_outline(sx, sy - 2.0, 24.0, std::f32::consts::PI, std::f32::consts::TAU, 1.0, PAL[body]);
            line_core(d, sx + 24.0, sy - 2.0, sx + 24.0, sy + 26.0, body);
            for i in 0..4 {
                let x0 = sx + 24.0 - (i * 12) as f32;
                let dy = (if i % 2 == 0 { 5.0 } else { -3.0 }) + wob * 0.3;
                let p0 = (x0, sy + 26.0);
                let pc = (x0 - 3.0, sy + 26.0 + dy);
                let p2 = (x0 - 12.0, sy + 26.0);
                let mut prev = p0;
                for k in 1..=3 {
                    let t = k as f32 / 3.0;
                    let q = (
                        (1.0 - t) * (1.0 - t) * p0.0 + 2.0 * (1.0 - t) * t * pc.0 + t * t * p2.0,
                        (1.0 - t) * (1.0 - t) * p0.1 + 2.0 * (1.0 - t) * t * pc.1 + t * t * p2.1,
                    );
                    line_core(d, prev.0, prev.1, q.0, q.1, body);
                    prev = q;
                }
            }
            line_core(d, sx - 24.0, sy + 6.0, sx - 33.0, sy + 12.0 + wob, body);
            line_core(d, sx + 24.0, sy + 6.0, sx + 33.0, sy + 12.0 - wob, body);
            d.ellipse_fill(sx - 9.0, sy - 4.0, 3.5, 5.5, PAL[CB]);
            d.ellipse_fill(sx + 9.0, sy - 4.0, 3.5, 5.5, PAL[CB]);
            d.ellipse_fill(sx, sy + 8.0, 4.0, 6.5, PAL[CB]);
        }
        _ => {
            // ドラゴン（VOLCANO）: □ベースの胴体と頭
            let body_col = if flash { CW } else { CR };
            // 軌跡の四角い胴体節
            let tr = &b.trail;
            let n = tr.len() as i32;
            let mut i = n - 6;
            while i >= 10 {
                let p = tr[i as usize];
                let q = tr[((i - 6).max(0)) as usize];
                let t = i as f32 / n as f32;
                let half = (15.0 * (1.0 - t * 0.7)).max(2.5);
                let ang = (p.1 - q.1).atan2(p.0 - q.0);
                let cxp = p.0 - cam_x;
                d.rect_outline_rot(cxp, p.1, half + 1.0, ang, 1.0, PAL[CGR]);
                d.rect_outline_rot(cxp, p.1, half, ang, 1.0, PAL[body_col]);
                let (s2, c2) = ang.sin_cos();
                let rot = |lx: f32, ly: f32| (cxp + lx * c2 - ly * s2, p.1 + lx * s2 + ly * c2);
                let a1 = rot(-3.0, -half);
                let a2 = rot(0.0, -half - 7.0);
                let a3 = rot(3.0, -half);
                line_core(d, a1.0, a1.1, a2.0, a2.1, CO);
                line_core(d, a2.0, a2.1, a3.0, a3.1, CO);
                i -= 6;
            }
            // 翼（首の位置から）
            if !tr.is_empty() {
                let neck = tr[(10).min(tr.len() - 1)];
                let nx = (neck.0 - cam_x) as i32 as f32;
                let ny = neck.1 as i32 as f32;
                let flap = (timer * 0.3).sin() * 6.0;
                let pts = [
                    (nx, ny - 6.0),
                    (nx - 16.0, ny - 26.0 - flap),
                    (nx - 6.0, ny - 14.0),
                    (nx + 4.0, ny - 28.0 - flap),
                    (nx + 8.0, ny - 8.0),
                ];
                for k in 0..4 {
                    line_core(d, pts[k].0, pts[k].1, pts[k + 1].0, pts[k + 1].1, CO);
                }
            }
            // 頭と胴体の四角
            let (ca, sa) = (b.ang.cos(), b.ang.sin());
            let pp = |fx: f32, fy: f32| (sx + ca * fx - sa * fy, sy + sa * fx + ca * fy);
            let hs = 17.0f32;
            let rect_p = |d: &mut Draw2d, x: f32, y: f32, w: f32, h: f32, col: usize| {
                outline4(d, [pp(x, y), pp(x + w, y), pp(x + w, y + h), pp(x, y + h)], col);
            };
            rect_p(d, -hs - 1.0, -hs - 1.0, (hs + 1.0) * 2.0, (hs + 1.0) * 2.0, CO);
            rect_p(d, -hs, -hs, hs * 2.0, hs * 2.0, body_col);
            rect_p(d, hs - 2.0, -8.0, 16.0, 16.0, body_col);
            for k in 0..3 {
                rect_p(d, hs + 2.0 + (k * 5) as f32, -6.0, 3.0, 5.0, CW);
            }
            // 角
            let h1 = pp(-8.0, -12.0);
            let h2 = pp(-22.0, -24.0);
            let h3 = pp(-26.0, -10.0);
            line_core(d, h1.0, h1.1, h2.0, h2.1, CY);
            line_core(d, h2.0, h2.1, h3.0, h3.1, CY);
            // 目・眉・鼻
            let eye = pp(4.0, -6.0);
            d.rect(eye.0 - 2.0, eye.1 - 2.0, 4.0, 4.0, PAL[CY]);
            let b1 = pp(-2.0, -11.0);
            let b2 = pp(10.0, -8.0);
            line_core(d, b1.0, b1.1, b2.0, b2.1, CO);
            let nose = pp(31.0, 0.0);
            pset(d, nose.0, nose.1, CO);
        }
    }
    if flash {
        circb(d, sx, sy, 10.0, CW, None);
    }
}
