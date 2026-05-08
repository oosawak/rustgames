use wasm_bindgen::JsCast;
use std::f32::consts::{PI, TAU};
use crate::math::lcg_f;
use crate::gpu::GpuState;
use super::bullet::BulletPool;
use super::enemy::{BlasterEnemy, EnemyKind};
use super::boss::{Boss, BossPhase};
use super::camera_mode::CameraMode;
use super::geometry::{build_blaster_scene, build_blaster_uni};

// ── パーティクル ────────────────────────────────────────────────────────────
pub struct BlasterParticle {
    pub x: f32, pub y: f32, pub z: f32,
    pub vx: f32, pub vy: f32, pub vz: f32,
    pub life: f32,
    pub max_life: f32,
    pub col: [f32; 3],
}

impl BlasterParticle {
    pub fn tick(&mut self, dt: f32) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        self.z += self.vz * dt;
        self.life -= dt;
    }
}

// ── シーン ──────────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq)]
pub enum BlasterScene { Title = 0, Playing = 1, StageClear = 2, BossClear = 3, GameOver = 4 }

// ── メインゲーム状態 ────────────────────────────────────────────────────────
pub struct BlasterGame {
    pub gpu:           GpuState,
    pub scene:         BlasterScene,
    pub time:          f64,
    pub dt:            f32,
    pub prev_ts:       f64,

    pub player_x:      f32,
    pub player_z:      f32,
    pub player_body_angle:   f32,  // 車体（移動方向）
    pub player_turret_angle: f32,  // 砲塔（最近の敵方向）
    pub player_hp:     i32,
    pub player_max_hp: i32,
    pub score:         u32,
    pub invincible:    f32,
    pub shoot_timer:   f32,

    pub input_dx:      f32,
    pub input_dz:      f32,
    pub input_shoot:   bool,

    pub enemies:       Vec<BlasterEnemy>,
    pub boss:          Boss,
    pub bullets:       BulletPool,
    pub particles:     Vec<BlasterParticle>,

    pub wave:          u32,
    pub wave_timer:    f32,
    pub wave_enemies_left: u32,
    pub is_boss_wave:  bool,

    pub camera:        CameraMode,
    pub audio_event:   u8,
    pub rng:           u64,
}

impl BlasterGame {
    pub async fn new(canvas_id: &str) -> Result<Self, String> {
        let canvas = web_sys::window().unwrap()
            .document().unwrap()
            .get_element_by_id(canvas_id).unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| "canvas cast failed")?;
        let gpu = GpuState::new(canvas).await?;
        Ok(Self {
            gpu,
            scene: BlasterScene::Title,
            time: 0.0, dt: 0.0, prev_ts: 0.0,
            player_x: 0.0, player_z: 5.0,
            player_body_angle: 0.0, player_turret_angle: 0.0,
            player_hp: 5, player_max_hp: 5,
            score: 0, invincible: 0.0, shoot_timer: 0.0,
            input_dx: 0.0, input_dz: 0.0, input_shoot: false,
            enemies: Vec::new(), boss: Boss::new(),
            bullets: BulletPool::new(),
            particles: Vec::new(),
            wave: 0, wave_timer: 0.0, wave_enemies_left: 0, is_boss_wave: false,
            camera: CameraMode::Tps,
            audio_event: 0,
            rng: 12345,
        })
    }

    pub fn start(&mut self) {
        self.scene = BlasterScene::Playing;
        self.player_x = 0.0; self.player_z = 5.0;
        self.player_body_angle = 0.0; self.player_turret_angle = 0.0;
        self.player_hp = self.player_max_hp;
        self.score = 0; self.invincible = 2.0;
        self.enemies.clear();
        self.bullets.pool.iter_mut().for_each(|b| b.active = false);
        self.particles.clear();
        self.wave = 0; self.is_boss_wave = false;
        self.start_wave();
    }

    fn start_wave(&mut self) {
        self.wave += 1;
        self.wave_timer = 0.0;
        self.bullets.pool.iter_mut().for_each(|b| if !b.is_player { b.active = false; });
        self.enemies.clear();

        if self.wave >= 6 {
            self.is_boss_wave = true;
            self.boss.spawn();
            self.audio_event = 7; // boss appear
        } else {
            self.is_boss_wave = false;
            let count = (self.wave * 2 + 2) as usize;
            for i in 0..count {
                let angle = TAU * i as f32 / count as f32;
                let r = 14.0 + lcg_f(&mut self.rng) * 3.0;
                let x = angle.sin() * r;
                let z = angle.cos() * r;
                let enemy = if self.wave >= 4 && i % 2 == 0 {
                    BlasterEnemy::new_shooter(x, z)
                } else {
                    BlasterEnemy::new_basic(x, z)
                };
                self.enemies.push(enemy);
            }
        }
    }

    pub fn tick(&mut self, ts: f64) {
        if self.scene != BlasterScene::Playing { return; }
        let dt = if self.prev_ts > 0.0 {
            ((ts - self.prev_ts) * 0.001).min(0.05) as f32
        } else { 0.016 };
        self.prev_ts = ts;
        self.dt = dt;
        self.time = ts * 0.001;
        self.audio_event = 0;

        // ── プレイヤー移動 + 車体回転（移動方向） ──
        if self.input_dx != 0.0 || self.input_dz != 0.0 {
            let spd = 6.0f32;
            let nx = self.player_x + self.input_dx * spd * dt;
            let nz = self.player_z + self.input_dz * spd * dt;
            if nx.abs() < 17.5 { self.player_x = nx; }
            if nz.abs() < 17.5 { self.player_z = nz; }
            // 車体は移動方向へ追従（やや遅め）
            let move_angle = self.input_dx.atan2(self.input_dz);
            let da = angle_diff(move_angle, self.player_body_angle);
            self.player_body_angle += da * (dt * 7.0).min(1.0);
        }

        // ── 砲塔は常に最近の敵に向く（車体と独立・速め） ──
        let aim_angle = self.nearest_enemy_angle();
        let da = angle_diff(aim_angle, self.player_turret_angle);
        self.player_turret_angle += da * (dt * 12.0).min(1.0);

        // ── 自動ショット（砲塔の向きで発射） ──
        self.shoot_timer -= dt;
        if self.input_shoot && self.shoot_timer <= 0.0 {
            self.shoot_timer = 0.12;
            let (s, c) = (self.player_turret_angle.sin(), self.player_turret_angle.cos());
            let spd = 18.0f32;
            self.bullets.spawn(
                self.player_x, 0.5, self.player_z,
                s * spd, 0.0, c * spd, 2.5, true, true, [0.3, 0.8, 1.0],
            );
            self.audio_event = 1;
        }

        self.bullets.tick(dt);
        // ホーミング弾のターゲットリストを作成
        let mut targets: Vec<(f32,f32)> = self.enemies.iter()
            .filter(|e| e.active).map(|e| (e.x, e.z)).collect();
        if self.boss.active { targets.push((self.boss.x, self.boss.z)); }
        self.bullets.update_homing(&targets, dt);
        self.invincible -= dt;

        self.update_enemies(dt);
        self.update_boss(dt);
        self.check_collisions();

        for p in &mut self.particles { p.tick(dt); }
        self.particles.retain(|p| p.life > 0.0);

        // ── ウェーブ終了チェック ──
        if !self.is_boss_wave {
            let alive = self.enemies.iter().filter(|e| e.active).count();
            if alive == 0 {
                self.wave_timer += dt;
                if self.wave_timer > 2.0 {
                    self.start_wave();
                }
            }
        }

        // ── 描画 ──
        let (verts, idxs) = build_blaster_scene(self);
        let asp = self.gpu.width as f32 / self.gpu.height as f32;
        let uni = build_blaster_uni(self, asp);
        self.gpu.render(&verts, &idxs, &uni);
    }

    fn update_enemies(&mut self, dt: f32) {
        let px = self.player_x;
        let pz = self.player_z;
        let mut new_bullets: Vec<(f32, f32, f32, f32, f32, f32, [f32; 3])> = Vec::new();

        for enemy in &mut self.enemies {
            if !enemy.active { continue; }
            let dx = px - enemy.x;
            let dz = pz - enemy.z;
            let dist = (dx*dx + dz*dz).sqrt().max(0.01);
            let dir_x = dx / dist;
            let dir_z = dz / dist;

            match enemy.kind {
                EnemyKind::Basic => {
                    let spd = 3.0f32;
                    enemy.x += dir_x * spd * dt;
                    enemy.z += dir_z * spd * dt;
                    // 車体は移動方向（＝プレイヤー方向）
                    let ba = dir_x.atan2(dir_z);
                    let da = angle_diff(ba, enemy.body_angle);
                    enemy.body_angle += da * (dt * 5.0).min(1.0);
                }
                EnemyKind::Shooter => {
                    let target_dist = 5.0f32;
                    let radial = (dist - target_dist) * 2.0;
                    let tan_x = -dir_z;
                    let tan_z = dir_x;
                    let mx = dir_x * radial + tan_x * 2.0;
                    let mz = dir_z * radial + tan_z * 2.0;
                    enemy.x += mx * dt;
                    enemy.z += mz * dt;
                    // 車体は実際の移動方向
                    let mlen = (mx*mx+mz*mz).sqrt();
                    if mlen > 0.1 {
                        let ba = (mx/mlen).atan2(mz/mlen);
                        let da = angle_diff(ba, enemy.body_angle);
                        enemy.body_angle += da * (dt * 4.0).min(1.0);
                    }
                    enemy.shoot_timer -= dt;
                    if enemy.shoot_timer <= 0.0 {
                        enemy.shoot_timer = 2.0;
                        let spd = 8.0f32;
                        new_bullets.push((enemy.x, 0.5, enemy.z,
                                          dir_x * spd, 0.0, dir_z * spd,
                                          [1.0, 0.2, 0.8]));
                    }
                }
            }
            // 砲塔は常にプレイヤー方向（速く回転）
            let ta = dir_x.atan2(dir_z);
            let da = angle_diff(ta, enemy.turret_angle);
            enemy.turret_angle += da * (dt * 10.0).min(1.0);
            enemy.x = enemy.x.clamp(-17.5, 17.5);
            enemy.z = enemy.z.clamp(-17.5, 17.5);
        }

        for (x, y, z, vx, vy, vz, col) in new_bullets {
            self.bullets.spawn(x, y, z, vx, vy, vz, 3.0, false, false, col);
        }
    }

    fn update_boss(&mut self, dt: f32) {
        if !self.boss.active { return; }
        let px = self.player_x; let pz = self.player_z;
        let b = &mut self.boss;
        let prev_x = b.x; let prev_z = b.z;
        b.move_angle += dt * 0.8;
        b.x = b.move_angle.sin() * 9.0;
        b.z = b.move_angle.cos() * 9.0 - 3.0;
        // 車体は実際の移動方向
        let mvx = b.x - prev_x; let mvz = b.z - prev_z;
        if mvx*mvx + mvz*mvz > 0.0001 {
            let ba = mvx.atan2(mvz);
            let da = angle_diff(ba, b.body_angle);
            b.body_angle += da * (dt * 4.0).min(1.0);
        }
        // 砲塔はプレイヤー方向
        let ta = (px - b.x).atan2(pz - b.z);
        let da = angle_diff(ta, b.turret_angle);
        b.turret_angle += da * (dt * 6.0).min(1.0);

        b.pattern_angle += dt * match b.phase {
            BossPhase::Phase1 => 1.5,
            BossPhase::Phase2 => 2.5,
            BossPhase::Phase3 => 4.0,
            BossPhase::Dead   => 0.0,
        };
        b.shoot_timer -= dt;
        let interval = match b.phase {
            BossPhase::Phase1 => 0.35,
            BossPhase::Phase2 => 0.20,
            BossPhase::Phase3 => 0.12,
            BossPhase::Dead   => 999.0,
        };
        if b.shoot_timer <= 0.0 && b.phase != BossPhase::Dead {
            b.shoot_timer = interval;
            let count = match b.phase {
                BossPhase::Phase1 => 6u32,
                BossPhase::Phase2 => 10,
                BossPhase::Phase3 => 16,
                BossPhase::Dead   => 0,
            };
            let bx  = b.x; let bz  = b.z;
            let ang = b.pattern_angle;
            let spd = match b.phase {
                BossPhase::Phase1 => 6.0f32, BossPhase::Phase2 => 8.0,
                BossPhase::Phase3 => 10.0,   BossPhase::Dead   => 0.0,
            };
            let mut spawns: Vec<(f32, f32)> = Vec::new();
            for i in 0..count {
                let a = ang + TAU * i as f32 / count as f32;
                spawns.push((a.sin() * spd, a.cos() * spd));
            }
            for (vx, vz) in spawns {
                self.bullets.spawn(bx, 0.5, bz, vx, 0.0, vz, 3.5, false, false, [1.0, 0.5, 0.0]);
            }
        }
    }

    fn check_collisions(&mut self) {
        let px = self.player_x;
        let pz = self.player_z;

        // ── 自機弾 vs 敵 ──
        let mut score_gain = 0u32;
        let mut audio: u8 = 0;
        let mut kill_positions: Vec<(f32, f32, f32)> = Vec::new();

        for bullet in &mut self.bullets.pool {
            if !bullet.active || !bullet.is_player { continue; }
            for enemy in &mut self.enemies {
                if !enemy.active { continue; }
                let dx = bullet.x - enemy.x;
                let dz = bullet.z - enemy.z;
                if dx*dx + dz*dz < 0.4 {
                    bullet.active = false;
                    enemy.hp -= 1;
                    audio = 3;
                    if enemy.hp <= 0 {
                        enemy.active = false;
                        score_gain += 100;
                        audio = 4;
                        kill_positions.push((enemy.x, enemy.y, enemy.z));
                    }
                    break;
                }
            }
        }
        self.score += score_gain;
        if audio > 0 { self.audio_event = audio; }
        let enemy_col = [1.0f32, 0.5, 0.1];
        for (x, y, z) in kill_positions {
            self.spawn_explosion(x, y, z, enemy_col);
        }

        // ── 自機弾 vs ボス ──
        if self.boss.active {
            let bx = self.boss.x;
            let bz = self.boss.z;
            let mut boss_hit = false;
            for bullet in &mut self.bullets.pool {
                if !bullet.active || !bullet.is_player { continue; }
                let dx = bullet.x - bx;
                let dz = bullet.z - bz;
                if dx*dx + dz*dz < 2.0 {
                    bullet.active = false;
                    boss_hit = true;
                }
            }
            if boss_hit {
                self.boss.hp -= 1;
                self.boss.update_phase();
                self.audio_event = 3;
                if self.boss.phase == BossPhase::Dead {
                    let (bx2, by, bz2) = (self.boss.x, self.boss.y, self.boss.z);
                    self.boss.active = false;
                    self.score += 5000;
                    self.scene = BlasterScene::BossClear;
                    self.audio_event = 5;
                    self.spawn_explosion(bx2, by, bz2, [1.0, 0.8, 0.0]);
                }
            }
        }

        // ── 敵弾 / 敵接触 vs 自機 ──
        if self.invincible <= 0.0 {
            let mut player_hit = false;
            for bullet in &mut self.bullets.pool {
                if !bullet.active || bullet.is_player { continue; }
                let dx = bullet.x - px;
                let dz = bullet.z - pz;
                if dx*dx + dz*dz < 0.3 {
                    bullet.active = false;
                    player_hit = true;
                }
            }
            // 敵との接触
            let mut enemy_touch = false;
            for enemy in &self.enemies {
                if !enemy.active { continue; }
                let dx = enemy.x - px;
                let dz = enemy.z - pz;
                if dx*dx + dz*dz < 0.6 {
                    enemy_touch = true;
                    break;
                }
            }
            if player_hit || enemy_touch {
                self.player_hp -= 1;
                self.invincible = 1.5;
                self.audio_event = 6;
                if self.player_hp <= 0 {
                    self.scene = BlasterScene::GameOver;
                    self.audio_event = 8;
                }
            }
        }
    }

    fn spawn_explosion(&mut self, x: f32, y: f32, z: f32, col: [f32; 3]) {
        for i in 0..16 {
            let a = TAU * i as f32 / 16.0;
            let spd = 3.0 + lcg_f(&mut self.rng) * 4.0;
            let r = lcg_f(&mut self.rng);
            self.particles.push(BlasterParticle {
                x, y: y + r * 0.5, z,
                vx: a.sin() * spd,
                vy: (r - 0.5) * 4.0,
                vz: a.cos() * spd,
                life: 0.5 + r * 0.8,
                max_life: 1.3,
                col,
            });
        }
    }

    pub fn set_move(&mut self, dx: f32, dz: f32)  { self.input_dx = dx; self.input_dz = dz; }
    pub fn set_shoot(&mut self, on: bool)           { self.input_shoot = on; }
    pub fn switch_camera(&mut self)                 { self.camera = self.camera.next(); }
    pub fn scene_u8(&self) -> u8                    { self.scene as u8 }
    pub fn camera_u8(&self) -> u8                   { self.camera.as_u8() }
    pub fn camera_name(&self) -> &str               { self.camera.name() }
    pub fn bullet_count(&self) -> u32               { self.bullets.active_count() as u32 }

    /// 最も近い敵/ボスへの角度を返す（いなければ現在の角度を維持）
    fn nearest_enemy_angle(&self) -> f32 {
        let px = self.player_x;
        let pz = self.player_z;
        let mut best_dist2 = f32::MAX;
        let mut best_angle = self.player_turret_angle;

        // 敵を探す
        for enemy in &self.enemies {
            if !enemy.active { continue; }
            let dx = enemy.x - px;
            let dz = enemy.z - pz;
            let d2 = dx * dx + dz * dz;
            if d2 < best_dist2 {
                best_dist2 = d2;
                best_angle = dx.atan2(dz);
            }
        }

        // ボス（敵がいないかボスの方が近い場合）
        if self.boss.active {
            let dx = self.boss.x - px;
            let dz = self.boss.z - pz;
            let d2 = dx * dx + dz * dz;
            if d2 < best_dist2 {
                best_angle = dx.atan2(dz);
            }
        }

        best_angle
    }
}

fn angle_diff(target: f32, current: f32) -> f32 {
    let mut d = target - current;
    while d >  PI { d -= TAU; }
    while d < -PI { d += TAU; }
    d
}
