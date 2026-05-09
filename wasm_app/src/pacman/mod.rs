pub mod map;
pub mod ghost;
pub mod player;

use web_sys::CanvasRenderingContext2d;
use map::Map;
use player::Player;
use ghost::{Ghost, GhostType, GhostMode};

#[derive(PartialEq)]
pub enum GamePhase {
    Playing,
    Dying,
    GameOver,
    LevelClear,
}

// audio_event: 0=なし, 1=ドット, 2=パワーペレット, 3=ゴースト食べ, 4=死亡, 5=レベルクリア, 6=ゲームオーバー
pub struct PacmanGame {
    pub map: Map,
    pub player: Player,
    pub ghosts: Vec<Ghost>,
    pub score: u32,
    pub lives: u8,
    pub state: GamePhase,
    pub dot_count: u32,
    pub power_timer: f32,
    pub time: f64,
    pub ctx: CanvasRenderingContext2d,
    pub cell: f64,
    pub audio_event: u8,
    dying_timer: f32,
    scatter_timer: f32,
    scatter_phase: u8,
}

impl PacmanGame {
    pub fn new(ctx: CanvasRenderingContext2d, cell: f64) -> Self {
        let map = Map::new();
        let dot_count = map.remaining_dots();
        let player = Player::new(cell);
        let ghosts = vec![
            Ghost::new(GhostType::Blinky, cell),
            Ghost::new(GhostType::Pinky,  cell),
            Ghost::new(GhostType::Inky,   cell),
            Ghost::new(GhostType::Clyde,  cell),
        ];
        PacmanGame {
            map,
            player,
            ghosts,
            score: 0,
            lives: 3,
            state: GamePhase::Playing,
            dot_count,
            power_timer: 0.0,
            time: 0.0,
            ctx,
            cell,
            audio_event: 0,
            dying_timer: 0.0,
            scatter_timer: 7.0,
            scatter_phase: 0,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        let dt = dt.min(0.05);
        self.time += dt as f64;

        match self.state {
            GamePhase::Dying => {
                self.dying_timer -= dt;
                if self.dying_timer <= 0.0 {
                    if self.lives == 0 {
                        self.audio_event = 6; // ゲームオーバー
                        self.state = GamePhase::GameOver;
                    } else {
                        self.reset_after_death();
                    }
                }
                return;
            }
            GamePhase::GameOver | GamePhase::LevelClear => return,
            GamePhase::Playing => {}
        }

        // Scatter/chase phase switching
        self.scatter_timer -= dt;
        if self.scatter_timer <= 0.0 {
            self.scatter_phase = (self.scatter_phase + 1) % 8;
            self.scatter_timer = if self.scatter_phase % 2 == 0 { 7.0 } else { 20.0 };
            let new_mode = if self.scatter_phase % 2 == 0 { GhostMode::Scatter } else { GhostMode::Chase };
            for g in &mut self.ghosts {
                if g.mode != GhostMode::Frightened && g.mode != GhostMode::Eaten {
                    g.mode = new_mode;
                }
            }
        }

        // Power pellet timer
        if self.power_timer > 0.0 {
            self.power_timer -= dt;
            if self.power_timer <= 0.0 {
                self.power_timer = 0.0;
                for g in &mut self.ghosts {
                    if g.mode == GhostMode::Frightened {
                        g.mode = GhostMode::Chase;
                    }
                }
            }
        }

        self.audio_event = 0; // 毎フレームリセット

        // Update player
        self.player.update(dt, &self.map);

        // Eat dots
        let (pr, pc) = self.player.grid_pos();
        let pts = self.map.eat_dot(pr, pc);
        if pts > 0 {
            self.score += pts;
            if pts == 50 {
                self.audio_event = 2; // パワーペレット
                self.power_timer = 8.0;
                for g in &mut self.ghosts {
                    g.frighten();
                }
            } else {
                self.audio_event = 1; // 通常ドット
            }
        }

        self.dot_count = self.map.remaining_dots();
        if self.dot_count == 0 {
            self.audio_event = 5; // レベルクリア
            self.state = GamePhase::LevelClear;
            return;
        }

        // Update ghosts
        for i in 0..self.ghosts.len() {
            let bp = self.ghosts[0].pos();
            self.ghosts[i].update(dt, &self.player, bp, &self.map);
        }

        // Ghost collision
        let (px, py) = (self.player.x, self.player.y);
        let cell = self.cell;
        for g in &mut self.ghosts {
            let dx = (g.x - px).abs();
            let dy = (g.y - py).abs();
            if dx < cell * 0.6 && dy < cell * 0.6 {
                match g.mode {
                    GhostMode::Frightened => {
                        g.mode = GhostMode::Eaten;
                        self.score += 200;
                        self.audio_event = 3; // ゴースト食べ
                    }
                    GhostMode::Eaten => {}
                    _ => {
                        self.audio_event = 4; // 死亡
                        self.state = GamePhase::Dying;
                        self.dying_timer = 2.0;
                        self.lives = self.lives.saturating_sub(1);
                        self.player.alive = false;
                        return;
                    }
                }
            }
        }
    }

    fn reset_after_death(&mut self) {
        self.player.reset();
        for g in &mut self.ghosts {
            g.reset();
        }
        self.state = GamePhase::Playing;
        self.power_timer = 0.0;
    }

    pub fn draw(&self) {
        let ctx = &self.ctx;
        let cell = self.cell;
        let canvas_width = cell * 28.0;
        let canvas_height = cell * 31.0 + 60.0;

        ctx.set_fill_style_str("black");
        ctx.fill_rect(0.0, 0.0, canvas_width, canvas_height + 60.0);

        // HUD
        let hud_y = 30.0;
        ctx.set_fill_style_str("white");
        ctx.set_font("20px monospace");
        let _ = ctx.fill_text(&format!("SCORE: {}", self.score), 8.0, hud_y);
        for i in 0..self.lives {
            let lx = canvas_width - 30.0 - i as f64 * 25.0;
            ctx.begin_path();
            ctx.set_fill_style_str("yellow");
            let _ = ctx.arc(lx, hud_y - 8.0, 9.0, 0.0, std::f64::consts::TAU);
            ctx.fill();
        }

        let offset_y = 40.0;

        self.map.draw(ctx, cell, offset_y);
        self.player.draw(ctx, offset_y);
        for g in &self.ghosts {
            g.draw(ctx, offset_y, self.time);
        }

        if self.state == GamePhase::GameOver {
            ctx.set_fill_style_str("rgba(0,0,0,0.6)");
            ctx.fill_rect(0.0, 0.0, canvas_width, canvas_height);
            ctx.set_fill_style_str("red");
            ctx.set_font("bold 36px monospace");
            let _ = ctx.fill_text("GAME OVER", canvas_width / 2.0 - 100.0, canvas_height / 2.0);
            ctx.set_fill_style_str("white");
            ctx.set_font("20px monospace");
            let _ = ctx.fill_text(&format!("Score: {}", self.score), canvas_width / 2.0 - 60.0, canvas_height / 2.0 + 40.0);
        }

        if self.state == GamePhase::LevelClear {
            ctx.set_fill_style_str("rgba(0,0,0,0.6)");
            ctx.fill_rect(0.0, 0.0, canvas_width, canvas_height);
            ctx.set_fill_style_str("yellow");
            ctx.set_font("bold 30px monospace");
            let _ = ctx.fill_text("LEVEL CLEAR!", canvas_width / 2.0 - 90.0, canvas_height / 2.0);
        }
    }

    pub fn set_input(&mut self, dir: i32) {
        // dir: 1=up, 2=right, 3=down, 4=left (maps to internal 0=up,1=right,2=down,3=left)
        if dir >= 1 && dir <= 4 {
            self.player.next_dir = dir - 1;
        }
    }
}
