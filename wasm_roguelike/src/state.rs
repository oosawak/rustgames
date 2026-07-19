/// Roguelike Dungeon - ゲーム状態管理
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, PartialEq)]
pub enum TileType {
    Floor,
    Wall,
    Room,
    StairDown,
    StairUp,
}

#[derive(Clone)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub struct Projectile {
    pub from_x: f64,
    pub from_y: f64,
    pub to_x: f64,
    pub to_y: f64,
    pub progress: f64,  // 0.0 to 1.0
    pub proj_type: i32, // 0=attack, 1=magic
    pub direction: i32, // 0=up, 1=left, 2=right, 3=down
}

pub struct RoguelikeGame {
    pub scene: RogueScene,
    pub depth: u32,
    pub level: u32,
    pub hp: u32,
    pub max_hp: u32,
    pub mp: u32,
    pub max_mp: u32,
    pub player_x: i32,
    pub player_y: i32,
    pub player_direction: i32,  // 0=up, 1=left, 2=right, 3=down
    pub enemies: Vec<Enemy>,
    pub messages: Vec<String>,
    pub map: Vec<Vec<TileType>>,
    pub map_width: i32,
    pub map_height: i32,
    pub rooms: Vec<Room>,
    pub visited: Vec<Vec<bool>>,
    pub player_shake: u32,
    pub enemy_shake: Vec<u32>,
    pub projectiles: Vec<Projectile>,
    pub exp: u32,
    pub next_level_exp: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RogueScene {
    Title = 0,
    Playing = 1,
    GameOver = 2,
}

impl RogueScene {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

pub struct Enemy {
    pub x: i32,
    pub y: i32,
    pub hp: u32,
    pub color: [f32; 3],
    pub name: String,
}

impl RoguelikeGame {
    fn calc_map_size(depth: u32) -> (i32, i32) {
        let width = 120 + ((depth.saturating_sub(1)) as i32 * 4);
        let height = 80 + ((depth.saturating_sub(1)) as i32 * 2);
        (width, height)
    }

    pub fn new() -> Self {
        let (map_width, map_height) = Self::calc_map_size(1);
        let (map, rooms) = Self::generate_dungeon(map_width, map_height, 1);
        let visited = vec![vec![false; map_width as usize]; map_height as usize];

        Self {
            scene: RogueScene::Title,
            depth: 1,
            level: 1,
            hp: 50,
            max_hp: 50,
            mp: 30,
            max_mp: 30,
            player_x: 0,
            player_y: 0,
            player_direction: 2,  // default facing right
            enemies: vec![],
            messages: vec!["ダンジョンに入った...".to_string()],
            map,
            map_width,
            map_height,
            rooms,
            visited,
            player_shake: 0,
            enemy_shake: vec![],
            projectiles: vec![],
            exp: 0,
            next_level_exp: 100,
        }
    }

    fn generate_dungeon(width: i32, height: i32, seed: u32) -> (Vec<Vec<TileType>>, Vec<Room>) {
        let mut map = vec![vec![TileType::Wall; width as usize]; height as usize];
        let mut rooms: Vec<Room> = Vec::new();
        let mut rng = LcgRng::new(seed);

        // 部屋を生成
        let room_count = 8 + (rng.next() % 5) as i32;
        for _ in 0..room_count {
            let room_width = 7 + (rng.next() % 5) as i32;
            let room_height = 5 + (rng.next() % 4) as i32;
            let room_x = (rng.next() as i32 % (width - room_width - 5)) + 2;
            let room_y = (rng.next() as i32 % (height - room_height - 5)) + 2;

            // 部屋が既存の部屋と重ならないか確認
            let mut overlaps = false;
            for r in &rooms {
                if room_x < r.x + r.width + 2 && room_x + room_width + 2 > r.x
                    && room_y < r.y + r.height + 2 && room_y + room_height + 2 > r.y {
                    overlaps = true;
                    break;
                }
            }

            if !overlaps {
                // 部屋を配置
                for ry in room_y..(room_y + room_height).min(height) {
                    for rx in room_x..(room_x + room_width).min(width) {
                        map[ry as usize][rx as usize] = TileType::Floor;
                    }
                }
                rooms.push(Room {
                    x: room_x,
                    y: room_y,
                    width: room_width,
                    height: room_height,
                });
            }
        }

        // 部屋を通路で接続
        for i in 1..rooms.len() {
            let (x1, y1) = (
                rooms[i - 1].x + rooms[i - 1].width / 2,
                rooms[i - 1].y + rooms[i - 1].height / 2,
            );
            let (x2, y2) = (
                rooms[i].x + rooms[i].width / 2,
                rooms[i].y + rooms[i].height / 2,
            );

            // 水平通路
            let (start, end) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
            for x in start..=end {
                if x >= 0 && x < width && y1 >= 0 && y1 < height {
                    map[y1 as usize][x as usize] = TileType::Floor;
                }
            }

            // 垂直通路
            let (start, end) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
            for y in start..=end {
                if x2 >= 0 && x2 < width && y >= 0 && y < height {
                    map[y as usize][x2 as usize] = TileType::Floor;
                }
            }
        }

        // 最初の部屋に上り階段を配置（部屋の内側に）
        if !rooms.is_empty() {
            let room = &rooms[0];
            let stair_x = (room.x + 1).max(0).min(width - 1);
            let stair_y = (room.y + 1).max(0).min(height - 1);
            if map[stair_y as usize][stair_x as usize] != TileType::Wall {
                map[stair_y as usize][stair_x as usize] = TileType::StairUp;
            }
        }

        // 最後の部屋に下り階段を配置（部屋の内側に）
        if rooms.len() > 1 {
            let room = &rooms[rooms.len() - 1];
            let stair_x = (room.x + room.width - 2).max(0).min(width - 1);
            let stair_y = (room.y + room.height - 2).max(0).min(height - 1);
            if map[stair_y as usize][stair_x as usize] != TileType::Wall {
                map[stair_y as usize][stair_x as usize] = TileType::StairDown;
            }
        }

        (map, rooms)
    }


    pub fn start_game(&mut self) {
        self.scene = RogueScene::Playing;
        self.messages.clear();
        self.messages.push("ゲーム開始！".to_string());

        // 最初の部屋にプレイヤーを配置
        if !self.rooms.is_empty() {
            let room = &self.rooms[0];
            self.player_x = room.x + room.width / 2;
            self.player_y = room.y + room.height / 2;
        }

        self.hp = self.max_hp;
        self.mp = self.max_mp;

        // 訪問済みをリセット
        for row in self.visited.iter_mut() {
            for cell in row.iter_mut() {
                *cell = false;
            }
        }

        // スタート位置を訪問済みに
        if self.player_y >= 0 && self.player_y < self.map_height
            && self.player_x >= 0 && self.player_x < self.map_width {
            self.visited[self.player_y as usize][self.player_x as usize] = true;
        }

        // 敵を配置
        self.enemies.clear();
        self.enemy_shake.clear();
        let mut rng = LcgRng::new(self.depth);

        for i in 0..3.min(self.rooms.len()) {
            let room = &self.rooms[i + 1];
            let ex = room.x + room.width / 2;
            let ey = room.y + room.height / 2;

            let colors = [
                [1.0, 0.2, 0.2],
                [1.0, 0.5, 0.0],
                [0.8, 0.2, 0.8],
            ];
            let names = ["Goblin", "Troll", "Ghost"];
            self.enemies.push(Enemy {
                x: ex,
                y: ey,
                hp: 20,
                color: colors[self.enemies.len() % 3],
                name: names[self.enemies.len() % 3].to_string(),
            });
            self.enemy_shake.push(0);
        }
    }

    fn is_walkable(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.map_width || y < 0 || y >= self.map_height {
            return false;
        }
        matches!(self.map[y as usize][x as usize],
            TileType::Floor | TileType::Room | TileType::StairDown | TileType::StairUp)
    }

    pub fn move_player(&mut self, action: i32) {
        if self.scene != RogueScene::Playing {
            return;
        }

        // action: 0=up, 1=left, 2=right, 3=down, 4=magic
        if action == 4 {
            // Magic attack - consumes MP
            let magic_cost: u32 = 5;
            if self.mp < magic_cost {
                self.add_message("MPが足りません".to_string());
                return;
            }

            self.mp -= magic_cost;

            // Magic attack in player direction
            let (dx, dy) = match self.player_direction {
                0 => (0, -18),  // up
                1 => (-18, 0),  // left
                2 => (18, 0),   // right
                3 => (0, 18),   // down
                _ => (0, 0),
            };
            let target_x = (self.player_x + dx) as f64;
            let target_y = (self.player_y + dy) as f64;
            self.projectiles.push(Projectile {
                from_x: self.player_x as f64,
                from_y: self.player_y as f64,
                to_x: target_x,
                to_y: target_y,
                progress: 0.0,
                proj_type: 1,  // magic
                direction: self.player_direction,
            });
            self.add_message("魔法を放った！".to_string());
            return;
        }

        self.player_direction = action;

        let (dx, dy) = match action {
            0 => (0, -1),
            1 => (-1, 0),
            2 => (1, 0),
            3 => (0, 1),
            _ => return,
        };

        let new_x = self.player_x + dx;
        let new_y = self.player_y + dy;

        // 敵への攻撃判定
        let mut attacked_enemy = false;
        for i in 0..self.enemies.len() {
            if self.enemies[i].x == new_x && self.enemies[i].y == new_y {
                // 敵に攻撃
                self.enemies[i].hp = (self.enemies[i].hp as i32 - 15).max(0) as u32;
                self.add_message("敵を攻撃した！".to_string());

                // 敵を震わせる
                if i < self.enemy_shake.len() {
                    self.enemy_shake[i] = 5;
                }

                if self.enemies[i].hp == 0 {
                    self.add_message("敵を倒した！".to_string());
                    self.gain_exp(10);
                    self.enemies.remove(i);
                    self.enemy_shake.remove(i);
                }

                attacked_enemy = true;
                break;
            }
        }

        if attacked_enemy {
            // 敵の反撃
            let enemy_positions: Vec<(i32, i32)> = self.enemies.iter().map(|e| (e.x, e.y)).collect();
            for (ex, ey) in enemy_positions {
                if (ex - self.player_x).abs() + (ey - self.player_y).abs() <= 1 {
                    self.hp = (self.hp as i32 - 5).max(0) as u32;
                    self.add_message("敵の反撃を受けた！".to_string());
                    self.player_shake = 5;
                }
            }

            if self.hp == 0 {
                self.scene = RogueScene::GameOver;
                self.add_message("ゲームオーバー".to_string());
            }

            self.mark_visible();
            return;
        }

        // マップの壁判定と階段チェック
        let tile = self.map[new_y as usize][new_x as usize];

        if tile == TileType::StairDown && self.depth < 30 {
            // 下り階段
            self.next_floor();
            return;
        }

        if tile == TileType::StairUp && self.depth > 1 {
            // 上り階段
            self.prev_floor();
            return;
        }

        if self.is_walkable(new_x, new_y) {
            self.player_x = new_x;
            self.player_y = new_y;
            self.add_message("移動した".to_string());

            // 敵を移動（簡易AI）
            let enemy_moves: Vec<(usize, i32, i32)> = self.enemies.iter().enumerate()
                .map(|(i, enemy)| {
                    let random_move = ((enemy.x * 73 + enemy.y * 97) as usize) % 4;
                    let (edx, edy) = match random_move {
                        0 => (0, -1),
                        1 => (-1, 0),
                        2 => (1, 0),
                        3 => (0, 1),
                        _ => (0, 0),
                    };
                    (i, enemy.x + edx, enemy.y + edy)
                })
                .collect();

            for (i, new_ex, new_ey) in enemy_moves {
                if self.is_walkable(new_ex, new_ey) && new_ex != self.player_x || new_ey != self.player_y {
                    // 他の敵との重複チェック
                    let mut occupied = false;
                    for (j, other_enemy) in self.enemies.iter().enumerate() {
                        if i != j && other_enemy.x == new_ex && other_enemy.y == new_ey {
                            occupied = true;
                            break;
                        }
                    }

                    if !occupied {
                        self.enemies[i].x = new_ex;
                        self.enemies[i].y = new_ey;
                    }
                }
            }

            // 敵の反撃
            let mut hit = false;
            for enemy in self.enemies.iter() {
                if self.player_x == enemy.x && self.player_y == enemy.y {
                    hit = true;
                    break;
                }
            }

            if hit {
                self.hp = (self.hp as i32 - 10).max(0) as u32;
                self.add_message("敵に攻撃された！".to_string());
                self.player_shake = 5;

                if self.hp == 0 {
                    self.scene = RogueScene::GameOver;
                    self.add_message("ゲームオーバー".to_string());
                }
            }

            // 訪問済みをマーク
            self.mark_visible();
        } else {
            self.add_message("壁にぶつかった".to_string());
        }
    }

    fn mark_visible(&mut self) {
        // プレイヤーの周辺 (視野範囲) を訪問済みに
        let view_range = 1;
        for dy in -view_range..=view_range {
            for dx in -view_range..=view_range {
                let x = self.player_x + dx;
                let y = self.player_y + dy;

                if x >= 0 && x < self.map_width && y >= 0 && y < self.map_height {
                    self.visited[y as usize][x as usize] = true;
                }
            }
        }
    }

    fn gain_exp(&mut self, amount: u32) {
        self.exp += amount;
        while self.exp >= self.next_level_exp {
            self.exp -= self.next_level_exp;
            self.level_up();
        }
    }

    fn level_up(&mut self) {
        self.level += 1;
        self.max_hp += 10;
        self.max_mp += 5;
        self.hp = self.max_hp;
        self.mp = self.max_mp;
        self.next_level_exp = self.level * 50;
        self.add_message(format!("レベルアップ！LV{}", self.level));
    }

    pub fn tick(&mut self, _ts: f64) {
        // 震える時間を減らす
        if self.player_shake > 0 {
            self.player_shake -= 1;
        }

        for shake in self.enemy_shake.iter_mut() {
            if *shake > 0 {
                *shake -= 1;
            }
        }

        // Update projectiles
        for projectile in self.projectiles.iter_mut() {
            projectile.progress += 0.008;
        }

        // Check magic collision with enemies and damage them
        let mut hit_projectiles = std::collections::HashSet::new();
        for (proj_idx, projectile) in self.projectiles.iter().enumerate() {
            if projectile.proj_type == 1 && projectile.progress > 0.1 {  // magic
                let current_x = projectile.from_x + (projectile.to_x - projectile.from_x) * projectile.progress;
                let current_y = projectile.from_y + (projectile.to_y - projectile.from_y) * projectile.progress;
                let map_x = current_x as i32;
                let map_y = current_y as i32;

                // Check enemy collision and damage
                for i in 0..self.enemies.len() {
                    if self.enemies[i].x == map_x && self.enemies[i].y == map_y {
                        self.enemies[i].hp = (self.enemies[i].hp as i32 - 5).max(0) as u32;
                        hit_projectiles.insert(proj_idx);

                        if self.enemies[i].hp == 0 {
                            self.enemy_shake[i] = 5;
                        } else {
                            self.enemy_shake[i] = 3;
                        }
                        break;
                    }
                }
            }
        }

        // Add messages after the loop and gain exp
        for i in 0..self.enemies.len() {
            if self.enemy_shake[i] == 5 {
                self.add_message("敵を倒した！".to_string());
                self.gain_exp(10);
                break;
            }
        }

        // Remove dead enemies
        self.enemies.retain(|e| e.hp > 0);
        self.enemy_shake.truncate(self.enemies.len());

        // Check magic collision with walls
        self.projectiles.retain(|p| {
            if p.proj_type == 1 {  // magic
                // Only check collision after progress > 0.1 to avoid colliding with starting position
                if p.progress > 0.1 {
                    let current_x = p.from_x + (p.to_x - p.from_x) * p.progress;
                    let current_y = p.from_y + (p.to_y - p.from_y) * p.progress;
                    let map_x = current_x as i32;
                    let map_y = current_y as i32;

                    // Check wall collision
                    if map_x < 0 || map_x >= self.map_width || map_y < 0 || map_y >= self.map_height {
                        return false;  // Out of bounds
                    }

                    let tile = self.map[map_y as usize][map_x as usize];
                    if tile == TileType::Wall {
                        return false;  // Hit wall
                    }
                }

                p.progress < 1.0
            } else {
                p.progress < 1.0
            }
        });
    }

    pub fn add_message(&mut self, msg: String) {
        self.messages.push(msg);
        // 最新5件のみ保持
        if self.messages.len() > 5 {
            self.messages.remove(0);
        }
    }

    pub fn game_over(&mut self) {
        self.scene = RogueScene::GameOver;
    }

    pub fn next_floor(&mut self) {
        if self.depth >= 30 {
            self.add_message("最下階です".to_string());
            return;
        }

        self.depth += 1;
        self.level += 1;
        self.hp = self.max_hp;
        self.mp = self.max_mp;
        self.messages.clear();
        self.messages.push(format!("F{} に到着した", self.depth));

        // マップサイズを更新
        let (map_width, map_height) = Self::calc_map_size(self.depth);
        self.map_width = map_width;
        self.map_height = map_height;

        // 新しいダンジョンを生成
        let (map, rooms) = Self::generate_dungeon(map_width, map_height, self.depth);
        self.map = map;
        self.rooms = rooms;

        // 訪問済みを新しいサイズでリセット
        self.visited = vec![vec![false; map_width as usize]; map_height as usize];

        // プレイヤーを上り階段の場所に配置
        if !self.rooms.is_empty() {
            let room = &self.rooms[0];
            self.player_x = (room.x + 1).max(0).min(self.map_width - 1);
            self.player_y = (room.y + 1).max(0).min(self.map_height - 1);
        }

        // 敵を配置
        self.enemies.clear();
        self.enemy_shake.clear();

        for i in 0..3.min(self.rooms.len()) {
            let room = &self.rooms[i + 1];
            let ex = room.x + room.width / 2;
            let ey = room.y + room.height / 2;

            let colors = [
                [1.0, 0.2, 0.2],
                [1.0, 0.5, 0.0],
                [0.8, 0.2, 0.8],
            ];
            let names = ["Goblin", "Troll", "Ghost"];
            self.enemies.push(Enemy {
                x: ex,
                y: ey,
                hp: 20 + (self.depth as u32 * 5),
                color: colors[self.enemies.len() % 3],
                name: names[self.enemies.len() % 3].to_string(),
            });
            self.enemy_shake.push(0);
        }

        // スタート位置を訪問済みに
        if self.player_y >= 0 && self.player_y < self.map_height
            && self.player_x >= 0 && self.player_x < self.map_width {
            self.visited[self.player_y as usize][self.player_x as usize] = true;
        }
    }

    pub fn prev_floor(&mut self) {
        if self.depth <= 1 {
            self.add_message("地上です".to_string());
            return;
        }

        self.depth -= 1;
        self.level = self.depth;
        self.hp = self.max_hp;
        self.mp = self.max_mp;
        self.messages.clear();
        self.messages.push(format!("F{} に戻った", self.depth));

        // マップサイズを更新
        let (map_width, map_height) = Self::calc_map_size(self.depth);
        self.map_width = map_width;
        self.map_height = map_height;

        // 新しいダンジョンを生成
        let (map, rooms) = Self::generate_dungeon(map_width, map_height, self.depth);
        self.map = map;
        self.rooms = rooms;

        // 訪問済みを新しいサイズでリセット
        self.visited = vec![vec![false; map_width as usize]; map_height as usize];

        // プレイヤーを下り階段の場所に配置
        if self.rooms.len() > 1 {
            let room = &self.rooms[self.rooms.len() - 1];
            self.player_x = (room.x + room.width - 2).max(0).min(self.map_width - 1);
            self.player_y = (room.y + room.height - 2).max(0).min(self.map_height - 1);
        }

        // 敵を配置
        self.enemies.clear();
        self.enemy_shake.clear();

        for i in 0..3.min(self.rooms.len()) {
            let room = &self.rooms[i + 1];
            let ex = room.x + room.width / 2;
            let ey = room.y + room.height / 2;

            let colors = [
                [1.0, 0.2, 0.2],
                [1.0, 0.5, 0.0],
                [0.8, 0.2, 0.8],
            ];
            let names = ["Goblin", "Troll", "Ghost"];
            self.enemies.push(Enemy {
                x: ex,
                y: ey,
                hp: 20 + (self.depth as u32 * 5),
                color: colors[self.enemies.len() % 3],
                name: names[self.enemies.len() % 3].to_string(),
            });
            self.enemy_shake.push(0);
        }

        // スタート位置を訪問済みに
        if self.player_y >= 0 && self.player_y < self.map_height
            && self.player_x >= 0 && self.player_x < self.map_width {
            self.visited[self.player_y as usize][self.player_x as usize] = true;
        }
    }
}

pub fn render_canvas(game: &RoguelikeGame, canvas_id: &str, width: i32, height: i32) {
    use web_sys::{window, HtmlCanvasElement, CanvasRenderingContext2d};
    use js_sys::Object;

    let window = window().unwrap();
    let document = window.document().unwrap();

    let canvas = document.get_element_by_id(canvas_id)
        .and_then(|e| e.dyn_into::<HtmlCanvasElement>().ok());

    if let Some(canvas) = canvas {
        canvas.set_width(width as u32);
        canvas.set_height(height as u32);

        if let Ok(ctx) = canvas.get_context("2d") {
            if let Some(ctx) = ctx {
                let ctx: CanvasRenderingContext2d = ctx.dyn_into().unwrap();

                // Clear
                ctx.set_fill_style(&"#000".into());
                ctx.fill_rect(0.0, 0.0, width as f64, height as f64);

                // Camera settings - zoom and follow player
                let view_width = 15i32;  // タイル数
                let view_height = 10i32;

                // Font and tile size settings
                let font_size = 20.0;  // フォントサイズ（px）
                let cell_size = 32.0;  // タイルサイズ（px）、フォント + 余白

                let cell_w = cell_size;
                let cell_h = cell_size;

                // カメラはプレイヤーを中心に
                let camera_x = (game.player_x - view_width / 2).max(0).min(game.map_width - view_width);
                let camera_y = (game.player_y - view_height / 2).max(0).min(game.map_height - view_height);

                // Draw visible tiles
                for y in camera_y..(camera_y + view_height).min(game.map_height) {
                    for x in camera_x..(camera_x + view_width).min(game.map_width) {
                        let screen_x = (x - camera_x) as f64 * cell_w;
                        let screen_y = (y - camera_y) as f64 * cell_h;

                        let tile_type = game.map[y as usize][x as usize];
                        let color = match tile_type {
                            crate::state::TileType::Wall => "#444",
                            crate::state::TileType::Floor => "#223",
                            crate::state::TileType::Room => "#335",
                            crate::state::TileType::StairDown => "#dd0",
                            crate::state::TileType::StairUp => "#0dd",
                        };

                        ctx.set_fill_style(&color.into());
                        ctx.fill_rect(screen_x, screen_y, cell_w, cell_h);

                        ctx.set_stroke_style(&"0ff".into());
                        ctx.set_line_width(0.3);
                        ctx.stroke_rect(screen_x, screen_y, cell_w, cell_h);

                        // Draw stair icons
                        if matches!(tile_type, crate::state::TileType::StairDown | crate::state::TileType::StairUp) {
                            if let Ok(img_elem) = window.document().unwrap()
                                .get_element_by_id("stairs-icon")
                                .unwrap()
                                .dyn_into::<web_sys::HtmlImageElement>()
                            {
                                let stair_x = screen_x + cell_w * 0.5;
                                let stair_y = screen_y + cell_h * 0.5;
                                let icon_size = cell_w * 0.6;

                                ctx.save();
                                ctx.translate(stair_x, stair_y).ok();
                                ctx.draw_image_with_html_image_element_and_dw_and_dh(
                                    &img_elem,
                                    -icon_size * 0.5,
                                    -icon_size * 0.5,
                                    icon_size,
                                    icon_size
                                ).ok();
                                ctx.restore();
                            }
                        }
                    }
                }

                // Draw enemies
                let enemy_icon_ids = ["goblin-icon", "troll-icon", "ghost-icon"];
                for (i, enemy) in game.enemies.iter().enumerate() {
                    if enemy.x >= camera_x && enemy.x < camera_x + view_width
                        && enemy.y >= camera_y && enemy.y < camera_y + view_height
                    {
                        let screen_x = (enemy.x - camera_x) as f64 * cell_w;
                        let screen_y = (enemy.y - camera_y) as f64 * cell_h;

                        // 震えるアニメーション用のオフセット
                        let mut shake_offset_x = 0.0;
                        let mut shake_offset_y = 0.0;
                        if i < game.enemy_shake.len() && game.enemy_shake[i] > 0 {
                            let shake = ((game.enemy_shake[i] * 7) % 4) as f64 - 1.5;
                            shake_offset_x = shake;
                            shake_offset_y = shake;
                        }

                        // Draw enemy icon
                        if let Ok(img_elem) = window.document().unwrap()
                            .get_element_by_id(enemy_icon_ids[i % 3])
                            .unwrap()
                            .dyn_into::<web_sys::HtmlImageElement>()
                        {
                            let icon_x = screen_x + cell_w * 0.5 + shake_offset_x;
                            let icon_y = screen_y + cell_h * 0.5 + shake_offset_y;
                            let icon_size = cell_w * 0.6;

                            ctx.save();
                            ctx.translate(icon_x, icon_y).ok();
                            ctx.draw_image_with_html_image_element_and_dw_and_dh(
                                &img_elem,
                                -icon_size * 0.5,
                                -icon_size * 0.5,
                                icon_size,
                                icon_size
                            ).ok();
                            ctx.restore();
                        }
                    }
                }

                // Draw player (always at center)
                let player_screen_x = (game.player_x - camera_x) as f64 * cell_w;
                let player_screen_y = (game.player_y - camera_y) as f64 * cell_h;

                // 震えるアニメーション用のオフセット
                let mut shake_offset_x = 0.0;
                let mut shake_offset_y = 0.0;
                if game.player_shake > 0 {
                    let shake = ((game.player_shake * 7) % 4) as f64 - 1.5;
                    shake_offset_x = shake;
                    shake_offset_y = shake;
                }

                // Draw player icon
                if let Ok(img_elem) = window.document().unwrap()
                    .get_element_by_id("player-icon")
                    .unwrap()
                    .dyn_into::<web_sys::HtmlImageElement>()
                {
                    let icon_x = player_screen_x + cell_w * 0.5 + shake_offset_x;
                    let icon_y = player_screen_y + cell_h * 0.5 + shake_offset_y;
                    let icon_size = cell_w * 0.6;

                    ctx.save();
                    ctx.translate(icon_x, icon_y).ok();

                    // 方向に応じて反転: left は反転、right は そのまま
                    if game.player_direction == 1 {
                        ctx.scale(-1.0, 1.0).ok();
                    }

                    ctx.draw_image_with_html_image_element_and_dw_and_dh(
                        &img_elem,
                        -icon_size * 0.5,
                        -icon_size * 0.5,
                        icon_size,
                        icon_size
                    ).ok();
                    ctx.restore();
                }

                // Draw magic projectiles
                for projectile in game.projectiles.iter() {
                    let current_x = projectile.from_x + (projectile.to_x - projectile.from_x) * projectile.progress;
                    let current_y = projectile.from_y + (projectile.to_y - projectile.from_y) * projectile.progress;

                    let screen_x = (current_x - camera_x as f64) * cell_w;
                    let screen_y = (current_y - camera_y as f64) * cell_h;
                    let icon_x = screen_x + cell_w * 0.5;
                    let icon_y = screen_y + cell_h * 0.5;

                    ctx.save();

                    // Draw magic as a glowing orb (cyan) - shrinks as it travels
                    let size_factor = 1.0 - projectile.progress;

                    // Glow effect
                    ctx.set_fill_style(&"rgba(0,255,255,0.2)".into());
                    ctx.begin_path();
                    ctx.arc(icon_x, icon_y, cell_w * 0.2 * size_factor, 0.0, std::f64::consts::PI * 2.0).ok();
                    ctx.fill();

                    // Core orb
                    ctx.set_fill_style(&"#0ff".into());
                    ctx.begin_path();
                    ctx.arc(icon_x, icon_y, cell_w * 0.12 * size_factor, 0.0, std::f64::consts::PI * 2.0).ok();
                    ctx.fill();

                    // Bright center
                    ctx.set_fill_style(&"#fff".into());
                    ctx.begin_path();
                    ctx.arc(icon_x, icon_y, cell_w * 0.05 * size_factor, 0.0, std::f64::consts::PI * 2.0).ok();
                    ctx.fill();

                    ctx.restore();
                }

                // Draw HP bar at top
                ctx.set_fill_style(&"#333".into());
                ctx.fill_rect(5.0, 5.0, 150.0, 30.0);

                ctx.set_fill_style(&"#f00".into());
                let hp_width = (game.hp as f64 / game.max_hp as f64) * 140.0;
                ctx.fill_rect(10.0, 10.0, hp_width, 10.0);

                ctx.set_fill_style(&"#fff".into());
                ctx.set_font("12px monospace");
                ctx.fill_text(
                    &format!("HP: {}/{}", game.hp, game.max_hp),
                    15.0,
                    28.0,
                ).ok();

                // Draw MP bar next to HP
                ctx.set_fill_style(&"#333".into());
                ctx.fill_rect(160.0, 5.0, 150.0, 30.0);

                ctx.set_fill_style(&"#00f".into());
                let mp_width = (game.mp as f64 / game.max_mp as f64) * 140.0;
                ctx.fill_rect(165.0, 10.0, mp_width, 10.0);

                ctx.set_fill_style(&"#fff".into());
                ctx.set_font("12px monospace");
                ctx.fill_text(
                    &format!("MP: {}/{}", game.mp, game.max_mp),
                    170.0,
                    28.0,
                ).ok();

                // Draw enemy HP and name
                for enemy in &game.enemies {
                    if enemy.x >= camera_x && enemy.x < camera_x + view_width
                        && enemy.y >= camera_y && enemy.y < camera_y + view_height
                    {
                        let screen_x = (enemy.x - camera_x) as f64 * cell_w;
                        let screen_y = (enemy.y - camera_y) as f64 * cell_h;

                        // HP above
                        ctx.set_fill_style(&"#fff".into());
                        ctx.set_font("8px monospace");
                        ctx.set_text_align("center");
                        ctx.fill_text(
                            &format!("HP:{}", enemy.hp),
                            screen_x + cell_w * 0.5,
                            screen_y - 5.0,
                        ).ok();

                        // Name below
                        ctx.set_fill_style(&"#aaf".into());
                        ctx.set_font("7px monospace");
                        ctx.set_text_align("center");
                        ctx.fill_text(
                            &enemy.name,
                            screen_x + cell_w * 0.5,
                            screen_y + cell_h + 10.0,
                        ).ok();
                    }
                }
            }
        }
    }
}

/// 簡易線形合同法乱数生成器
struct LcgRng {
    state: u32,
}

impl LcgRng {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        (self.state / 65536) % 32768
    }
}
