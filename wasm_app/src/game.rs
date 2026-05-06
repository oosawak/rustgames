// ゲーム状態モジュール: プレイヤー移動、レベル管理、パーティクル生成などのゲームロジックを定義する

use wasm_bindgen::JsCast;
use crate::constants::*;
use crate::math::{mat_mul, perspective, look_at, lcg_f};
use crate::maze::Maze;
use crate::particle::Particle;
use crate::geometry::{Light, Uni, LIGHT_COLS, find_lights, build_scene};
use crate::gpu::GpuState;

pub struct GameState {
    pub gpu:         GpuState,
    pub maze:        Maze,
    pub light_pos:   [[f32;4];4],  // 4つのポイントライトのワールド座標
    pub px: usize, pub pz: usize,
    pub facing: u8,
    pub steps: u32,
    pub total_steps: u32,
    pub level: u32,
    pub level_clear: bool,
    pub warp_timer: f32,
    pub particles: Vec<Particle>,
    pub time: f32,
    pub prev_ts: f64,
}

impl GameState {
    pub async fn new(canvas_id: &str) -> Result<Self, String> {
        let doc    = web_sys::window().ok_or("no window")?.document().ok_or("no doc")?;
        let canvas = doc.get_element_by_id(canvas_id).ok_or("no canvas")?
            .dyn_into::<web_sys::HtmlCanvasElement>().map_err(|_| "not canvas")?;
        let mut seed = (js_sys::Math::random() * u64::MAX as f64) as u64 | 1;
        let maze = Maze::new(seed);
        let light_pos = find_lights(&maze, &mut seed);
        Ok(GameState{
            gpu: GpuState::new(canvas).await?,
            maze, light_pos,
            px:0, pz:0, facing:S,
            steps:0, total_steps:0, level:1,
            level_clear:false, warp_timer:0.0,
            particles: Vec::new(),
            time:0.0, prev_ts:0.0,
        })
    }

    pub fn tick(&mut self, ts: f64) {
        let dt = ((ts - self.prev_ts) / 1000.0).clamp(0.0, 0.1) as f32;
        self.prev_ts = ts;
        self.time    = (ts / 1000.0) as f32;

        // パーティクル更新（重力 + フェードアウト）
        for p in &mut self.particles {
            if p.life <= 0.0 { continue; }
            p.pos[0] += p.vel[0];
            p.pos[1] += p.vel[1];
            p.pos[2] += p.vel[2];
            p.vel[1]  -= 0.003; // 重力
            p.life    -= dt * 1.8;
            if p.life < 0.0 { p.life = 0.0; }
        }
        // 死んだパーティクルを定期的に削除
        if self.particles.len() > 200 {
            self.particles.retain(|p| p.life > 0.0);
        }

        // ワープタイマー更新
        if self.level_clear { self.warp_timer += dt; }

        let warp = self.warp_amount();
        let (verts,idxs) = build_scene(&self.maze, self.time, &self.particles, &self.light_pos);

        let center = [self.px as f32+0.5, EYE_H, self.pz as f32+0.5];
        let fwd:[f32;3] = match self.facing {
            d if d==N=>[0.0,0.0,-1.0], d if d==S=>[0.0,0.0,1.0],
            d if d==E=>[1.0,0.0,0.0],  _=>[-1.0,0.0,0.0],
        };
        // プレイヤーの後ろ上方からの三人称視点
        const CAM_BACK: f32 = 0.45;
        const CAM_UP:   f32 = 0.18;
        let eye = [
            center[0] - fwd[0]*CAM_BACK,
            center[1] + CAM_UP,
            center[2] - fwd[2]*CAM_BACK,
        ];
        let ctr = [center[0]+fwd[0]*0.6, center[1]-0.04, center[2]+fwd[2]*0.6];
        let view = look_at(eye,ctr,[0.0,1.0,0.0]);
        let proj = perspective(90.0f32.to_radians(),
            self.gpu.width as f32/self.gpu.height as f32, 0.04, 50.0);

        // ユニフォーム用ライト構築
        let mut lights = [Light{pos:[0.0;4],col:[0.0;4]};4];
        for i in 0..4 {
            lights[i] = Light { pos: self.light_pos[i], col: LIGHT_COLS[i] };
        }
        let uni = Uni{ vp:mat_mul(proj,view), time:self.time, warp, pad:[0.0;2], lights };
        self.gpu.render(&verts,&idxs,&uni);
    }

    pub fn warp_amount(&self) -> f32 {
        if !self.level_clear { return 0.0; }
        // ベルカーブ: 0→1→0 を1.5秒かけて変化
        let t = (self.warp_timer / 1.5).clamp(0.0, 1.0);
        (t * std::f32::consts::PI).sin()
    }

    // action: 0=前進 1=左旋回 2=右旋回 3=後退
    pub fn act(&mut self, action: i32) {
        if self.level_clear { return; }
        match action {
            1 => self.facing=match self.facing{d if d==N=>W_DIR,d if d==W_DIR=>S,d if d==S=>E,_=>N},
            2 => self.facing=match self.facing{d if d==N=>E,d if d==E=>S,d if d==S=>W_DIR,_=>N},
            0 => { let f=self.facing; self.try_move(f); }
            3 => {
                let b=match self.facing{d if d==N=>S,d if d==S=>N,d if d==E=>W_DIR,_=>E};
                self.try_move(b);
            }
            _ => {}
        }
    }

    fn try_move(&mut self, dir: u8) {
        match self.maze.can_move(self.px, self.pz, dir) {
            Some((nx,nz)) => {
                self.px=nx; self.pz=nz; self.steps+=1;
                if self.px==MAZE_W-1 && self.pz==MAZE_H-1 {
                    self.total_steps+=self.steps;
                    self.level_clear=true;
                    self.warp_timer=0.0;
                    self.spawn_goal_particles();
                }
            }
            None => {
                self.spawn_hit_particles(dir);
            }
        }
    }

    fn spawn_hit_particles(&mut self, dir: u8) {
        let mut rng = (self.time * 100000.0) as u64 | 1;
        let (wx, wz) = match dir {
            d if d==N => (self.px as f32+0.5, self.pz as f32),
            d if d==S => (self.px as f32+0.5, self.pz as f32+1.0),
            d if d==E => (self.px as f32+1.0, self.pz as f32+0.5),
            _         => (self.px as f32,      self.pz as f32+0.5),
        };
        for _ in 0..12 {
            let angle = lcg_f(&mut rng) * std::f32::consts::TAU;
            let speed = 0.025 + lcg_f(&mut rng) * 0.04;
            self.particles.push(Particle {
                pos:  [wx, 0.2 + lcg_f(&mut rng) * 0.6, wz],
                vel:  [angle.cos()*speed, 0.03+lcg_f(&mut rng)*0.04, angle.sin()*speed],
                life: 1.0,
            });
        }
    }

    fn spawn_goal_particles(&mut self) {
        let mut rng = (self.time * 99999.0) as u64 | 3;
        let (gx,gz) = ((MAZE_W-1) as f32+0.5, (MAZE_H-1) as f32+0.5);
        for _ in 0..30 {
            let angle = lcg_f(&mut rng) * std::f32::consts::TAU;
            let speed = 0.04 + lcg_f(&mut rng) * 0.06;
            self.particles.push(Particle {
                pos:  [gx, 0.5 + lcg_f(&mut rng) * 0.8, gz],
                vel:  [angle.cos()*speed, 0.05+lcg_f(&mut rng)*0.06, angle.sin()*speed],
                life: 1.0,
            });
        }
    }

    pub fn next_level(&mut self) {
        let mut seed = (js_sys::Math::random() * u64::MAX as f64) as u64 | 1;
        self.maze = Maze::new(seed);
        self.light_pos = find_lights(&self.maze, &mut seed);
        self.px=0; self.pz=0; self.facing=S; self.steps=0;
        self.level+=1; self.level_clear=false; self.warp_timer=0.0;
        self.particles.clear();
    }

    pub fn reset(&mut self) {
        let mut seed = (js_sys::Math::random() * u64::MAX as f64) as u64 | 1;
        self.maze = Maze::new(seed);
        self.light_pos = find_lights(&self.maze, &mut seed);
        self.px=0; self.pz=0; self.facing=S;
        self.steps=0; self.total_steps=0; self.level=1;
        self.level_clear=false; self.warp_timer=0.0;
        self.particles.clear();
    }
}
