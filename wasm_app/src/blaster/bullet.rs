#[derive(Clone)]
pub struct Bullet {
    pub active: bool,
    pub x: f32, pub y: f32, pub z: f32,
    pub vx: f32, pub vy: f32, pub vz: f32,
    pub life: f32,
    pub is_player: bool,
    pub homing: bool,
    pub col: [f32; 3],
}

pub const MAX_BULLETS: usize = 256;

pub struct BulletPool {
    pub pool: Vec<Bullet>,
}

impl BulletPool {
    pub fn new() -> Self {
        Self {
            pool: (0..MAX_BULLETS).map(|_| Bullet {
                active: false, x: 0.0, y: 0.5, z: 0.0,
                vx: 0.0, vy: 0.0, vz: 0.0, life: 0.0,
                is_player: false, homing: false, col: [1.0, 1.0, 1.0],
            }).collect(),
        }
    }

    pub fn spawn(&mut self, x: f32, y: f32, z: f32,
                 vx: f32, vy: f32, vz: f32,
                 life: f32, is_player: bool, homing: bool, col: [f32; 3]) -> bool {
        for b in &mut self.pool {
            if !b.active {
                *b = Bullet { active: true, x, y, z, vx, vy, vz, life, is_player, homing, col };
                return true;
            }
        }
        false
    }

    pub fn tick(&mut self, dt: f32) {
        for b in &mut self.pool {
            if !b.active { continue; }
            b.x += b.vx * dt;
            b.y += b.vy * dt;
            b.z += b.vz * dt;
            b.life -= dt;
            if b.life <= 0.0 || b.x.abs() > 20.0 || b.z.abs() > 20.0 {
                b.active = false;
            }
        }
    }

    /// ホーミング弾の方向を目標に向けて補正する（プレイヤー弾のみ）
    pub fn update_homing(&mut self, targets: &[(f32, f32)], dt: f32) {
        for b in &mut self.pool {
            if !b.active || !b.homing { continue; }
            // 最も近いターゲットを探す
            let mut best_dx = 0.0f32;
            let mut best_dz = 0.0f32;
            let mut best_d2 = f32::MAX;
            for &(tx, tz) in targets {
                let dx = tx - b.x;
                let dz = tz - b.z;
                let d2 = dx * dx + dz * dz;
                if d2 < best_d2 { best_d2 = d2; best_dx = dx; best_dz = dz; }
            }
            if best_d2 < f32::MAX {
                let len = best_d2.sqrt().max(0.001);
                let nx = best_dx / len;
                let nz = best_dz / len;
                let spd = (b.vx * b.vx + b.vz * b.vz).sqrt();
                // 現在速度方向をターゲット方向に向けてゆっくり補正
                let turn = (dt * 4.0).min(1.0);
                b.vx = b.vx + (nx * spd - b.vx) * turn;
                b.vz = b.vz + (nz * spd - b.vz) * turn;
                // 速度を一定に保つ
                let new_spd = (b.vx * b.vx + b.vz * b.vz).sqrt().max(0.001);
                b.vx = b.vx / new_spd * spd;
                b.vz = b.vz / new_spd * spd;
            }
        }
    }

    pub fn active_count(&self) -> usize {
        self.pool.iter().filter(|b| b.active).count()
    }
}
