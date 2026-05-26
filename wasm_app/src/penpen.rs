use wasm_bindgen::JsCast;
use std::f32::consts::PI;
use crate::math::{lcg_f, perspective, look_at, mat_mul};
use crate::gpu::GpuState;
use crate::geometry::{Vertex, Light, Uni};

// ── シーン定義 ──────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq)]
pub enum PenPenScene {
    Title = 0,
    Playing = 1,
    LevelClear = 2,
    GameOver = 3,
}

// ── 障害物 ──────────────────────────────────────────────────────────────────
pub struct Obstacle {
    pub x: f32,      // トラック中心からの相対X座標 (-4.0 ~ 4.0)
    pub z: f32,      // トラックに沿ったZ座標
    pub kind: u8,    // 0=雪だるま, 1=松の木, 2=氷の結晶
    pub active: bool,
}

// ── 収集アイテム（魚） ──────────────────────────────────────────────────────
pub struct Collectible {
    pub x: f32,      // トラック中心からの相対X座標 (-4.0 ~ 4.0)
    pub y: f32,      // トラック面からの高さ
    pub z: f32,      // トラックに沿ったZ座標
    pub active: bool,
}

// ── パーティクル ────────────────────────────────────────────────────────────
pub struct PenPenParticle {
    pub x: f32, pub y: f32, pub z: f32,
    pub vx: f32, pub vy: f32, pub vz: f32,
    pub life: f32,
    pub max_life: f32,
    pub col: [f32; 3],
    pub is_sparkle: bool,
}

// ── ゲーム状態 ──────────────────────────────────────────────────────────────
pub struct PenPenGame {
    pub gpu:            GpuState,
    pub scene:          PenPenScene,
    pub time:           f64,
    pub dt:             f32,
    pub prev_ts:        f64,

    // プレイヤー物理・状態
    pub player_x:       f32, // 絶対X座標
    pub player_y:       f32, // 絶対Y座標
    pub player_z:       f32, // 絶対Z座標
    pub player_vx:      f32,
    pub player_vy:      f32,
    pub player_speed:   f32, // 前進Z速度
    pub player_hp:      i32,
    pub player_max_hp:  i32,
    pub score:          u32,
    pub fish_collected: u32,
    pub invincible:     f32, // 無敵タイマー

    // 入力
    pub input_dx:       f32,  // 左右入力 (-1.0 ~ 1.0)
    pub input_jump:     bool, // ジャンプ中か

    // ゲームワールド
    pub obstacles:      Vec<Obstacle>,
    pub collectibles:   Vec<Collectible>,
    pub particles:      Vec<PenPenParticle>,
    pub track_length:   f32,
    pub level:          u32,

    pub camera_mode:    u8,  // 0=TPS, 1=TOP, 2=FPS
    pub audio_event:    u8,  // 0=なし, 1=滑走音/ジャンプ, 2=魚収集, 3=障害物衝突, 4=レベルクリア, 5=ゲームオーバー
    pub rng:            u64,
    pub demo_mode:      bool,
}

// ── トラック定義関数 ────────────────────────────────────────────────────────
#[inline]
pub fn track_center(z: f32) -> f32 {
    (z * 0.035).sin() * 5.5 + (z * 0.012).cos() * 2.5
}

#[inline]
pub fn track_y(x: f32, z: f32) -> f32 {
    let dx = x - track_center(z);
    dx * dx * 0.08 // 放物線状のハーフパイプ
}

impl PenPenGame {
    pub async fn new(canvas_id: &str) -> Result<Self, String> {
        let canvas = web_sys::window().unwrap()
            .document().unwrap()
            .get_element_by_id(canvas_id).unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| "canvas cast failed")?;
        let gpu = GpuState::new(canvas, true).await?;
        Ok(Self {
            gpu,
            scene: PenPenScene::Title,
            time: 0.0, dt: 0.0, prev_ts: 0.0,
            player_x: 0.0, player_y: 0.0, player_z: 0.0,
            player_vx: 0.0, player_vy: 0.0, player_speed: 10.0,
            player_hp: 5, player_max_hp: 5,
            score: 0, fish_collected: 0, invincible: 0.0,
            input_dx: 0.0, input_jump: false,
            obstacles: Vec::new(),
            collectibles: Vec::new(),
            particles: Vec::new(),
            track_length: 600.0,
            level: 1,
            camera_mode: 0,
            audio_event: 0,
            rng: 98765,
            demo_mode: false,
        })
    }

    pub async fn new_demo(canvas_id: &str) -> Result<Self, String> {
        let canvas = web_sys::window().unwrap()
            .document().unwrap()
            .get_element_by_id(canvas_id).unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| "canvas cast failed")?;
        // Demo mode: VSync OFF for high FPS
        let gpu = GpuState::new(canvas, false).await?;
        Ok(Self {
            gpu,
            scene: PenPenScene::Title,
            time: 0.0, dt: 0.0, prev_ts: 0.0,
            player_x: 0.0, player_y: 0.0, player_z: 0.0,
            player_vx: 0.0, player_vy: 0.0, player_speed: 10.0,
            player_hp: 5, player_max_hp: 5,
            score: 0, fish_collected: 0, invincible: 0.0,
            input_dx: 0.0, input_jump: false,
            obstacles: Vec::new(),
            collectibles: Vec::new(),
            particles: Vec::new(),
            track_length: 600.0,
            level: 1,
            camera_mode: 0,
            audio_event: 0,
            rng: 98765,
            demo_mode: true,
        })
    }

    pub fn start(&mut self) {
        self.scene = PenPenScene::Playing;
        self.player_x = track_center(0.0);
        self.player_y = 0.0;
        self.player_z = 0.0;
        self.player_vx = 0.0;
        self.player_vy = 0.0;
        self.player_speed = 10.0;
        self.player_hp = self.player_max_hp;
        self.score = 0;
        self.fish_collected = 0;
        self.invincible = 1.5;
        self.particles.clear();
        
        self.obstacles.clear();
        self.collectibles.clear();
        if self.demo_mode {
            self.track_length = 1000.0;
            self.generate_segment(0.0, 1000.0);
        } else {
            self.track_length = 500.0 + (self.level as f32 * 100.0);
            self.generate_segment(40.0, self.track_length - 40.0);
        }
    }

    fn generate_level(&mut self) {
        // Redundant but kept for compatibility if needed elsewhere
        self.start();
    }

    fn generate_segment(&mut self, start_z: f32, end_z: f32) {
        let mut rng = 12345 + (self.level as u64 * 333) + (start_z as u64 * 7);

        // 障害物の配置
        if !self.demo_mode {
            let mut z = start_z;
            while z < end_z {
                let num_obs = if lcg_f(&mut rng) < 0.3 { 2 } else { 1 };
                for _ in 0..num_obs {
                    let rx = -4.0 + lcg_f(&mut rng) * 8.0;
                    let kind = (lcg_f(&mut rng) * 3.0) as u8; // 0, 1, 2
                    self.obstacles.push(Obstacle {
                        x: rx,
                        z: z + lcg_f(&mut rng) * 8.0,
                        kind,
                        active: true,
                    });
                }
                z += 28.0 - (self.level as f32).min(10.0) * 1.0;
            }
        }

        // 魚の配置
        let mut z = start_z + 10.0;
        while z < end_z - 10.0 {
            let formation = (lcg_f(&mut rng) * 3.0) as u8;
            let rx = -3.5 + lcg_f(&mut rng) * 7.0;

            match formation {
                0 => {
                    // 直線の魚 (3連)
                    for i in 0..3 {
                        let fz = z + i as f32 * 3.0;
                        self.collectibles.push(Collectible {
                            x: rx,
                            y: 0.2,
                            z: fz,
                            active: true,
                        });
                    }
                }
                1 => {
                    // アーチ状の魚 (ジャンプで取る)
                    for i in 0..3 {
                        let fz = z + i as f32 * 3.0;
                        let fy = if i == 1 { 1.4 } else { 0.7 };
                        self.collectibles.push(Collectible {
                            x: rx,
                            y: fy,
                            z: fz,
                            active: true,
                        });
                    }
                }
                _ => {
                    // S字蛇行の魚
                    for i in 0..4 {
                        let fz = z + i as f32 * 4.0;
                        let fx = rx + (i as f32 * 1.5 - 2.25);
                        if fx >= -4.5 && fx <= 4.5 {
                            self.collectibles.push(Collectible {
                                x: fx,
                                y: 0.2,
                                z: fz,
                                active: true,
                            });
                        }
                    }
                }
            }
            z += 35.0;
        }
    }

    pub fn tick(&mut self, ts: f64) {
        if self.scene != PenPenScene::Playing { return; }

        let dt = if self.prev_ts > 0.0 {
            ((ts - self.prev_ts) * 0.001).min(0.05) as f32
        } else { 0.016 };
        self.prev_ts = ts;
        self.dt = dt;
        self.time = ts * 0.001;
        self.audio_event = 0;

        if self.invincible > 0.0 {
            self.invincible -= dt;
        }

        // ── プレイヤー物理演算 ──────────────────────────────────────────────
        if self.demo_mode {
            // Demo mode: Auto-steering towards center
            let target_x = track_center(self.player_z);
            if self.player_x < target_x - 0.3 {
                self.input_dx = (target_x - self.player_x).clamp(0.0, 1.0);
            } else if self.player_x > target_x + 0.3 {
                self.input_dx = (target_x - self.player_x).clamp(-1.0, 0.0);
            } else {
                self.input_dx = 0.0;
            }
        }

        // 1. 前進速度の加速 (滑走抵抗を考慮しつつ自動加速)
        let target_base_speed = 18.0 + (self.level as f32 * 1.5);
        if self.player_speed < target_base_speed {
            self.player_speed += 2.0 * dt;
        } else {
            self.player_speed -= 0.5 * dt;
        }
        self.player_z += self.player_speed * dt;

        // 2. 左右移動（氷の上特有の滑り・慣性）
        let center_x = track_center(self.player_z);
        let rel_x = self.player_x - center_x;

        // ハーフパイプの斜面重力 (中心に向かう力: f = -k * x)
        let slope_gravity = -rel_x * 8.0;

        // キー入力フォース
        let input_force = self.input_dx * 35.0;

        // 摩擦（速度に比例する減衰）
        let friction = -self.player_vx * 1.6;

        // 加速度適用
        let accel_x = input_force + slope_gravity + friction;
        self.player_vx += accel_x * dt;
        self.player_x += self.player_vx * dt;

        // コース壁衝突制限 (|相対X| >= 5.0 で反射・減速)
        let rel_x_new = self.player_x - center_x;
        if rel_x_new.abs() >= 5.0 {
            let sign = if rel_x_new > 0.0 { 1.0 } else { -1.0 };
            self.player_x = center_x + sign * 4.95;
            self.player_vx = -self.player_vx * 0.25; // はね返り
            self.player_speed = (self.player_speed - 2.0).max(5.0); // 減速
            // 雪しぶき
            self.spawn_snow_particles(self.player_x, self.player_y, self.player_z, 5);
        }

        // 3. ジャンプ物理演算
        let surface_y = track_y(self.player_x, self.player_z);
        if self.player_y <= surface_y {
            // 地面に接している
            self.player_y = surface_y;
            self.player_vy = 0.0;

            if self.input_jump {
                self.player_vy = 7.0; // 上向き初速
                self.audio_event = 1; // ジャンプ音イベント
                self.spawn_snow_particles(self.player_x, self.player_y, self.player_z, 8);
            }
        } else {
            // 空中
            self.player_vy -= 16.0 * dt; // 重力加速度
            self.player_y += self.player_vy * dt;

            // 着地判定
            if self.player_y <= surface_y {
                self.player_y = surface_y;
                self.player_vy = 0.0;
                self.spawn_snow_particles(self.player_x, self.player_y, self.player_z, 6);
            }
        }

        // ── コアゲームループの更新 ──────────────────────────────────────────
        // 1. スノーボードのスノープライル・エフェクト (定期的にパーティクル発生)
        if self.player_y <= surface_y + 0.05 {
            let slip_amount = self.player_vx.abs() * 0.1;
            let spawn_rate = (self.player_speed * 0.4 + slip_amount * 5.0) as i32;
            if (ts as i32 % 4) < spawn_rate.min(8) {
                // ペンギンの足元から後ろに雪を散らす
                let p_vx = -self.player_vx * 0.2 + (lcg_f(&mut self.rng) - 0.5) * 1.5;
                let p_vy = lcg_f(&mut self.rng) * 1.0;
                let p_vz = -self.player_speed * 0.3;
                self.particles.push(PenPenParticle {
                    x: self.player_x + (lcg_f(&mut self.rng) - 0.5) * 0.3,
                    y: self.player_y,
                    z: self.player_z - 0.4,
                    vx: p_vx, vy: p_vy, vz: p_vz,
                    life: 0.6 + lcg_f(&mut self.rng) * 0.4,
                    max_life: 1.0,
                    col: [0.9, 0.95, 1.0],
                    is_sparkle: false,
                });
            }
        }

        // 2. 衝突判定 (障害物)
        let mut hit_obstacle = None;
        for obs in &mut self.obstacles {
            if !obs.active { continue; }
            let abs_ox = track_center(obs.z) + obs.x;
            let abs_oy = track_y(abs_ox, obs.z);

            let dz = (self.player_z - obs.z).abs();
            let dx = (self.player_x - abs_ox).abs();
            let dy = (self.player_y - abs_oy).abs();

            if dz < 0.6 && dx < 0.6 && dy < 0.8 {
                if self.invincible <= 0.0 {
                    obs.active = false;
                    hit_obstacle = Some((abs_ox, abs_oy, obs.z));
                    break;
                }
            }
        }

        if let Some((abs_ox, abs_oy, obs_z)) = hit_obstacle {
            self.player_hp -= 1;
            self.player_speed = (self.player_speed * 0.4).max(4.0);
            self.player_vx = -self.player_vx * 0.5;
            self.invincible = 1.5;
            self.audio_event = 3; // 衝突音
            self.spawn_burst_particles(abs_ox, abs_oy + 0.3, obs_z, [1.0, 0.3, 0.0], 15);

            if self.player_hp <= 0 {
                self.scene = PenPenScene::GameOver;
                self.audio_event = 5; // ゲームオーバー音
            }
        }

        // 3. 収集判定 (魚)
        let mut collected_fish = None;
        for fish in &mut self.collectibles {
            if !fish.active { continue; }
            let abs_fx = track_center(fish.z) + fish.x;
            let abs_fy = track_y(abs_fx, fish.z) + fish.y;

            let dz = (self.player_z - fish.z).abs();
            let dx = (self.player_x - abs_fx).abs();
            let dy = (self.player_y - abs_fy).abs();

            if dz < 0.8 && dx < 0.8 && dy < 1.0 {
                fish.active = false;
                collected_fish = Some((abs_fx, abs_fy, fish.z));
                break;
            }
        }

        if let Some((abs_fx, abs_fy, fish_z)) = collected_fish {
            self.fish_collected += 1;
            self.score += 150 + (self.player_speed as u32 * 5);
            self.player_speed = (self.player_speed + 3.0).min(32.0); // スピードアップ
            self.audio_event = 2; // アイテム取得音
            self.spawn_burst_particles(abs_fx, abs_fy, fish_z, [1.0, 0.1, 0.7], 10);
        }


        // 4. パーティクル更新
        for p in &mut self.particles {
            if p.life <= 0.0 { continue; }
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.z += p.vz * dt;
            if !p.is_sparkle {
                p.vy -= 1.5 * dt; // 雪は重力で下に落ちる
            }
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);

        // 5. ゴール判定
        if self.player_z >= self.track_length {
            if self.demo_mode {
                // Endless: generate next segment and prune old ones
                let old_length = self.track_length;
                self.track_length += 1000.0;
                self.generate_segment(old_length, self.track_length);
                
                // Prune
                self.obstacles.retain(|obs| obs.z > self.player_z - 100.0);
                self.collectibles.retain(|coll| coll.z > self.player_z - 100.0);
            } else {
                self.scene = PenPenScene::LevelClear;
                self.audio_event = 4; // クリア音
            }
        }
    }

    fn spawn_snow_particles(&mut self, x: f32, y: f32, z: f32, count: usize) {
        for _ in 0..count {
            self.particles.push(PenPenParticle {
                x, y, z,
                vx: (lcg_f(&mut self.rng) - 0.5) * 4.0,
                vy: lcg_f(&mut self.rng) * 3.0 + 1.0,
                vz: (lcg_f(&mut self.rng) - 0.6) * 3.0,
                life: 0.5 + lcg_f(&mut self.rng) * 0.4,
                max_life: 0.9,
                col: [0.95, 0.98, 1.0],
                is_sparkle: false,
            });
        }
    }

    fn spawn_burst_particles(&mut self, x: f32, y: f32, z: f32, color: [f32; 3], count: usize) {
        for _ in 0..count {
            self.particles.push(PenPenParticle {
                x, y, z,
                vx: (lcg_f(&mut self.rng) - 0.5) * 6.0,
                vy: (lcg_f(&mut self.rng) - 0.3) * 6.0 + 1.0,
                vz: (lcg_f(&mut self.rng) - 0.5) * 6.0,
                life: 0.4 + lcg_f(&mut self.rng) * 0.3,
                max_life: 0.7,
                col: color,
                is_sparkle: true,
            });
        }
    }

    pub fn act(&mut self, key: i32) {
        // キーアクション: JS側から呼び出される
        // key: 0=なし, 1=左移動開始, 2=右移動開始, 3=移動停止, 4=ジャンプ
        match key {
            1 => self.input_dx = 1.0,  // 左で右へ (逆)
            2 => self.input_dx = -1.0, // 右で左へ (逆)
            3 => self.input_dx = 0.0,
            4 => self.input_jump = true,
            5 => self.input_jump = false,
            _ => {}
        }
    }

    pub fn set_move_input(&mut self, dx: f32) {
        self.input_dx = dx.clamp(-1.0, 1.0);
    }

    pub fn set_jump_input(&mut self, on: bool) {
        self.input_jump = on;
    }

    pub fn next_level(&mut self) {
        self.level += 1;
        self.start();
    }

    pub fn reset_game(&mut self) {
        self.level = 1;
        self.start();
    }
}

// ── ジオメトリ生成・ヘルパー ────────────────────────────────────────────────
#[inline]
fn rot_y_z(x: f32, y: f32, z: f32, yaw: f32, roll: f32) -> [f32; 3] {
    // 1. Z軸回転 (Roll: ローカルでの傾き)
    let cr = roll.cos();
    let sr = roll.sin();
    let rx = x * cr - y * sr;
    let ry = x * sr + y * cr;

    // 2. Y軸回転 (Yaw: 進行方向の回転)
    let cy = yaw.cos();
    let sy = yaw.sin();
    let fx = rx * cy + z * sy;
    let fy = ry;
    let fz = -rx * sy + z * cy;

    [fx, fy, fz]
}

// ローカルオフセット(ox,oy,oz)とサイズ(sx,sy,sz)を持つボックスをyaw, roll回転して追加
fn push_oriented_box(verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>,
                      cx: f32, cy: f32, cz: f32,
                      ox: f32, oy: f32, oz: f32,
                      sx: f32, sy: f32, sz: f32,
                      yaw: f32, roll: f32, col: [f32; 4]) {
    let _rot_center = rot_y_z(ox, oy, oz, yaw, roll);
    let (_wcx, _wcy, _wcz) = (cx + _rot_center[0], cy + _rot_center[1], cz + _rot_center[2]);
    let base = verts.len() as u32;

    for &(lx, ly, lz) in &[
        (-sx,-sy,-sz),(sx,-sy,-sz),(sx,sy,-sz),(-sx,sy,-sz),
        (-sx,-sy, sz),(sx,-sy, sz),(sx,sy, sz),(-sx,sy, sz),
    ] {
        // 回転適用（ローカル中心から）
        let rx = ox + lx;
        let ry = oy + ly;
        let rz = oz + lz;
        let p = rot_y_z(rx, ry, rz, yaw, roll);
        verts.push(Vertex { pos: [cx + p[0], cy + p[1], cz + p[2]], _p: 0.0, col });
    }

    for &(a, b, c, d) in &[
        (0u32,1,2,3),(4,7,6,5),(0,4,5,1),(2,6,7,3),(0,3,7,4),(1,5,6,2)
    ] {
        idxs.extend_from_slice(&[base+a,base+b,base+c, base+a,base+c,base+d]);
    }
}

// 軸平行ボックス
fn push_box(verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>,
             cx: f32, cy: f32, cz: f32,
             sx: f32, sy: f32, sz: f32, col: [f32; 4]) {
    push_oriented_box(verts, idxs, cx, cy, cz, 0.0, 0.0, 0.0, sx, sy, sz, 0.0, 0.0, col);
}

// 魚（ピンクのホログラム魚）
fn draw_fish(verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>, cx: f32, cy: f32, cz: f32, time: f32) {
    let fish_col = [1.0, 0.0, 0.5, 3.0]; // マテリアル3 (エミッシブ)
    let wiggle = (time * 8.0 + cz).sin() * 0.25;

    // 魚体
    push_oriented_box(verts, idxs, cx, cy, cz, 0.0, 0.0, 0.0, 0.16, 0.08, 0.08, wiggle, 0.0, fish_col);
    // 魚の尾ヒレ (後ろに配置し、逆方向に動かす)
    push_oriented_box(verts, idxs, cx, cy, cz, 0.0, 0.0, -0.22, 0.04, 0.12, 0.02, -wiggle * 1.5, 0.0, fish_col);
}

// 雪だるま
fn draw_snowman(verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>, cx: f32, cy: f32, cz: f32) {
    let body_col = [0.92, 0.95, 1.0, 1.0];
    let hat_col = [0.75, 0.1, 0.1, 1.0];
    let beak_col = [1.0, 0.45, 0.0, 1.0];

    // 下半身
    push_box(verts, idxs, cx, cy + 0.3, cz, 0.38, 0.3, 0.38, body_col);
    // 頭
    push_box(verts, idxs, cx, cy + 0.76, cz, 0.25, 0.22, 0.25, body_col);
    // 鼻
    push_box(verts, idxs, cx, cy + 0.76, cz + 0.3, 0.04, 0.04, 0.10, beak_col);
    // バケツ帽子
    push_box(verts, idxs, cx, cy + 1.05, cz, 0.18, 0.14, 0.18, hat_col);
}

// 松の木
fn draw_pine_tree(verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>, cx: f32, cy: f32, cz: f32) {
    let trunk_col = [0.38, 0.22, 0.12, 1.0];
    let leaves_col = [0.08, 0.65, 0.18, 2.0]; // マテリアル2: 壁

    // 幹
    push_box(verts, idxs, cx, cy + 0.3, cz, 0.10, 0.3, 0.10, trunk_col);
    // 葉（下層段）
    push_box(verts, idxs, cx, cy + 0.8, cz, 0.52, 0.26, 0.52, leaves_col);
    // 葉（中層段）
    push_box(verts, idxs, cx, cy + 1.25, cz, 0.38, 0.22, 0.38, leaves_col);
    // 葉（上層段）
    push_box(verts, idxs, cx, cy + 1.6, cz, 0.24, 0.16, 0.24, leaves_col);
}

// 氷の結晶障害物
fn draw_crystal(verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>, cx: f32, cy: f32, cz: f32) {
    let crystal_col = [0.0, 0.85, 1.0, 3.0]; // エミッシブ・ネオンブルー
    push_oriented_box(verts, idxs, cx, cy + 0.4, cz, 0.0, 0.0, 0.0, 0.25, 0.42, 0.25, 0.4, 0.3, crystal_col);
    push_oriented_box(verts, idxs, cx, cy + 0.3, cz, 0.0, 0.0, 0.0, 0.15, 0.35, 0.15, -0.6, -0.2, crystal_col);
}

// ペンギンモデル描画
fn draw_penguin(verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>, cx: f32, cy: f32, cz: f32, yaw: f32, roll: f32, time: f32) {
    let black_col = [0.08, 0.09, 0.14, 1.0];
    let white_col = [0.94, 0.94, 0.94, 1.0];
    let orange_col = [1.0, 0.45, 0.0, 1.0];
    let scarf_col = [1.0, 0.05, 0.3, 3.0]; // ネオンピンクマフラー (マテリアル3: エミッシブ)

    // 体
    push_oriented_box(verts, idxs, cx, cy, cz, 0.0, 0.42, 0.0, 0.24, 0.34, 0.20, yaw, roll, black_col);
    // お腹
    push_oriented_box(verts, idxs, cx, cy, cz, 0.0, 0.36, 0.19, 0.17, 0.24, 0.03, yaw, roll, white_col);
    // クチバシ
    push_oriented_box(verts, idxs, cx, cy, cz, 0.0, 0.62, 0.22, 0.07, 0.04, 0.08, yaw, roll, orange_col);

    let flap = (time * 15.0).sin() * 0.5;
    // 羽（左）: 回転中心を肩(0.38)にし、ボックス中心を下にずらす(0.38 - 0.11)
    push_oriented_box(verts, idxs, cx, cy, cz, -0.26, 0.38, -0.04, 0.03, 0.11, 0.08, yaw, 0.0, black_col);
    // 回転を適用するために、描画時にオフセットを考慮した工夫が必要ですが、
    // 現在のpush_oriented_boxの仕様(中心回転)に合わせるため、
    // 中心を肩から羽の長さの半分(0.11)下に配置します。
    // その上で、回転を適用したボックスを再計算します。
    
    // 正しい実装:
    // 肩の位置で回転させるため、回転中心(oy)を 0.38 にし、
    // ボックスの中心(oy_box)を 0.38 - 0.11 = 0.27 にします。
    // 現在の関数は回転中心とボックス中心が同一(oy)である必要があるため、
    // 描画ロジックを肩基準で回転するように修正します。

    // 左羽：回転中心を肩(0.38)に配置し、ボックスの中心を下にオフセット
    // 頂点オフセット ly を肩に対して調整します。
    // push_oriented_box(verts, idxs, cx, cy, cz, -0.26, 0.27, -0.04, 0.03, 0.11, 0.08, yaw, roll + flap, black_col);
    // 上記だと中心回転になるため、以下のように「肩で回転した位置」にボックスを配置します。

    let flap_l = flap;
    let flap_r = -flap;

    // 左羽の描画
    // 肩を中心に回転行列を計算して配置する（手動回転）
    let x_off = -0.26;
    let y_off = 0.38;
    let z_off = -0.04;
    
    // 肩を基準点とした回転後のローカル位置
    let rot_x = 0.0;
    let rot_y = -0.11;
    let rot_z = 0.0;
    
    // 簡易的に肩を中心に回転したボックスを描画
    // push_oriented_box の仕様に合わせ、回転中心を肩(0.38)に設定
    push_oriented_box(verts, idxs, cx, cy, cz, x_off, y_off + rot_y, z_off, 0.03, 0.11, 0.08, yaw, roll + flap_l, black_col);
    
    // 右羽の描画
    let x_off_r = 0.26;
    push_oriented_box(verts, idxs, cx, cy, cz, x_off_r, y_off + rot_y, z_off, 0.03, 0.11, 0.08, yaw, roll + flap_r, black_col);

    // 足（左）
    push_oriented_box(verts, idxs, cx, cy, cz, -0.11, 0.05, 0.08, 0.08, 0.04, 0.14, yaw, roll, orange_col);
    // 足（右）
    push_oriented_box(verts, idxs, cx, cy, cz, 0.11, 0.05, 0.08, 0.08, 0.04, 0.14, yaw, roll, orange_col);

    // ネオンマフラー (首周りに巻く)
    push_oriented_box(verts, idxs, cx, cy, cz, 0.0, 0.54, 0.0, 0.25, 0.04, 0.21, yaw, roll, scarf_col);
    // マフラーのタレ (左後ろになびく)
    push_oriented_box(verts, idxs, cx, cy, cz, -0.16, 0.44, -0.21, 0.04, 0.12, 0.04, yaw + 0.3, roll + 0.1, scarf_col);
}

// ── 描画シーンのビルド ──────────────────────────────────────────────────────
pub fn build_penpen_scene(g: &PenPenGame) -> (Vec<Vertex>, Vec<u32>) {
    let mut verts: Vec<Vertex> = Vec::with_capacity(8192);
    let mut idxs:  Vec<u32>   = Vec::with_capacity(16384);

    let current_z = g.player_z;
    let min_render_z = (current_z - 35.0).max(0.0);
    let max_render_z = current_z + 180.0;

    // ── トラック（ハーフパイプ氷面）の構築 ──────────────────────────────────────
    let z_step = 2.0_f32;
    let u_step = 1.0_f32;

    let mut z0 = (min_render_z / z_step).floor() * z_step;
    while z0 < max_render_z {
        let z1 = z0 + z_step;
        let xc0 = track_center(z0);
        let xc1 = track_center(z1);

        let mut u0 = -5.0_f32;
        while u0 < 5.0 {
            let u1 = u0 + u_step;

            // 4つ角の座標計算
            let p0 = [xc0 + u0, u0 * u0 * 0.08, z0];
            let p1 = [xc0 + u1, u1 * u1 * 0.08, z0];
            let p2 = [xc1 + u1, u1 * u1 * 0.08, z1];
            let p3 = [xc1 + u0, u0 * u0 * 0.08, z1];

            // 氷のグリッドデザイン (マテリアル1.0: 通常)
            let is_stripe = ((z0 / 4.0) as i32 % 2 == 0) ^ ((u0 as i32 + 5) % 2 == 0);
            let col = if is_stripe {
                [0.45, 0.72, 1.0, 1.0]  // 明るい氷ブルー
            } else {
                [0.10, 0.20, 0.44, 1.0]  // 深いアイスブルー
            };

            let b = verts.len() as u32;
            for p in &[p0, p1, p2, p3] {
                verts.push(Vertex { pos: *p, _p: 0.0, col });
            }
            idxs.extend_from_slice(&[b, b+1, b+2, b, b+2, b+3]);

            u0 += u_step;
        }

        // コース両端のネオン境界ポール (10mごとに配置)
        if (z0 as i32 % 10) == 0 {
            // 左側ポール (シアンネオン: マテリアル3)
            let left_x = xc0 - 5.0;
            let left_y = 5.0 * 5.0 * 0.08;
            push_box(&mut verts, &mut idxs, left_x, left_y + 0.3, z0, 0.08, 0.35, 0.08, [0.0, 0.85, 1.0, 3.0]);

            // 右側ポール (ピンクネオン: マテリアル3)
            let right_x = xc0 + 5.0;
            let right_y = 5.0 * 5.0 * 0.08;
            push_box(&mut verts, &mut idxs, right_x, right_y + 0.3, z0, 0.08, 0.35, 0.08, [1.0, 0.05, 0.6, 3.0]);
        }

        z0 += z_step;
    }

    // ── ゴールゲート（巨大なネオンレインボー門） ───────────────────────────────
    let gate_z = g.track_length;
    if gate_z >= min_render_z && gate_z <= max_render_z {
        let xc = track_center(gate_z);
        let base_y = 5.0 * 5.0 * 0.08; // 端の高さ
        let neon_green = [0.0, 1.0, 0.45, 3.0];

        // 左柱
        push_box(&mut verts, &mut idxs, xc - 5.0, base_y + 1.8, gate_z, 0.18, 1.8, 0.18, neon_green);
        // 右柱
        push_box(&mut verts, &mut idxs, xc + 5.0, base_y + 1.8, gate_z, 0.18, 1.8, 0.18, neon_green);
        // 横梁 (上部のバー)
        push_box(&mut verts, &mut idxs, xc, base_y + 3.6, gate_z, 5.0, 0.15, 0.18, neon_green);
        // ゴール文字板 (ゲート中央)
        push_box(&mut verts, &mut idxs, xc, base_y + 3.1, gate_z, 1.2, 0.3, 0.05, [1.0, 0.9, 0.0, 3.0]);
    }

    // ── 障害物の描画 ────────────────────────────────────────────────────────
    for obs in &g.obstacles {
        if !obs.active { continue; }
        if obs.z < min_render_z || obs.z > max_render_z { continue; }

        let abs_ox = track_center(obs.z) + obs.x;
        let abs_oy = track_y(abs_ox, obs.z);

        match obs.kind {
            0 => draw_snowman(&mut verts, &mut idxs, abs_ox, abs_oy, obs.z),
            1 => draw_pine_tree(&mut verts, &mut idxs, abs_ox, abs_oy, obs.z),
            2 => draw_crystal(&mut verts, &mut idxs, abs_ox, abs_oy, obs.z),
            _ => {}
        }
    }

    // ── アイテム（魚）の描画 ──────────────────────────────────────────────────
    for fish in &g.collectibles {
        if !fish.active { continue; }
        if fish.z < min_render_z || fish.z > max_render_z { continue; }

        let abs_fx = track_center(fish.z) + fish.x;
        let abs_fy = track_y(abs_fx, fish.z) + fish.y;

        draw_fish(&mut verts, &mut idxs, abs_fx, abs_fy, fish.z, g.time as f32);
    }

    // ── プレイヤー（ペンギン）の描画 ──────────────────────────────────────────
    if g.player_hp > 0 {
        // ダメージ中の無敵点滅
        let flash = (g.invincible > 0.0) && ((g.time * 8.0) as i32 % 2 == 0);
        if !flash {
            // カブの際のロール角（傾き）
            let roll = -g.player_vx * 0.08;
            // 進行方向への向き (Yaw)
            let yaw = (g.player_vx / g.player_speed).atan();
            draw_penguin(&mut verts, &mut idxs, g.player_x, g.player_y, g.player_z, yaw, roll, g.time as f32);
        }
    }

    // ── パーティクルの描画 ────────────────────────────────────────────────────
    for p in &g.particles {
        if p.life <= 0.0 { continue; }
        if p.z < min_render_z || p.z > max_render_z { continue; }

        let alpha = (p.life / p.max_life).min(1.0);
        let col = [p.col[0] * alpha, p.col[1] * alpha, p.col[2] * alpha, 3.0]; // パーティクルは発光

        // 十字交差クワッド
        let size = if p.is_sparkle { 0.06 } else { 0.12 * alpha };
        let base = verts.len() as u32;

        // クワッド1
        verts.push(Vertex { pos: [p.x - size, p.y, p.z], _p: 0.0, col });
        verts.push(Vertex { pos: [p.x + size, p.y, p.z], _p: 0.0, col });
        verts.push(Vertex { pos: [p.x, p.y + size, p.z], _p: 0.0, col });
        verts.push(Vertex { pos: [p.x, p.y - size, p.z], _p: 0.0, col });

        idxs.extend_from_slice(&[base, base+1, base+2, base, base+3, base+1]);
    }

    (verts, idxs)
}

// ── Uniformデータのビルド ────────────────────────────────────────────────────
pub fn build_penpen_uni(g: &PenPenGame, aspect: f32) -> Uni {
    // カメラアイ・ルックアット
    let _player_base = [g.player_x, g.player_y, g.player_z];
    let next_z = g.player_z + 12.0;
    let next_xc = track_center(next_z);

    let [eye, ctr, up] = match g.camera_mode {
        1 => {
            // TOPビュー（真上から見下ろし）
            [
                [track_center(g.player_z), g.player_y + 11.0, g.player_z - 1.5],
                [track_center(g.player_z + 4.0), g.player_y, g.player_z + 4.0],
                [0.0, 1.0, 0.01]
            ]
        }
        2 => {
            // FPSビュー（ペンギン視点）
            [
                [g.player_x, g.player_y + 0.65, g.player_z + 0.8],
                [next_xc, g.player_y + 0.45, next_z],
                [0.0, 1.0, 0.0]
            ]
        }
        _ => {
            // TPSビュー（後方追従・標準）
            let cam_z = g.player_z - 4.2;
            let cam_xc = track_center(cam_z);
            [
                [cam_xc, g.player_y + 1.45, cam_z],
                [next_xc, g.player_y + 0.35, next_z],
                [0.0, 1.0, 0.0]
            ]
        }
    };

    let view = look_at(eye, ctr, up);
    let proj = perspective(PI * 0.40, aspect, 0.05, 100.0);
    let vp   = mat_mul(proj, view);

    // プレイヤーの前方で一番近い障害物を探す
    let mut closest_obs_pos = [track_center(g.player_z + 40.0), 2.5, g.player_z + 40.0];
    let mut min_dist = f32::MAX;
    for obs in &g.obstacles {
        if obs.active && obs.z > g.player_z && obs.z < g.player_z + 80.0 {
            let abs_ox = track_center(obs.z) + obs.x;
            let abs_oy = track_y(abs_ox, obs.z) + 0.8;
            let dist = (abs_ox - g.player_x).powi(2) + (obs.z - g.player_z).powi(2);
            if dist < min_dist {
                min_dist = dist;
                closest_obs_pos = [abs_ox, abs_oy, obs.z];
            }
        }
    }

    // 4つのポイントライト
    let lights = [
        Light {
            pos: [g.player_x, g.player_y + 0.5, g.player_z, 0.0],
            col: [1.0, 0.0, 0.3, 3.2] // マフラーのピンク発光
        },
        Light {
            pos: [g.player_x, g.player_y + 1.2, g.player_z, 0.0],
            col: [1.0, 1.0, 1.0, 3.0] // ペンギン頭上の白色ライト
        },
        Light {
            pos: [closest_obs_pos[0], closest_obs_pos[1], closest_obs_pos[2], 2.0],
            col: [0.0, 0.8, 1.0, 2.5] // 障害物への青いスポットライト
        },
        Light {
            pos: [closest_obs_pos[0], closest_obs_pos[1] + 0.5, closest_obs_pos[2], 1.5],
            col: [1.0, 1.0, 0.0, 1.5] // 障害物への警告黄色ライト
        },
    ];

    Uni {
        vp,
        time: g.time as f32,
        warp: 0.0,
        pad: [0.0; 2],
        lights,
        // 背景夜空＆フォグ色
        fog_col: [0.0, 0.0, 0.04, 1.0],
    }
}

// ── サウンド定義生成 ────────────────────────────────────────────────────────
pub fn sound_def(event: u8) -> String {
    use crate::audio_tool::{SoundDef, OscDef, WaveType};
    match event {
        1 => {
            // ジャンプ音
            SoundDef::new("jump")
                .add(OscDef::new(WaveType::Sine, 280.0, 0.15, 0.22).with_sweep(520.0, 0.15))
                .to_json()
        }
        2 => {
            // 魚ゲット音 (キラキラチャイム)
            SoundDef::new("collect")
                .add(OscDef::new(WaveType::Sine, 750.0, 0.08, 0.25))
                .add(OscDef::new(WaveType::Sine, 1000.0, 0.14, 0.25).with_osc_delay(0.06))
                .to_json()
        }
        3 => {
            // クラッシュ音
            SoundDef::new("crash")
                .add(OscDef::new(WaveType::Sawtooth, 150.0, 0.35, 0.35).with_sweep(40.0, 0.35))
                .add(OscDef::new(WaveType::Triangle, 100.0, 0.35, 0.3).with_sweep(30.0, 0.35))
                .to_json()
        }
        4 => {
            // レベルクリア音 (アルペジオ)
            SoundDef::new("clear")
                .add(OscDef::new(WaveType::Triangle, 523.25, 0.15, 0.25))
                .add(OscDef::new(WaveType::Triangle, 659.25, 0.15, 0.25).with_osc_delay(0.08))
                .add(OscDef::new(WaveType::Triangle, 783.99, 0.15, 0.25).with_osc_delay(0.16))
                .add(OscDef::new(WaveType::Triangle, 1046.50, 0.40, 0.25).with_osc_delay(0.24))
                .to_json()
        }
        5 => {
            // ゲームオーバー音
            SoundDef::new("gameover")
                .add(OscDef::new(WaveType::Sawtooth, 180.0, 0.70, 0.30).with_sweep(45.0, 0.70))
                .add(OscDef::new(WaveType::Sawtooth, 178.0, 0.70, 0.30).with_sweep(44.0, 0.70).with_detune(15.0))
                .to_json()
        }
        _ => "{}".to_string(),
    }
}

