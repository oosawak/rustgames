#![allow(dead_code)]

use wasm_bindgen::JsCast;
use std::f32::consts::{PI, TAU};
use crate::math::lcg_f;
use crate::gpu::GpuState;
use super::geometry::{build_scene, build_uni};

#[derive(PartialEq, Clone, Copy)]
pub enum EarthDefScene { Title = 0, Playing = 1, GameOver = 2 }

#[derive(Clone, Copy, PartialEq)]
pub enum LaserType { Beam = 0, Spread = 1, Reflect = 2 }

#[derive(Clone, Copy, PartialEq)]
pub enum EnemyKind { Basic, Speed, Armored, Splitter }

pub struct Enemy {
    pub active: bool,
    pub kind: EnemyKind,
    pub x: f32, pub y: f32, pub z: f32,
    pub hp: i32,
    pub speed: f32,
    pub rot_x: f32, pub rot_y: f32, pub rot_z: f32,
    pub hit_flash: f32,
    pub scale: f32,
}

pub struct Particle {
    pub x: f32, pub y: f32, pub z: f32,
    pub vx: f32, pub vy: f32, pub vz: f32,
    pub life: f32, pub max_life: f32,
    pub col: [f32; 4],
}

pub struct BeamVis {
    pub ox: f32, pub oy: f32, pub oz: f32,
    pub dx: f32, pub dy: f32, pub dz: f32,
    pub len: f32,
    pub life: f32,
    pub col: [f32; 4],
}

pub struct EarthDefGame {
    pub gpu: GpuState,
    pub scene: EarthDefScene,
    pub time: f32,
    pub dt: f32,
    pub prev_ts: f64,

    pub earth_hp: i32,
    pub earth_max_hp: i32,
    pub earth_rot: f32,
    pub earth_hit_flash: f32,

    pub cam_azimuth: f32,
    pub cam_elevation: f32,
    pub cam_distance: f32,

    pub aim_azimuth: f32,
    pub aim_elevation: f32,

    pub laser_type: LaserType,
    pub laser_active: bool,
    pub laser_timer: f32,

    pub flash_charges: u32,
    pub flash_max_charges: u32,
    pub flash_recharge_timer: f32,
    pub flash_active: f32,

    pub cam_input_x: f32,
    pub cam_input_y: f32,
    pub aim_input_x: f32,
    pub aim_input_y: f32,
    pub fire_input: bool,

    pub chain_point: Option<[f32; 3]>,

    pub enemies: Vec<Enemy>,
    pub particles: Vec<Particle>,
    pub beams: Vec<BeamVis>,
    pub score: u32,
    pub wave: u32,
    pub kills_in_wave: u32,
    pub wave_timer: f32,
    pub spawn_timer: f32,
    pub audio_event: u8,
    pub rng: u64,
}

impl Enemy {
    fn new(kind: EnemyKind, x: f32, y: f32, z: f32) -> Self {
        let (hp, speed, scale) = match kind {
            EnemyKind::Basic    => (1, 1.5f32, 0.7f32),
            EnemyKind::Speed    => (1, 3.0,    0.45),
            EnemyKind::Armored  => (3, 1.0,    1.1),
            EnemyKind::Splitter => (1, 2.0,    0.65),
        };
        Enemy {
            active: true, kind, x, y, z,
            hp, speed,
            rot_x: 0.0, rot_y: 0.0, rot_z: 0.0,
            hit_flash: 0.0, scale,
        }
    }
}

impl EarthDefGame {
    pub async fn new(canvas_id: &str) -> Result<Self, String> {
        let canvas = web_sys::window().unwrap()
            .document().unwrap()
            .get_element_by_id(canvas_id).unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| "canvas cast failed".to_string())?;
        let gpu = GpuState::new(canvas).await?;
        Ok(Self {
            gpu,
            scene: EarthDefScene::Title,
            time: 0.0, dt: 0.016, prev_ts: 0.0,
            earth_hp: 100, earth_max_hp: 100,
            earth_rot: 0.0, earth_hit_flash: 0.0,
            cam_azimuth: 0.0, cam_elevation: 0.3, cam_distance: 7.0,
            aim_azimuth: 0.0, aim_elevation: 0.0,
            laser_type: LaserType::Beam, laser_active: false, laser_timer: 0.0,
            flash_charges: 3, flash_max_charges: 3,
            flash_recharge_timer: 0.0, flash_active: 0.0,
            cam_input_x: 0.0, cam_input_y: 0.0,
            aim_input_x: 0.0, aim_input_y: 0.0,
            fire_input: false,
            chain_point: None,
            enemies: Vec::new(), particles: Vec::new(), beams: Vec::new(),
            score: 0, wave: 1, kills_in_wave: 0,
            wave_timer: 0.0, spawn_timer: 0.0,
            audio_event: 0, rng: 0x1234567890abcdef,
        })
    }

    pub fn start(&mut self) {
        self.scene = EarthDefScene::Playing;
        self.earth_hp = 100;
        self.earth_rot = 0.0;
        self.earth_hit_flash = 0.0;
        self.cam_azimuth = 0.0;
        self.cam_elevation = 0.3;
        self.cam_distance  = 7.0;
        self.aim_azimuth = 0.0;
        self.aim_elevation = 0.0;
        self.laser_type = LaserType::Beam;
        self.laser_timer = 0.0;
        self.flash_charges = 3;
        self.flash_recharge_timer = 0.0;
        self.flash_active = 0.0;
        self.chain_point = None;
        self.particles.clear();
        self.beams.clear();
        self.score = 0;
        self.wave = 1;
        self.kills_in_wave = 0;
        self.spawn_timer = 0.0;
        self.audio_event = 0;
    }

    pub fn tick(&mut self, ts: f64) {
        // Sync GPU surface to canvas size every frame
        {
            let win = web_sys::window().unwrap();
            let w = win.inner_width().unwrap().as_f64().unwrap() as u32;
            let h = win.inner_height().unwrap().as_f64().unwrap() as u32;
            self.gpu.resize(w.max(1), h.max(1));
        }

        let dt = if self.prev_ts > 0.0 {
            ((ts - self.prev_ts) * 0.001).min(0.05) as f32
        } else {
            0.016
        };
        self.prev_ts = ts;
        self.dt = dt;
        self.time += dt;

        if self.scene != EarthDefScene::Playing {
            let (verts, idxs) = build_scene(self);
            let asp = self.gpu.width as f32 / self.gpu.height as f32;
            let uni = build_uni(self, asp);
            self.gpu.render(&verts, &idxs, &uni);
            return;
        }

        self.audio_event = 0;
        self.earth_rot += dt * 0.4;
        if self.earth_hit_flash > 0.0 {
            self.earth_hit_flash = (self.earth_hit_flash - dt * 3.0).max(0.0);
        }

        // Camera
        self.cam_azimuth += self.cam_input_x * dt * 1.5;
        self.cam_elevation = (self.cam_elevation + self.cam_input_y * dt * 1.5).clamp(-1.4, 1.4);

        // Aim
        self.aim_azimuth += self.aim_input_x * dt * 2.0;
        self.aim_elevation = (self.aim_elevation + self.aim_input_y * dt * 2.0).clamp(-1.5, 1.5);

        if self.laser_timer > 0.0 { self.laser_timer = (self.laser_timer - dt).max(0.0); }
        if self.flash_active > 0.0 { self.flash_active = (self.flash_active - dt * 2.0).max(0.0); }

        // Flash recharge
        self.flash_recharge_timer += dt;
        if self.flash_recharge_timer >= 15.0 && self.flash_charges < self.flash_max_charges {
            self.flash_charges += 1;
            self.flash_recharge_timer = 0.0;
        }

        // Update enemies
        let mut earth_damage = 0i32;
        for e in &mut self.enemies {
            if !e.active { continue; }
            let dist = (e.x*e.x + e.y*e.y + e.z*e.z).sqrt();
            if dist < 0.9 {
                e.active = false;
                earth_damage += 10;
            } else {
                let inv = 1.0 / dist;
                e.x -= e.x * inv * e.speed * dt;
                e.y -= e.y * inv * e.speed * dt;
                e.z -= e.z * inv * e.speed * dt;
                e.rot_y += dt * (1.0 + e.speed * 0.3);
                e.rot_x += dt * 0.7;
                if e.hit_flash > 0.0 { e.hit_flash = (e.hit_flash - dt * 3.0).max(0.0); }
            }
        }
        if earth_damage > 0 {
            self.earth_hp = (self.earth_hp - earth_damage).max(0);
            self.earth_hit_flash = 1.0;
            self.audio_event = 3;
        }
        if self.earth_hp <= 0 { self.scene = EarthDefScene::GameOver; }

        // Spawn enemies
        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            let rate = (2.5 - self.wave as f32 * 0.15).max(0.5);
            self.spawn_timer = rate;
            let new_e = self.make_enemy();
            self.enemies.push(new_e);
        }

        // Update beams (drain pattern for safety)
        self.beams = self.beams.drain(..).filter_map(|mut b| {
            b.life -= dt;
            if b.life > 0.0 { Some(b) } else { None }
        }).collect();

        // Update particles
        self.particles = self.particles.drain(..).filter_map(|mut p| {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.z += p.vz * dt;
            p.vy -= dt * 0.5;
            p.life -= dt;
            if p.life > 0.0 { Some(p) } else { None }
        }).collect();

        // Render
        let (verts, idxs) = build_scene(self);
        let asp = self.gpu.width as f32 / self.gpu.height as f32;
        let uni = build_uni(self, asp);
        self.gpu.render(&verts, &idxs, &uni);
    }

    fn make_enemy(&mut self) -> Enemy {
        let theta = lcg_f(&mut self.rng) * TAU;
        let phi = lcg_f(&mut self.rng) * PI - PI * 0.5;
        let r = 22.0 + lcg_f(&mut self.rng) * 3.0;
        let x = phi.cos() * theta.sin() * r;
        let y = phi.sin() * r;
        let z = phi.cos() * theta.cos() * r;

        let rk = lcg_f(&mut self.rng);
        let kind = if self.wave >= 4 {
            if rk < 0.40 { EnemyKind::Basic }
            else if rk < 0.65 { EnemyKind::Speed }
            else if rk < 0.85 { EnemyKind::Armored }
            else { EnemyKind::Splitter }
        } else if self.wave >= 3 {
            if rk < 0.50 { EnemyKind::Basic }
            else if rk < 0.80 { EnemyKind::Speed }
            else { EnemyKind::Armored }
        } else if self.wave >= 2 {
            if rk < 0.70 { EnemyKind::Basic } else { EnemyKind::Speed }
        } else {
            EnemyKind::Basic
        };

        Enemy::new(kind, x, y, z)
    }

    pub fn set_cam_input(&mut self, x: f32, y: f32) {
        self.cam_input_x = x;
        self.cam_input_y = y;
    }

    pub fn set_aim_input(&mut self, x: f32, y: f32) {
        self.aim_input_x = x;
        self.aim_input_y = y;
    }

    pub fn fire(&mut self) {
        if self.scene != EarthDefScene::Playing { return; }
        if self.laser_timer > 0.0 { return; }
        self.fire_laser();
        self.laser_timer = 0.3;
        self.audio_event = 1;
    }

    fn fire_laser(&mut self) {
        let ae = self.aim_elevation;
        let aa = self.aim_azimuth;
        let dx = ae.cos() * aa.sin();
        let dy = ae.sin();
        let dz = ae.cos() * aa.cos();

        let col: [f32; 4] = match self.laser_type {
            LaserType::Beam    => [0.0, 1.0, 1.0, 1.0],
            LaserType::Spread  => [0.8, 1.0, 0.2, 1.0],
            LaserType::Reflect => [1.0, 0.0, 1.0, 1.0],
        };

        match self.laser_type {
            LaserType::Beam => {
                let t = self.ray_vs_enemies(0.0, 0.0, 0.0, dx, dy, dz);
                self.beams.push(BeamVis { ox:0.0, oy:0.0, oz:0.0, dx, dy, dz, len:t, life:0.3, col });
            }
            LaserType::Spread => {
                for offset in [-0.25f32, 0.0, 0.25] {
                    let az2 = aa + offset;
                    let dx2 = ae.cos() * az2.sin();
                    let dz2 = ae.cos() * az2.cos();
                    let t = self.ray_vs_enemies(0.0, 0.0, 0.0, dx2, dy, dz2);
                    self.beams.push(BeamVis { ox:0.0, oy:0.0, oz:0.0, dx:dx2, dy, dz:dz2, len:t, life:0.3, col });
                }
            }
            LaserType::Reflect => {
                let t1 = self.ray_vs_enemies(0.0, 0.0, 0.0, dx, dy, dz);
                self.beams.push(BeamVis { ox:0.0, oy:0.0, oz:0.0, dx, dy, dz, len:t1, life:0.4, col });
                if t1 >= 20.0 {
                    let h1x = dx * 20.0;
                    let h1y = dy * 20.0;
                    let h1z = dz * 20.0;
                    // Reflect: n = normalize(h1) = d, reflected = d - 2*dot(d,d)*d = -d
                    let rdx = -dx; let rdy = -dy; let rdz = -dz;
                    let t2 = self.ray_vs_enemies(h1x, h1y, h1z, rdx, rdy, rdz);
                    self.beams.push(BeamVis { ox:h1x, oy:h1y, oz:h1z, dx:rdx, dy:rdy, dz:rdz, len:t2, life:0.4, col });
                    if t2 >= 20.0 {
                        let h2x = h1x + rdx * 20.0;
                        let h2y = h1y + rdy * 20.0;
                        let h2z = h1z + rdz * 20.0;
                        let t3 = self.ray_vs_enemies(h2x, h2y, h2z, dx, dy, dz);
                        self.beams.push(BeamVis { ox:h2x, oy:h2y, oz:h2z, dx, dy, dz, len:t3, life:0.4, col });
                    }
                }
            }
        }
    }

    fn ray_vs_enemies(&mut self, ox: f32, oy: f32, oz: f32, dx: f32, dy: f32, dz: f32) -> f32 {
        let mut best_t = 25.0f32;
        let mut new_enemies: Vec<Enemy> = Vec::new();
        let mut scored = 0u32;
        let mut kill_count = 0u32;
        let mut got_hit = false;
        let mut wave_up = false;

        // Collect explosion particles separately to avoid borrow issues
        let mut new_particles: Vec<Particle> = Vec::new();

        for e in &mut self.enemies {
            if !e.active { continue; }
            let rel = [e.x - ox, e.y - oy, e.z - oz];
            let t = rel[0]*dx + rel[1]*dy + rel[2]*dz;
            if t < 0.0 { continue; }
            let cx = rel[0] - t*dx;
            let cy = rel[1] - t*dy;
            let cz = rel[2] - t*dz;
            let perp2 = cx*cx + cy*cy + cz*cz;
            let r = 0.7 * e.scale;
            if perp2 < r*r {
                if t < best_t { best_t = t; }
                e.hp -= 1;
                e.hit_flash = 1.0;
                got_hit = true;
                if e.hp <= 0 {
                    let pts: u32 = match e.kind {
                        EnemyKind::Basic    => 10,
                        EnemyKind::Speed    => 20,
                        EnemyKind::Armored  => 50,
                        EnemyKind::Splitter => 30,
                    };
                    scored += pts;
                    kill_count += 1;
                    let ecol: [f32; 4] = match e.kind {
                        EnemyKind::Basic    => [0.2, 0.8, 0.3, 1.0],
                        EnemyKind::Speed    => [0.9, 0.9, 0.1, 1.0],
                        EnemyKind::Armored  => [0.8, 0.2, 0.2, 1.0],
                        EnemyKind::Splitter => [0.1, 0.9, 0.9, 1.0],
                    };
                    if e.kind == EnemyKind::Splitter {
                        new_enemies.push(Enemy::new(EnemyKind::Speed, e.x + 0.6, e.y, e.z));
                        new_enemies.push(Enemy::new(EnemyKind::Speed, e.x - 0.6, e.y, e.z));
                    }
                    let (ex, ey, ez) = (e.x, e.y, e.z);
                    for _ in 0..10 {
                        let vx = (lcg_f(&mut self.rng) - 0.5) * 6.0;
                        let vy = (lcg_f(&mut self.rng) - 0.5) * 6.0;
                        let vz = (lcg_f(&mut self.rng) - 0.5) * 6.0;
                        let life = 0.4 + lcg_f(&mut self.rng) * 0.6;
                        new_particles.push(Particle { x:ex, y:ey, z:ez, vx, vy, vz, life, max_life:life, col:ecol });
                    }
                    e.active = false;
                }
            }
        }

        if got_hit && self.audio_event < 2 { self.audio_event = 2; }
        self.score += scored;
        self.kills_in_wave += kill_count;
        self.particles.extend(new_particles);
        self.enemies.extend(new_enemies);

        // Wave progression
        if self.kills_in_wave >= 20 {
            self.wave += 1;
            self.kills_in_wave = 0;
            wave_up = true;
        }
        if wave_up && self.audio_event == 0 { self.audio_event = 5; }

        best_t
    }

    pub fn flash_bomb(&mut self) {
        if self.scene != EarthDefScene::Playing { return; }
        if self.flash_charges == 0 { return; }
        self.flash_charges -= 1;
        self.flash_recharge_timer = 0.0;
        self.flash_active = 1.0;
        self.audio_event = 4;

        let mut bonus = 0u32;
        let mut new_particles: Vec<Particle> = Vec::new();

        for e in &mut self.enemies {
            if !e.active { continue; }
            bonus += match e.kind {
                EnemyKind::Basic    => 10,
                EnemyKind::Speed    => 20,
                EnemyKind::Armored  => 50,
                EnemyKind::Splitter => 30,
            };
            let col: [f32; 4] = match e.kind {
                EnemyKind::Basic    => [0.2, 0.8, 0.3, 1.0],
                EnemyKind::Speed    => [0.9, 0.9, 0.1, 1.0],
                EnemyKind::Armored  => [0.8, 0.2, 0.2, 1.0],
                EnemyKind::Splitter => [0.1, 0.9, 0.9, 1.0],
            };
            for _ in 0..8 {
                let vx = (lcg_f(&mut self.rng) - 0.5) * 8.0;
                let vy = (lcg_f(&mut self.rng) - 0.5) * 8.0;
                let vz = (lcg_f(&mut self.rng) - 0.5) * 8.0;
                let life = 0.5 + lcg_f(&mut self.rng) * 0.8;
                new_particles.push(Particle { x:e.x, y:e.y, z:e.z, vx, vy, vz, life, max_life:life, col });
            }
            e.active = false;
        }
        self.score += bonus;
        self.particles.extend(new_particles);
    }

    pub fn set_laser_type(&mut self, t: u8) {
        self.laser_type = match t {
            1 => LaserType::Spread,
            2 => LaserType::Reflect,
            _ => LaserType::Beam,
        };
    }

    // ─── Tap-to-fire ────────────────────────────────────────────────────────
    pub fn fire_at_screen(&mut self, nx: f32, ny: f32) {
        if self.scene != EarthDefScene::Playing { return; }
        if self.laser_timer > 0.0 { return; }

        let asp = self.gpu.width as f32 / self.gpu.height as f32;

        // Chain origin: last hit point while beams still visible
        let (ox, oy, oz) = if let Some(cp) = self.chain_point {
            if !self.beams.is_empty() { (cp[0], cp[1], cp[2]) } else { (0.0, 0.0, 0.0) }
        } else { (0.0, 0.0, 0.0) };

        let col: [f32; 4] = match self.laser_type {
            LaserType::Beam    => [0.0, 1.0, 1.0, 1.0],
            LaserType::Spread  => [0.8, 1.0, 0.2, 1.0],
            LaserType::Reflect => [1.0, 0.0, 1.0, 1.0],
        };

        match self.laser_type {
            LaserType::Spread => {
                let (dx, dy, dz) = self.tap_to_dir(nx, ny, asp);
                let az = dx.atan2(dz);
                let el = dy.asin();
                let mut hit_pos: Option<[f32; 3]> = None;
                for offset in [-0.2f32, 0.0, 0.2] {
                    let az2 = az + offset;
                    let dx2 = el.cos() * az2.sin();
                    let dz2 = el.cos() * az2.cos();
                    let t = self.ray_vs_enemies(ox, oy, oz, dx2, dy, dz2);
                    self.beams.push(BeamVis { ox, oy, oz, dx: dx2, dy, dz: dz2, len: t, life: 0.25, col });
                    if t < 24.9 && hit_pos.is_none() {
                        hit_pos = Some([ox + dx2*t, oy + dy*t, oz + dz2*t]);
                    }
                }
                self.chain_point = hit_pos;
            }
            LaserType::Reflect => {
                let (dx, dy, dz) = if let Some(idx) = self.find_enemy_at_screen(nx, ny, asp) {
                    let (ex, ey, ez) = (self.enemies[idx].x - ox, self.enemies[idx].y - oy, self.enemies[idx].z - oz);
                    let l = (ex*ex + ey*ey + ez*ez).sqrt().max(0.001);
                    (ex/l, ey/l, ez/l)
                } else { self.tap_to_dir(nx, ny, asp) };
                let t1 = self.ray_vs_enemies(ox, oy, oz, dx, dy, dz);
                self.beams.push(BeamVis { ox, oy, oz, dx, dy, dz, len: t1, life: 0.35, col });
                if t1 < 24.9 {
                    let hit = [ox + dx*t1, oy + dy*t1, oz + dz*t1];
                    let hl = (hit[0]*hit[0] + hit[1]*hit[1] + hit[2]*hit[2]).sqrt().max(0.001);
                    let n = [hit[0]/hl, hit[1]/hl, hit[2]/hl];
                    let dot = dx*n[0] + dy*n[1] + dz*n[2];
                    let (rdx, rdy, rdz) = (dx - 2.0*dot*n[0], dy - 2.0*dot*n[1], dz - 2.0*dot*n[2]);
                    let t2 = self.ray_vs_enemies(hit[0], hit[1], hit[2], rdx, rdy, rdz);
                    self.beams.push(BeamVis { ox: hit[0], oy: hit[1], oz: hit[2], dx: rdx, dy: rdy, dz: rdz, len: t2, life: 0.35, col });
                    self.chain_point = Some([hit[0]+rdx*t2, hit[1]+rdy*t2, hit[2]+rdz*t2]);
                } else {
                    self.chain_point = None;
                }
            }
            LaserType::Beam => {
                let (dx, dy, dz) = if let Some(idx) = self.find_enemy_at_screen(nx, ny, asp) {
                    let (ex, ey, ez) = (self.enemies[idx].x - ox, self.enemies[idx].y - oy, self.enemies[idx].z - oz);
                    let l = (ex*ex + ey*ey + ez*ez).sqrt().max(0.001);
                    (ex/l, ey/l, ez/l)
                } else { self.tap_to_dir(nx, ny, asp) };
                let t = self.ray_vs_enemies(ox, oy, oz, dx, dy, dz);
                self.beams.push(BeamVis { ox, oy, oz, dx, dy, dz, len: t, life: 0.25, col });
                self.chain_point = if t < 24.9 {
                    Some([ox + dx*t, oy + dy*t, oz + dz*t])
                } else { None };
            }
        }

        self.laser_timer = 0.08;
        self.audio_event = 1;
    }

    /// Nearest enemy (index) to the camera ray through tap (nx, ny) in NDC.
    fn find_enemy_at_screen(&self, nx: f32, ny: f32, asp: f32) -> Option<usize> {
        let (ray_ox, ray_oy, ray_oz, ray_dx, ray_dy, ray_dz) = self.camera_ray(nx, ny, asp);
        let mut best_idx = None;
        let mut best_ang = 0.30f32; // ~17° tolerance
        for (i, e) in self.enemies.iter().enumerate() {
            if !e.active { continue; }
            let rel = [e.x - ray_ox, e.y - ray_oy, e.z - ray_oz];
            let t = rel[0]*ray_dx + rel[1]*ray_dy + rel[2]*ray_dz;
            if t < 0.5 { continue; }
            let cx = rel[0] - t*ray_dx;
            let cy = rel[1] - t*ray_dy;
            let cz = rel[2] - t*ray_dz;
            let ang = ((cx*cx + cy*cy + cz*cz).sqrt() / t).atan();
            if ang < best_ang { best_ang = ang; best_idx = Some(i); }
        }
        best_idx
    }

    /// Camera ray: returns (origin_xyz, dir_xyz) for NDC tap point.
    fn camera_ray(&self, nx: f32, ny: f32, asp: f32) -> (f32, f32, f32, f32, f32, f32) {
        let az = self.cam_azimuth;
        let el = self.cam_elevation;
        let dist = self.cam_distance;
        let eye = [az.sin()*el.cos()*dist, el.sin()*dist, az.cos()*el.cos()*dist];
        let elen = (eye[0]*eye[0] + eye[1]*eye[1] + eye[2]*eye[2]).sqrt().max(0.001);
        let fwd = [-eye[0]/elen, -eye[1]/elen, -eye[2]/elen];
        // right = fwd × (0,1,0) = (-fwd.z, 0, fwd.x)
        let rlen = (fwd[2]*fwd[2] + fwd[0]*fwd[0]).sqrt().max(0.001);
        let right = [-fwd[2]/rlen, 0.0f32, fwd[0]/rlen];
        // cam_up = right × fwd
        let ux = right[1]*fwd[2] - right[2]*fwd[1];
        let uy = right[2]*fwd[0] - right[0]*fwd[2];
        let uz = right[0]*fwd[1] - right[1]*fwd[0];
        let tf = (PI / 6.0).tan(); // tan(30°) for 60° FOV
        let dx = fwd[0] + nx*asp*tf*right[0] + ny*tf*ux;
        let dy = fwd[1] + nx*asp*tf*right[1] + ny*tf*uy;
        let dz = fwd[2] + nx*asp*tf*right[2] + ny*tf*uz;
        let dl = (dx*dx + dy*dy + dz*dz).sqrt().max(0.001);
        (eye[0], eye[1], eye[2], dx/dl, dy/dl, dz/dl)
    }

    /// Tap direction projected from origin (for miss case).
    fn tap_to_dir(&self, nx: f32, ny: f32, asp: f32) -> (f32, f32, f32) {
        let (_, _, _, dx, dy, dz) = self.camera_ray(nx, ny, asp);
        (dx, dy, dz)
    }
    // ────────────────────────────────────────────────────────────────────────

    pub fn scene_u8(&self) -> u8 { self.scene as u8 }
    pub fn earth_hp(&self) -> i32 { self.earth_hp }
    pub fn earth_max_hp(&self) -> i32 { self.earth_max_hp }
    pub fn score(&self) -> u32 { self.score }
    pub fn wave(&self) -> u32 { self.wave }
    pub fn flash_charges(&self) -> u32 { self.flash_charges }
    pub fn flash_max_charges(&self) -> u32 { self.flash_max_charges }
    pub fn audio_event(&self) -> u8 { self.audio_event }
}
