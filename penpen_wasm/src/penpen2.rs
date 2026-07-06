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
    Launcher = 4,
}

// ── 障害物 ──────────────────────────────────────────────────────────────────
pub struct Obstacle {
    pub x: f32,      // トラック中心からの相対X座標 (-4.0 ~ 4.0)
    pub z: f32,      // トラックに沿ったZ座標
    pub kind: u8,    // 0=雪だるま, 1=松の木, 2=氷の結晶, 3=ジャンプ台, 4=加速板
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
    pub input_pull:     bool, // パチンコ引っ張り中か (S/Down)
    pub input_accel:    bool, // 加速中か (W/↑)
    pub input_brake:    bool, // ブレーキ中か (S/↓)

    // ゲームワールド
    pub obstacles:      Vec<Obstacle>,
    pub collectibles:   Vec<Collectible>,
    pub particles:      Vec<PenPenParticle>,
    pub track_length:   f32,
    pub level:          u32,

    pub camera_mode:    u8,  // 0=TPS, 1=TOP, 2=FPS
    pub audio_event:    u8,  // 0=なし, 1=滑走音/ジャンプ, 2=魚収集, 3=障害物衝突, 4=レベルクリア, 5=ゲームオーバー
    pub rng:            u64,

    // パチンコ引っ張り状態
    pub pull_dist:      f32,  // パチンコ引っ張り距離 (0.0 ~ 5.0)
    pub is_pulling:     bool, // パチンコを実際に引っ張り中か
}

// ── トラック定義関数 ────────────────────────────────────────────────────────
#[inline]
pub fn track_center(z: f32) -> f32 {
    (z * 0.035).sin() * 5.5 + (z * 0.012).cos() * 2.5
}

#[inline]
pub fn track_center_y(z: f32) -> f32 {
    if z < 0.0 {
        return 0.0;
    }
    // Dynamic mountain slope with prominent peaks and valleys
    let base_y = -z * 0.065; // 6.5% overall down slope
    let hill_y = (z * 0.015).sin() * 18.0 + (z * 0.008).cos() * 10.0 + (z * 0.005).sin() * 5.0;
    base_y + hill_y
}

#[inline]
pub fn track_y(x: f32, z: f32) -> f32 {
    let dx = x - track_center(z);
    track_center_y(z) + dx * dx * 0.08 // 放物線状のハーフパイプ + センターY
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
            player_vx: 0.0, player_vy: 0.0, player_speed: 0.0,
            player_hp: 5, player_max_hp: 5,
            score: 0, fish_collected: 0, invincible: 0.0,
            input_dx: 0.0, input_jump: false, input_pull: false, input_accel: false, input_brake: false,
            obstacles: Vec::new(),
            collectibles: Vec::new(),
            particles: Vec::new(),
            track_length: 600.0,
            level: 1,
            camera_mode: 0,
            audio_event: 0,
            rng: 98765,
            pull_dist: 0.0,
            is_pulling: false,
        })
    }

    pub fn start(&mut self) {
        self.scene = PenPenScene::Launcher;
        self.player_x = track_center(-2.0);
        self.player_y = track_y(self.player_x, -2.0);
        self.player_z = -2.0;
        self.player_vx = 0.0;
        self.player_vy = 0.0;
        self.player_speed = 0.0;
        self.player_hp = self.player_max_hp;
        self.score = 0;
        self.fish_collected = 0;
        self.invincible = 1.5;
        self.pull_dist = 0.0;
        self.is_pulling = false;
        self.input_pull = false;
        self.input_accel = false;
        self.input_brake = false;
        self.particles.clear();
        self.generate_level();
    }

    fn generate_level(&mut self) {
        self.obstacles.clear();
        self.collectibles.clear();
        self.track_length = 500.0 + (self.level as f32 * 100.0);

        let mut rng = 12345 + (self.level as u64 * 333);

        // 障害物・特殊ボードの配置 (Z = 40 から ゴール前 40 まで)
        let mut z = 40.0;
        while z < self.track_length - 40.0 {
            let roll_val = lcg_f(&mut rng);
            if roll_val < 0.12 {
                // 加速板 (Boost Pad) を配置！
                let rx = -3.0 + lcg_f(&mut rng) * 6.0;
                self.obstacles.push(Obstacle {
                    x: rx,
                    z,
                    kind: 4, // Boost Pad
                    active: true,
                });
            } else if roll_val < 0.22 {
                // ジャンプ台 (Jump Ramp) を配置！
                let rx = -2.5 + lcg_f(&mut rng) * 5.0;
                self.obstacles.push(Obstacle {
                    x: rx,
                    z,
                    kind: 3, // Jump Ramp
                    active: true,
                });
            } else if roll_val < 0.55 {
                // 障害物を配置 (雪だるま、木、結晶)
                let rx = -4.0 + lcg_f(&mut rng) * 8.0;
                let kind = (lcg_f(&mut rng) * 3.0) as u8; // 0, 1, 2
                self.obstacles.push(Obstacle {
                    x: rx,
                    z: z + lcg_f(&mut rng) * 4.0,
                    kind,
                    active: true,
                });
            }
            z += 24.0 - (self.level as f32).min(10.0) * 0.8;
        }

        // 魚の配置 (列やジャンプが必要な空中位置などに配置)
        z = 25.0;
        while z < self.track_length - 20.0 {
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
        if self.scene != PenPenScene::Playing && self.scene != PenPenScene::Launcher { return; }

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

        // ── 1. パチンコ引っ張りシーンの処理 ────────────────────────────────────
        if self.scene == PenPenScene::Launcher {
            // 引っ張り距離に応じてペンギンを後ろに移動（スリングショット効果）
            self.player_z = -2.0 - self.pull_dist * 0.5;
            self.player_x = track_center(self.player_z);

            // ペンギンの高さは地形に合わせて変化（跳ねが自然に起こる）
            let surface_y = track_y(self.player_x, self.player_z);
            if self.player_y <= surface_y {
                self.player_y = surface_y;
                self.player_vy = 0.0;
            } else {
                // 空中：重力を適用
                self.player_vy -= 16.0 * dt;
                self.player_y += self.player_vy * dt;
                if self.player_y <= surface_y {
                    self.player_y = surface_y;
                    self.player_vy = 0.0;
                }
            }

            if self.input_pull {
                self.is_pulling = true;
                self.pull_dist = (self.pull_dist + 2.5 * dt).min(5.0);
            } else if self.is_pulling {
                // キーを離した ➔ 射出！
                self.player_speed = 8.0 + self.pull_dist * 8.5; // 最大で 50.5m/s
                self.scene = PenPenScene::Playing;
                self.is_pulling = false;
                self.audio_event = 1; // 射出音（ジャンプ音を流用）
                self.spawn_snow_particles(self.player_x, self.player_y, self.player_z, 25);
            }
            return;
        }

        // ── 2. プレイヤー物理演算 (Playing シーン) ─────────────────────────────────
        // 速度切れ判定（前進速度が0.2以下になればスピード切れゲームオーバー）
        if self.player_speed <= 0.2 {
            self.scene = PenPenScene::GameOver;
            self.audio_event = 5; // ゲームオーバー音
            return;
        }

        let center_x = track_center(self.player_z);
        let rel_x = self.player_x - center_x;

        // 地形による摩擦係数の変化 (氷と雪の判定)
        // |rel_x| <= 1.8 ➔ 氷 (Ice: 低摩擦), |rel_x| > 1.8 ➔ 雪 (Snow: 高摩擦)
        let is_ice = rel_x.abs() <= 1.8;
        let f_coeff = if is_ice { 0.015 } else { 0.16 };
        let lateral_friction_coeff = if is_ice { 1.2 } else { 2.8 };

        // 勾配（Z傾斜）による重力加速度の計算 (前進力/減速力)
        let slope = (track_center_y(self.player_z + 0.1) - track_center_y(self.player_z - 0.1)) / 0.2;
        let mut accel_z = -slope * 22.0; // 下り坂でプラス、上り坂でマイナス

        // 加速・ブレーキ入力を反映
        if self.input_accel {
            accel_z += 15.0; // 加速
        } else if self.input_brake {
            accel_z -= 12.0; // ブレーキ
        }

        // 前進速度の更新 (勾配力 - 摩擦抵抗 - 空気抵抗)
        let drag_force = 0.0025 * self.player_speed * self.player_speed;
        let friction_force = f_coeff * self.player_speed;
        self.player_speed += (accel_z - friction_force - drag_force) * dt;
        self.player_speed = self.player_speed.max(0.0);
        self.player_speed = self.player_speed.max(0.0);
        self.player_z += self.player_speed * dt;

        // 左右移動とハーフパイプの斜面重力 (中心に向かう力: f = -k * x)
        let slope_gravity = -rel_x * 8.0;
        let input_force = self.input_dx * 35.0;
        let friction_x = -self.player_vx * lateral_friction_coeff;
        let accel_x = input_force + slope_gravity + friction_x;

        self.player_vx += accel_x * dt;
        self.player_x += self.player_vx * dt;

        // コース壁衝突制限 (|相対X| >= 5.0 で反射・減速)
        let rel_x_new = self.player_x - center_x;
        if rel_x_new.abs() >= 5.0 {
            let sign = if rel_x_new > 0.0 { 1.0 } else { -1.0 };
            self.player_x = center_x + sign * 4.95;
            self.player_vx = -self.player_vx * 0.25; // はね返り
            self.player_speed = (self.player_speed - 3.0).max(0.0); // 減速
            self.spawn_snow_particles(self.player_x, self.player_y, self.player_z, 5);
        }

        // 3. ジャンプ物理演算
        let surface_y = track_y(self.player_x, self.player_z);
        if self.player_y <= surface_y {
            // 地面に接している
            self.player_y = surface_y;
            self.player_vy = 0.0;

            if self.input_jump {
                let jump_force = if self.input_accel { 9.5 } else { 7.0 }; // 加速中はジャンプ強化
                self.player_vy = jump_force;
                self.audio_event = 1; // ジャンプ音イベント
                self.spawn_snow_particles(self.player_x, self.player_y, self.player_z, 8);
            }
        } else {
            // 空中
            let gravity = if self.input_accel { 12.0 } else { 16.0 }; // 加速中は重力を弱める
            self.player_vy -= gravity * dt;
            self.player_y += self.player_vy * dt;

            // 着地判定
            if self.player_y <= surface_y {
                let fall_speed = -self.player_vy;
                self.player_y = surface_y;
                self.player_vy = 0.0;
                let num_parts = if fall_speed > 6.0 { 12 } else { 5 };
                self.spawn_snow_particles(self.player_x, self.player_y, self.player_z, num_parts);
            }
        }

        // ── 3. コアゲームループの更新 ──────────────────────────────────────────
        // 1. スノーボードのスノープライル・エフェクト (定期的にパーティクル発生)
        if self.player_y <= surface_y + 0.05 {
            let slip_amount = self.player_vx.abs() * 0.1;
            let spawn_rate = (self.player_speed * 0.4 + slip_amount * 5.0) as i32;
            if (ts as i32 % 4) < spawn_rate.min(8) {
                let p_vx = -self.player_vx * 0.2 + (lcg_f(&mut self.rng) - 0.5) * 1.5;
                let p_vy = lcg_f(&mut self.rng) * 1.0;
                let p_vz = -self.player_speed * 0.3;
                let part_col = if is_ice { [0.75, 0.9, 1.0] } else { [0.95, 0.96, 1.0] };
                self.particles.push(PenPenParticle {
                    x: self.player_x + (lcg_f(&mut self.rng) - 0.5) * 0.3,
                    y: self.player_y,
                    z: self.player_z - 0.4,
                    vx: p_vx, vy: p_vy, vz: p_vz,
                    life: 0.6 + lcg_f(&mut self.rng) * 0.4,
                    max_life: 1.0,
                    col: part_col,
                    is_sparkle: false,
                });
            }
        }

        // 2. 衝突判定 (障害物・加速板・ジャンプ台)
        let mut hit_obstacle = None;
        for obs in &mut self.obstacles {
            if !obs.active { continue; }
            let abs_ox = track_center(obs.z) + obs.x;
            let abs_oy = track_y(abs_ox, obs.z);

            let dz = (self.player_z - obs.z).abs();
            let dx = (self.player_x - abs_ox).abs();
            let dy = (self.player_y - abs_oy).abs();

            if dz < 0.65 && dx < 0.7 && dy < 0.9 {
                obs.active = false;
                hit_obstacle = Some((abs_ox, abs_oy, obs.z, obs.kind));
                break;
            }
        }

        if let Some((abs_ox, abs_oy, obs_z, kind)) = hit_obstacle {
            if kind <= 2 {
                // 障害物 (雪だるま、木、結晶) ➔ 即クラッシュ (ゲームオーバー)！
                self.player_hp = 0;
                self.player_speed = 0.0;
                self.scene = PenPenScene::GameOver;
                self.audio_event = 5; // ゲームオーバー音
                self.spawn_burst_particles(abs_ox, abs_oy + 0.3, obs_z, [1.0, 0.2, 0.1], 30);
            } else if kind == 3 {
                // ジャンプ台 (Jump Ramp) ➔ 空中ジャンプ！
                self.player_vy = 9.5;
                self.player_speed += 6.0;
                self.audio_event = 1; // ジャンプ音
                self.spawn_burst_particles(abs_ox, abs_oy, obs_z, [1.0, 0.9, 0.0], 15);
            } else if kind == 4 {
                // 加速板 (Boost Pad) ➔ 前進加速！
                self.player_speed = (self.player_speed + 12.0).min(45.0);
                self.audio_event = 2; // チャイム音
                self.spawn_burst_particles(abs_ox, abs_oy, obs_z, [0.0, 1.0, 0.5], 20);
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
            self.scene = PenPenScene::LevelClear;
            self.audio_event = 4; // クリア音
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
        match key {
            1 => self.input_dx = -1.0,
            2 => self.input_dx = 1.0,
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

    pub fn set_pull_input(&mut self, on: bool) {
        self.input_pull = on;
    }

    pub fn set_accel_input(&mut self, on: bool) {
        self.input_accel = on;
    }

    pub fn set_brake_input(&mut self, on: bool) {
        self.input_brake = on;
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

// ジャンプ台 (Jump Ramp: イエローネオン)
fn draw_jump_ramp(verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>, cx: f32, cy: f32, cz: f32) {
    let col = [1.0, 0.82, 0.0, 3.0]; // ネオンイエロー
    push_box(verts, idxs, cx, cy + 0.1, cz - 0.4, 0.7, 0.1, 0.25, col);
    push_box(verts, idxs, cx, cy + 0.25, cz, 0.7, 0.22, 0.25, col);
    push_box(verts, idxs, cx, cy + 0.4, cz + 0.4, 0.7, 0.35, 0.25, col);
}

// 加速板 (Boost Pad: グリーンネオン)
fn draw_boost_pad(verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>, cx: f32, cy: f32, cz: f32) {
    let col = [0.0, 1.0, 0.4, 3.0]; // ネオングリーン
    push_box(verts, idxs, cx, cy + 0.02, cz, 0.8, 0.01, 0.5, col);
    push_box(verts, idxs, cx - 0.2, cy + 0.02, cz + 0.1, 0.15, 0.01, 0.15, col);
    push_box(verts, idxs, cx + 0.2, cy + 0.02, cz + 0.1, 0.15, 0.01, 0.15, col);
}

// 背景の山 (Low-Poly Mountain Peak)
fn draw_mountain(verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>, cx: f32, cy: f32, cz: f32, height: f32) {
    let rock_col = [0.3, 0.35, 0.25, 1.0]; // ダークグレーブラウン (岩肌)
    let snow_col = [0.92, 0.94, 0.98, 1.0]; // 薄いホワイト (雪化粧)
    let tree_col = [0.25, 0.55, 0.2, 1.0]; // 濃い緑 (樹木)

    let base_w = height * 0.7;
    push_box(verts, idxs, cx, cy + height * 0.2, cz, base_w, height * 0.2, base_w, rock_col);
    push_box(verts, idxs, cx, cy + height * 0.5, cz, base_w * 0.65, height * 0.2, base_w * 0.65, rock_col);
    push_box(verts, idxs, cx, cy + height * 0.8, cz, base_w * 0.35, height * 0.2, base_w * 0.35, snow_col);

    // 山肌に樹木を配置
    let tree_h = height * 0.15;
    let trees = vec![
        (-base_w * 0.3, height * 0.35, -base_w * 0.2),
        (base_w * 0.2, height * 0.40, base_w * 0.3),
        (-base_w * 0.15, height * 0.55, base_w * 0.1),
        (base_w * 0.35, height * 0.50, -base_w * 0.25),
    ];
    for (dx, dy, dz) in trees {
        push_box(verts, idxs, cx + dx, cy + dy, cz + dz, tree_h * 0.3, tree_h, tree_h * 0.3, tree_col);
    }
}

// パチンコのゴムひも (Dotted laser elastic band)
fn draw_elastic_band(verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>, p1: [f32; 3], p2: [f32; 3], col: [f32; 4]) {
    let segments = 8;
    for i in 0..segments {
        let t = i as f32 / segments as f32;
        let cx = p1[0] + (p2[0] - p1[0]) * t;
        let cy = p1[1] + (p2[1] - p1[1]) * t;
        let cz = p1[2] + (p2[2] - p1[2]) * t;
        push_box(verts, idxs, cx, cy, cz, 0.06, 0.03, 0.15, col);
    }
}

// ペンギンモデル描画
fn draw_penguin(verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>, cx: f32, cy: f32, cz: f32, yaw: f32, roll: f32, is_launcher: bool) {
    let (black_col, white_col, orange_col, scarf_col) = if is_launcher {
        // ランチャーシーン：明るい緑色のペンギン
        (
            [0.2, 0.75, 0.3, 1.0],           // 緑（体）
            [0.85, 0.95, 0.85, 1.0],         // 淡い白緑（お腹）
            [1.0, 0.8, 0.2, 1.0],            // 黄オレンジ（クチバシ・足）
            [0.2, 1.0, 0.3, 3.5]              // ネオン明るい緑（マフラー）
        )
    } else {
        // 通常シーン：真っ黒いペンギン
        (
            [0.0, 0.0, 0.0, 1.0],            // 黒（体）
            [0.94, 0.94, 0.94, 1.0],         // 白（お腹）
            [1.0, 0.45, 0.0, 1.0],           // オレンジ（クチバシ・足）
            [1.0, 0.05, 0.3, 3.0]             // ネオンピンク（マフラー）
        )
    };

    // 体
    push_oriented_box(verts, idxs, cx, cy, cz, 0.0, 0.42, 0.0, 0.24, 0.34, 0.20, yaw, roll, black_col);
    // お腹
    push_oriented_box(verts, idxs, cx, cy, cz, 0.0, 0.36, 0.19, 0.17, 0.24, 0.03, yaw, roll, white_col);
    // クチバシ
    push_oriented_box(verts, idxs, cx, cy, cz, 0.0, 0.62, 0.22, 0.07, 0.04, 0.08, yaw, roll, orange_col);

    // 羽（左）
    push_oriented_box(verts, idxs, cx, cy, cz, -0.26, 0.38, -0.04, 0.03, 0.22, 0.08, yaw, roll + 0.25, black_col);
    // 羽（右）
    push_oriented_box(verts, idxs, cx, cy, cz, 0.26, 0.38, -0.04, 0.03, 0.22, 0.08, yaw, roll - 0.25, black_col);

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
    let min_render_z = (current_z - 35.0).max(-12.0);
    let max_render_z = current_z + 180.0;

    // ── トラック（ハーフパイプ氷＆雪面）の構築 ──────────────────────────────────────
    let z_step = 2.0_f32;
    let u_step = 1.0_f32;

    let mut z0 = (min_render_z / z_step).floor() * z_step;
    while z0 < max_render_z {
        let z1 = z0 + z_step;
        let xc0 = track_center(z0);
        let xc1 = track_center(z1);

        let mut u0 = -7.0_f32;
        while u0 < 7.0 {
            let u1 = u0 + u_step;

            // 4つ角の座標計算 (ハーフパイプを浅くして雪山らしく)
            let yc0 = track_center_y(z0);
            let yc1 = track_center_y(z1);
            let p0 = [xc0 + u0, yc0 + u0 * u0 * 0.03, z0];
            let p1 = [xc0 + u1, yc0 + u1 * u1 * 0.03, z0];
            let p2 = [xc1 + u1, yc1 + u1 * u1 * 0.03, z1];
            let p3 = [xc1 + u0, yc1 + u0 * u0 * 0.03, z1];

            // 氷（中央）と雪（左右）のビジュアルデザイン
            let is_ice = u0.abs() < 1.2;
            let col = if is_ice {
                let stripe = ((z0 / 5.0) as i32 % 2 == 0) ^ ((u0 as i32) % 3 == 0);
                if stripe {
                    [0.88, 0.92, 0.98, 1.0]  // 淡いアイス（ほぼ雪色）
                } else {
                    [0.90, 0.94, 1.0, 1.0]   // より淡いアイス
                }
            } else {
                let stripe = (z0 / 8.0) as i32 % 2 == 0;
                if stripe {
                    [0.92, 0.94, 0.98, 1.0]  // 明るい雪原
                } else {
                    [0.88, 0.90, 0.96, 1.0]  // わずかに暗い雪原
                }
            };

            let b = verts.len() as u32;
            for p in &[p0, p1, p2, p3] {
                verts.push(Vertex { pos: *p, _p: 0.0, col });
            }
            idxs.extend_from_slice(&[b, b+1, b+2, b, b+2, b+3]);

            u0 += u_step;
        }

        // ネオン境界ポールは削除（自然な雪山のため）

        z0 += z_step;
    }

    // ── 背景の巨大な雪山（フラザンの両脇にプロシージャル生成） ───────────────────────────
    let m_step = 30.0_f32;
    let mut mz = (min_render_z / m_step).floor() * m_step;
    while mz < max_render_z {
        let m_xc = track_center(mz);
        let m_yc = track_center_y(mz);

        // 左側の山
        draw_mountain(&mut verts, &mut idxs, m_xc - 24.0, m_yc, mz + 10.0, 24.0);
        // 右側の山 (Zをずらして配置)
        draw_mountain(&mut verts, &mut idxs, m_xc + 24.0, m_yc, mz + 30.0, 26.0);

        mz += m_step;
    }

    // ── パチンコ（スタート台の巨大なゴムひも）の構築 ──────────────────────────────
    if min_render_z <= 0.0 && max_render_z >= -8.0 {
        let sling_z = 0.0_f32;
        let left_post_x = -3.2_f32;
        let left_post_y = track_y(left_post_x, sling_z);
        let right_post_x = 3.2_f32;
        let right_post_y = track_y(right_post_x, sling_z);
        
        let sling_col = [0.9, 0.7, 0.3, 2.5]; // 自然な木の色（茶色）

        // 左の巨大なY字ポスト
        push_box(&mut verts, &mut idxs, left_post_x, left_post_y + 0.6, sling_z, 0.16, 0.6, 0.16, sling_col);
        push_oriented_box(&mut verts, &mut idxs, left_post_x, left_post_y + 1.2, sling_z, -0.22, 0.3, 0.0, 0.08, 0.3, 0.08, 0.0, 0.45, sling_col);
        push_oriented_box(&mut verts, &mut idxs, left_post_x, left_post_y + 1.2, sling_z, 0.22, 0.3, 0.0, 0.08, 0.3, 0.08, 0.0, -0.45, sling_col);

        // 右の巨大なY字ポスト
        push_box(&mut verts, &mut idxs, right_post_x, right_post_y + 0.6, sling_z, 0.16, 0.6, 0.16, sling_col);
        push_oriented_box(&mut verts, &mut idxs, right_post_x, right_post_y + 1.2, sling_z, -0.22, 0.3, 0.0, 0.08, 0.3, 0.08, 0.0, 0.45, sling_col);
        push_oriented_box(&mut verts, &mut idxs, right_post_x, right_post_y + 1.2, sling_z, 0.22, 0.3, 0.0, 0.08, 0.3, 0.08, 0.0, -0.45, sling_col);

        // ゴムバンドの描画 (プレイヤーがパチンコを引っ張っているか、Launcher シーンにいる場合)
        if g.scene == PenPenScene::Launcher {
            let left_hook = [left_post_x - 0.32, left_post_y + 1.45, sling_z];
            let right_hook = [right_post_x + 0.32, right_post_y + 1.45, sling_z];
            let penguin_back = [g.player_x, g.player_y + 0.42, g.player_z - 0.2];
            
            let band_color = [0.0, 1.0, 0.8, 3.0]; // ネオンシアンゴムひも
            draw_elastic_band(&mut verts, &mut idxs, left_hook, penguin_back, band_color);
            draw_elastic_band(&mut verts, &mut idxs, right_hook, penguin_back, band_color);
        }
    }

    // ── ゴールゲート（巨大なネオンレインボー門） ───────────────────────────────
    let gate_z = g.track_length;
    if gate_z >= min_render_z && gate_z <= max_render_z {
        let xc = track_center(gate_z);
        let base_y = track_y(xc - 5.0, gate_z);
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

    // ── 障害物（雪だるま・木・結晶・ジャンプ台・加速板）の描画 ─────────────────────────────
    for obs in &g.obstacles {
        if !obs.active { continue; }
        if obs.z < min_render_z || obs.z > max_render_z { continue; }

        let abs_ox = track_center(obs.z) + obs.x;
        let abs_oy = track_y(abs_ox, obs.z);

        match obs.kind {
            0 => draw_snowman(&mut verts, &mut idxs, abs_ox, abs_oy, obs.z),
            1 => draw_pine_tree(&mut verts, &mut idxs, abs_ox, abs_oy, obs.z),
            2 => draw_crystal(&mut verts, &mut idxs, abs_ox, abs_oy, obs.z),
            3 => draw_jump_ramp(&mut verts, &mut idxs, abs_ox, abs_oy, obs.z),
            4 => draw_boost_pad(&mut verts, &mut idxs, abs_ox, abs_oy, obs.z),
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
            let is_launcher = g.scene == PenPenScene::Launcher;
            draw_penguin(&mut verts, &mut idxs, g.player_x, g.player_y, g.player_z, yaw, roll, is_launcher);
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
    let next_y = track_y(next_xc, next_z);

    let [eye, ctr, up] = if g.scene == PenPenScene::Launcher {
        // パチンコ引っ張りシーンのカメラ：ペンギンを後上から見る
        [
            [g.player_x, g.player_y + 2.0, g.player_z - 5.5],  // より後上から見る
            [g.player_x, g.player_y + 0.3, g.player_z],        // ペンギンを見ている
            [0.0, 1.0, 0.0]
        ]
    } else {
        match g.camera_mode {
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
                    [next_xc, next_y + 0.45, next_z],
                    [0.0, 1.0, 0.0]
                ]
            }
            _ => {
                // TPSビュー（ペンギンを中心に追従・地形起伏には追従しない）
                [
                    [g.player_x, g.player_y + 1.5, g.player_z - 4.2],  // ペンギンの後ろ上
                    [g.player_x, g.player_y + 0.3, g.player_z + 6.0],  // ペンギンの前方を見る
                    [0.0, 1.0, 0.0]
                ]
            }
        }
    };

    let view = look_at(eye, ctr, up);
    let proj = perspective(PI * 0.40, aspect, 0.05, 100.0);
    let vp   = mat_mul(proj, view);

    // ランチャーシーン時は明るく鮮やかなライティング、それ以外は自然なスキーリゾート風
    let (lights, fog_col) = if g.scene == PenPenScene::Launcher {
        // 明るく鮮やかなランチャーシーン（都市的な見た目）
        (
            [
                Light {
                    pos: [g.player_x, g.player_y + 25.0, g.player_z - 15.0, 0.0],
                    col: [1.0, 1.0, 0.9, 5.5] // 太陽光（より明るい黄白色）
                },
                Light {
                    pos: [g.player_x, g.player_y + 2.0, g.player_z, 0.0],
                    col: [1.0, 1.0, 1.0, 3.5] // ペンギン上の明るい白光
                },
                Light {
                    pos: [track_center(g.player_z + 40.0), track_center_y(g.player_z + 40.0) + 20.0, g.player_z + 40.0, 0.0],
                    col: [0.8, 0.95, 1.0, 3.0] // スカイライト（明るい空色）
                },
                Light {
                    pos: [track_center(g.player_z + 20.0), track_center_y(g.player_z + 20.0) + 10.0, g.player_z + 20.0, 1.0],
                    col: [0.6, 0.8, 1.0, 2.0] // 影の補助光
                },
            ],
            [0.7, 0.92, 1.0, 1.0] // 明るく鮮やかな空色フォグ
        )
    } else {
        // 自然なスキーリゾート風ライティング
        (
            [
                Light {
                    pos: [g.player_x, g.player_y + 20.0, g.player_z - 10.0, 0.0],
                    col: [1.0, 0.95, 0.75, 4.0] // 太陽光（黄白色）
                },
                Light {
                    pos: [g.player_x, g.player_y + 1.2, g.player_z, 0.0],
                    col: [1.0, 1.0, 1.0, 2.5] // ペンギン頭上の補助白色ライト
                },
                Light {
                    pos: [track_center(g.player_z + 40.0), track_center_y(g.player_z + 40.0) + 15.0, g.player_z + 40.0, 0.0],
                    col: [0.7, 0.85, 1.0, 2.0] // スカイライト（空色の反射光）
                },
                Light {
                    pos: [track_center(g.player_z + 20.0), track_center_y(g.player_z + 20.0) + 8.0, g.player_z + 20.0, 1.0],
                    col: [0.5, 0.7, 1.0, 1.5] // 影の補助光（青）
                },
            ],
            [0.8, 0.9, 1.0, 1.0] // 薄い青フォグ
        )
    };

    Uni {
        vp,
        time: g.time as f32,
        warp: 0.0,
        pad: [0.0; 2],
        lights,
        fog_col,
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

