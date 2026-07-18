/// Roguelike Dungeon - ゲーム状態管理
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, PartialEq)]
pub enum TileType {
    Floor,
    Wall,
    Room,
}

#[derive(Clone)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
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
    pub enemies: Vec<Enemy>,
    pub messages: Vec<String>,
    pub map: Vec<Vec<TileType>>,
    pub map_width: i32,
    pub map_height: i32,
    pub rooms: Vec<Room>,
    pub visited: Vec<Vec<bool>>,
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
}

impl RoguelikeGame {
    pub fn new() -> Self {
        let map_width = 80i32;
        let map_height = 50i32;
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
            enemies: vec![],
            messages: vec!["ダンジョンに入った...".to_string()],
            map,
            map_width,
            map_height,
            rooms,
            visited,
        }
    }

    fn generate_dungeon(width: i32, height: i32, seed: u32) -> (Vec<Vec<TileType>>, Vec<Room>) {
        let mut map = vec![vec![TileType::Wall; width as usize]; height as usize];
        let mut rooms: Vec<Room> = Vec::new();
        let mut rng = LcgRng::new(seed);

        // 部屋を生成
        let room_count = 6 + (rng.next() % 4) as i32;
        for _ in 0..room_count {
            let room_width = 6 + (rng.next() % 5) as i32;
            let room_height = 4 + (rng.next() % 4) as i32;
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
            self.enemies.push(Enemy {
                x: ex,
                y: ey,
                hp: 20,
                color: colors[self.enemies.len() % 3],
            });
        }
    }

    fn is_walkable(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.map_width || y < 0 || y >= self.map_height {
            return false;
        }
        self.map[y as usize][x as usize] == TileType::Floor
    }

    pub fn move_player(&mut self, action: i32) {
        if self.scene != RogueScene::Playing {
            return;
        }

        // action: 0=up, 1=left, 2=right, 3=down
        let (dx, dy) = match action {
            0 => (0, -1),
            1 => (-1, 0),
            2 => (1, 0),
            3 => (0, 1),
            _ => return,
        };

        let new_x = self.player_x + dx;
        let new_y = self.player_y + dy;

        // マップの壁判定
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
                if self.is_walkable(new_ex, new_ey) {
                    self.enemies[i].x = new_ex;
                    self.enemies[i].y = new_ey;
                }
            }

            // 敵との衝突判定
            let mut hit = false;
            for enemy in &self.enemies {
                if self.player_x == enemy.x && self.player_y == enemy.y {
                    hit = true;
                    break;
                }
            }

            if hit {
                self.hp = (self.hp as i32 - 10).max(0) as u32;
                self.add_message("敵に攻撃された！".to_string());
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

    pub fn tick(&mut self, _ts: f64) {
        // ゲームループ（フレーム更新）
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
        self.depth += 1;
        self.level += 1;
        self.hp = self.max_hp;
        self.mp = self.max_mp;
        self.messages.clear();
        self.messages.push(format!("F{} に到着した", self.depth));

        // 新しいダンジョンを生成
        let (map, rooms) = Self::generate_dungeon(self.map_width, self.map_height, self.depth);
        self.map = map;
        self.rooms = rooms;

        // 訪問済みをリセット
        for row in self.visited.iter_mut() {
            for cell in row.iter_mut() {
                *cell = false;
            }
        }

        // プレイヤーを最初の部屋に配置
        if !self.rooms.is_empty() {
            let room = &self.rooms[0];
            self.player_x = room.x + room.width / 2;
            self.player_y = room.y + room.height / 2;
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

                let cell_w = width as f64 / view_width as f64;
                let cell_h = height as f64 / view_height as f64;

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
                            crate::state::TileType::Wall => "#222",
                            crate::state::TileType::Floor => "#001",
                            crate::state::TileType::Room => "#003",
                        };

                        ctx.set_fill_style(&color.into());
                        ctx.fill_rect(screen_x, screen_y, cell_w, cell_h);

                        ctx.set_stroke_style(&"0ff".into());
                        ctx.set_line_width(0.3);
                        ctx.stroke_rect(screen_x, screen_y, cell_w, cell_h);
                    }
                }

                // Draw enemies
                for enemy in &game.enemies {
                    if enemy.x >= camera_x && enemy.x < camera_x + view_width
                        && enemy.y >= camera_y && enemy.y < camera_y + view_height
                    {
                        let screen_x = (enemy.x - camera_x) as f64 * cell_w + 2.0;
                        let screen_y = (enemy.y - camera_y) as f64 * cell_h + 2.0;
                        let color = format!("rgb({},{},{})",
                            (enemy.color[0] * 255.0) as i32,
                            (enemy.color[1] * 255.0) as i32,
                            (enemy.color[2] * 255.0) as i32
                        );
                        ctx.set_fill_style(&color.into());
                        ctx.fill_rect(screen_x, screen_y, cell_w - 4.0, cell_h - 4.0);
                    }
                }

                // Draw player (always at center)
                let player_screen_x = (game.player_x - camera_x) as f64 * cell_w + 2.0;
                let player_screen_y = (game.player_y - camera_y) as f64 * cell_h + 2.0;
                ctx.set_fill_style(&"#0f0".into());
                ctx.fill_rect(player_screen_x, player_screen_y, cell_w - 4.0, cell_h - 4.0);
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
