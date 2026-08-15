/// Roguelike Dungeon - ゲーム状態管理
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, PartialEq)]
pub enum TileType {
    Floor,
    Wall,
    Room,
    Pit,
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

#[derive(Clone)]
struct FloorState {
    map: Vec<Vec<TileType>>,
    rooms: Vec<Room>,
    visited: Vec<Vec<bool>>,
}

pub struct Projectile {
    pub from_x: f64,
    pub from_y: f64,
    pub to_x: f64,
    pub to_y: f64,
    pub progress: f64,  // 0.0 to 1.0
    pub proj_type: i32, // 0=attack, 1=magic, 2=arrow
    pub damage: u32,
    pub direction: i32, // 0=up, 1=left, 2=right, 3=down
}

#[derive(Clone)]
pub struct AttackEffect {
    pub x: i32,
    pub y: i32,
    pub ttl: u32,
    pub max_ttl: u32,
    pub color: &'static str,
}

#[derive(Clone)]
pub struct DamageNumber {
    pub x: i32,
    pub y: i32,
    pub amount: u32,
    pub ttl: u32,
    pub max_ttl: u32,
    pub color: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WeaponType {
    WoodenSword = 0,   // +3
    IronSword = 1,     // +5
    Spear = 2,         // +7
    Bow = 3,           // +9
    Staff = 4,         // +8
    CursedBlade = 5,   // +12
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WeaponStyle {
    Sword,
    Spear,
    Bow,
    Staff,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ArmorType {
    LeatherArmor = 0,  // +2
    ChainMail = 1,     // +4
    SteelPlate = 2,    // +6
    DragonScale = 3,   // +8
    CursedMail = 4,    // +10
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AccessoryType {
    GoldRing = 0,          // ゴールド獲得+20%
    VampireRing = 1,       // ダメージの10%HP回復
    LuckyRing = 2,         // クリティカル率+10%
    HealingNecklace = 3,   // MaxHP+10, HP自動回復
    ManaEarrings = 4,      // MaxMP+15, MP自動回復
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ItemType {
    HealthPotion = 0,      // HP +50
    ManaPotion = 1,        // MP +30
    PoisonPotion = 2,      // 敵に毒ダメージ
    EnergyDrink = 3,       // HP+Max+5
    Gem = 4,               // 売却アイテム
    SkeletonKey = 5,       // ドア開錠
    Scroll = 6,            // 魔法書
    GoldenCoin = 7,        // お金
}

#[derive(Clone, Copy, Debug)]
pub struct Equipment {
    pub weapon: Option<WeaponType>,
    pub armor: Option<ArmorType>,
    pub accessory: Option<AccessoryType>,
}

#[derive(Clone, Debug)]
pub struct EquipmentInventory {
    pub weapons: Vec<WeaponType>,
    pub armors: Vec<ArmorType>,
    pub accessories: Vec<AccessoryType>,
}

impl Equipment {
    pub fn new() -> Self {
        Self {
            weapon: Some(WeaponType::WoodenSword),  // 初期装備
            armor: None,
            accessory: None,
        }
    }

    pub fn get_atk_bonus(&self) -> u32 {
        let weapon_bonus = match self.weapon {
            Some(WeaponType::WoodenSword) => 3,
            Some(WeaponType::IronSword) => 5,
            Some(WeaponType::Spear) => 7,
            Some(WeaponType::Bow) => 9,
            Some(WeaponType::Staff) => 8,
            Some(WeaponType::CursedBlade) => 12,
            None => 0,
        };
        weapon_bonus
    }

    pub fn get_def_bonus(&self) -> u32 {
        let armor_bonus = match self.armor {
            Some(ArmorType::LeatherArmor) => 2,
            Some(ArmorType::ChainMail) => 4,
            Some(ArmorType::SteelPlate) => 6,
            Some(ArmorType::DragonScale) => 8,
            Some(ArmorType::CursedMail) => 10,
            None => 0,
        };
        armor_bonus
    }

    pub fn get_max_hp_bonus(&self) -> u32 {
        match self.accessory {
            Some(AccessoryType::HealingNecklace) => 10,
            _ => 0,
        }
    }

    pub fn get_max_mp_bonus(&self) -> u32 {
        match self.accessory {
            Some(AccessoryType::ManaEarrings) => 15,
            _ => 0,
        }
    }
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
    pub dodge_animation: u32,
    pub guard_timer: u32,
    pub enemy_shake: Vec<u32>,
    pub projectiles: Vec<Projectile>,
    pub attack_effects: Vec<AttackEffect>,
    pub damage_numbers: Vec<DamageNumber>,
    pub enemy_attack_interval_scale: u32,
    pub no_damage_mode: bool,
    pub exp: u32,
    pub next_level_exp: u32,
    pub equipment: Equipment,
    pub current_room: Option<usize>,  // 現在いる部屋のインデックス
    pub inventory: [u32; 8],  // ItemType ごとの数量（8 種類）
    pub eq_inventory: EquipmentInventory,  // ドロップした装備
    floor_states: Vec<Option<FloorState>>,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnemyVariant {
    Weak = 0,    // 薄い色：HP×0.6, ATK×0.7
    Normal = 1,  // 標準：HP×1.0, ATK×1.0
    Strong = 2,  // 濃い色：HP×1.2, ATK×1.2
    Boss = 3,    // ボス：HP×2.0, ATK×1.5
}

#[derive(Clone)]
pub struct EnemyData {
    pub name: &'static str,
    pub base_hp: u32,
    pub base_atk: u32,
    pub base_color: (f32, f32, f32),  // RGB
    pub drop_rate: u32,               // 0-100
    pub min_depth: u32,
    pub max_depth: u32,
}

pub struct Enemy {
    pub x: i32,
    pub y: i32,
    pub hp: u32,
    pub max_hp: u32,
    pub color: [f32; 3],
    pub name: String,
    pub enemy_type: u32,  // 敵のマスターテーブルインデックス
    pub variant: EnemyVariant,
    pub atk: u32,
    pub def: u32,  // 防御力
    pub drop_rate: u32,
    pub is_boss: bool,  // ボス敵フラグ
    pub pursuit_timer: u32,  // 60フレーム毎の移動カウンター
    pub can_see_player: bool,  // プレイヤーを見かけたか
    pub attack_cooldown: u32,
}

// 敵マスターテーブル（敵タイプの定義）
const ENEMY_MASTER: &[EnemyData] = &[
    // Goblin (0-2)
    EnemyData { name: "Young Goblin", base_hp: 12, base_atk: 3, base_color: (0.6, 1.0, 0.6), drop_rate: 20, min_depth: 1, max_depth: 3 },
    EnemyData { name: "Goblin", base_hp: 20, base_atk: 5, base_color: (1.0, 1.0, 0.3), drop_rate: 30, min_depth: 1, max_depth: 5 },
    EnemyData { name: "Hobgoblin", base_hp: 28, base_atk: 7, base_color: (0.3, 0.8, 0.3), drop_rate: 40, min_depth: 3, max_depth: 7 },

    // Bat (3-5)
    EnemyData { name: "Young Bat", base_hp: 10, base_atk: 2, base_color: (0.7, 0.7, 0.7), drop_rate: 15, min_depth: 1, max_depth: 4 },
    EnemyData { name: "Bat", base_hp: 18, base_atk: 4, base_color: (0.5, 0.3, 0.8), drop_rate: 25, min_depth: 2, max_depth: 6 },
    EnemyData { name: "Giant Bat", base_hp: 26, base_atk: 6, base_color: (0.3, 0.1, 0.6), drop_rate: 35, min_depth: 5, max_depth: 10 },

    // Skeleton (6-8)
    EnemyData { name: "Skeleton", base_hp: 18, base_atk: 4, base_color: (0.9, 0.9, 0.9), drop_rate: 25, min_depth: 3, max_depth: 6 },
    EnemyData { name: "Skeleton Warrior", base_hp: 25, base_atk: 6, base_color: (1.0, 1.0, 1.0), drop_rate: 35, min_depth: 4, max_depth: 8 },
    EnemyData { name: "Skeleton Knight", base_hp: 35, base_atk: 9, base_color: (0.7, 0.7, 0.8), drop_rate: 45, min_depth: 6, max_depth: 12 },

    // Spider (9-11)
    EnemyData { name: "Young Spider", base_hp: 22, base_atk: 5, base_color: (0.8, 0.6, 0.2), drop_rate: 30, min_depth: 5, max_depth: 10 },
    EnemyData { name: "Spider", base_hp: 30, base_atk: 7, base_color: (1.0, 0.6, 0.1), drop_rate: 40, min_depth: 7, max_depth: 12 },
    EnemyData { name: "Giant Spider", base_hp: 40, base_atk: 10, base_color: (0.9, 0.4, 0.0), drop_rate: 50, min_depth: 10, max_depth: 15 },

    // Troll (12-14)
    EnemyData { name: "Young Troll", base_hp: 25, base_atk: 6, base_color: (0.5, 0.5, 0.5), drop_rate: 30, min_depth: 4, max_depth: 8 },
    EnemyData { name: "Troll", base_hp: 35, base_atk: 8, base_color: (0.7, 0.4, 0.7), drop_rate: 40, min_depth: 6, max_depth: 12 },
    EnemyData { name: "Troll King", base_hp: 48, base_atk: 12, base_color: (0.5, 0.2, 0.5), drop_rate: 50, min_depth: 10, max_depth: 18 },

    // Zombie (15-17)
    EnemyData { name: "Zombie", base_hp: 24, base_atk: 5, base_color: (0.3, 0.6, 0.3), drop_rate: 30, min_depth: 8, max_depth: 15 },
    EnemyData { name: "Zombie Warrior", base_hp: 32, base_atk: 7, base_color: (0.2, 0.5, 0.2), drop_rate: 40, min_depth: 10, max_depth: 18 },
    EnemyData { name: "Zombie Lord", base_hp: 44, base_atk: 11, base_color: (0.1, 0.3, 0.1), drop_rate: 50, min_depth: 15, max_depth: 25 },

    // Ghost (18-20)
    EnemyData { name: "Spirit", base_hp: 22, base_atk: 5, base_color: (0.6, 0.8, 1.0), drop_rate: 35, min_depth: 10, max_depth: 16 },
    EnemyData { name: "Ghost", base_hp: 30, base_atk: 7, base_color: (0.7, 0.9, 1.0), drop_rate: 45, min_depth: 11, max_depth: 20 },
    EnemyData { name: "Phantom", base_hp: 42, base_atk: 11, base_color: (0.4, 0.6, 0.9), drop_rate: 55, min_depth: 16, max_depth: 26 },

    // Mummy (21-23)
    EnemyData { name: "Mummy", base_hp: 32, base_atk: 7, base_color: (0.8, 0.7, 0.5), drop_rate: 40, min_depth: 12, max_depth: 18 },
    EnemyData { name: "Mummy Priest", base_hp: 40, base_atk: 9, base_color: (0.9, 0.8, 0.6), drop_rate: 45, min_depth: 14, max_depth: 22 },
    EnemyData { name: "Pharaoh", base_hp: 52, base_atk: 13, base_color: (0.7, 0.6, 0.3), drop_rate: 55, min_depth: 18, max_depth: 28 },

    // Ogre (24-26)
    EnemyData { name: "Young Ogre", base_hp: 35, base_atk: 8, base_color: (0.7, 0.6, 0.4), drop_rate: 40, min_depth: 14, max_depth: 20 },
    EnemyData { name: "Ogre", base_hp: 45, base_atk: 10, base_color: (0.8, 0.6, 0.3), drop_rate: 50, min_depth: 16, max_depth: 25 },
    EnemyData { name: "Ogre Warlord", base_hp: 58, base_atk: 14, base_color: (0.6, 0.4, 0.1), drop_rate: 60, min_depth: 20, max_depth: 30 },

    // Wyvern (27-29)
    EnemyData { name: "Young Wyvern", base_hp: 38, base_atk: 9, base_color: (1.0, 0.5, 0.3), drop_rate: 45, min_depth: 18, max_depth: 24 },
    EnemyData { name: "Wyvern", base_hp: 50, base_atk: 12, base_color: (1.0, 0.3, 0.1), drop_rate: 50, min_depth: 20, max_depth: 29 },
    EnemyData { name: "Hell Wyvern", base_hp: 65, base_atk: 16, base_color: (0.8, 0.1, 0.0), drop_rate: 60, min_depth: 24, max_depth: 30 },
];

impl RoguelikeGame {
    fn is_ranged_enemy_type(enemy_type: u32) -> bool {
        // Bats, spiders, zombie warriors, priests, and wyverns stay back and fire.
        matches!(enemy_type, 3 | 10 | 16 | 22 | 28)
    }

    fn calc_map_size(depth: u32) -> (i32, i32) {
        let width = 120 + ((depth.saturating_sub(1)) as i32 * 4);
        let height = 80 + ((depth.saturating_sub(1)) as i32 * 2);
        (width, height)
    }

    // 敵タイプから敵のステータスを計算
    fn get_enemy_stats(enemy_type: u32, variant: EnemyVariant) -> (u32, u32) {
        if enemy_type >= ENEMY_MASTER.len() as u32 {
            return (1, 1);
        }
        let data = &ENEMY_MASTER[enemy_type as usize];
        let (hp_mul, atk_mul) = match variant {
            EnemyVariant::Weak => (0.6, 0.7),
            EnemyVariant::Normal => (1.0, 1.0),
            EnemyVariant::Strong => (1.2, 1.2),
            EnemyVariant::Boss => (2.0, 1.5),
        };
        let hp = (data.base_hp as f32 * hp_mul) as u32;
        let atk = (data.base_atk as f32 * atk_mul) as u32;
        (hp.max(1), atk.max(1))
    }

    // 指定された深さから敵を生成
    fn spawn_random_enemy_for_floor(depth: u32, rng: &mut LcgRng) -> Option<(u32, EnemyVariant)> {
        let available: Vec<u32> = (0..ENEMY_MASTER.len() as u32)
            .filter(|&i| {
                let data = &ENEMY_MASTER[i as usize];
                depth >= data.min_depth && depth <= data.max_depth
            })
            .collect();

        if available.is_empty() {
            return None;
        }

        let enemy_type = available[(rng.next() as usize) % available.len()];

        // 敵のバリアント決定: 70% Normal, 20% Strong, 5% Boss, 5% Weak
        let roll = rng.next() % 100;
        let variant = if roll < 5 {
            EnemyVariant::Weak
        } else if roll < 25 {
            EnemyVariant::Strong
        } else if roll < 30 {
            EnemyVariant::Boss
        } else {
            EnemyVariant::Normal
        };

        Some((enemy_type, variant))
    }

    // F1, F10, F20, F30 でボス敵を生成
    fn should_spawn_boss(depth: u32) -> bool {
        depth == 1 || depth == 10 || depth == 20 || depth == 30
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
            messages: vec!["Entered the dungeon...".to_string()],
            map,
            map_width,
            map_height,
            rooms,
            visited,
            player_shake: 0,
            dodge_animation: 0,
            guard_timer: 0,
            enemy_shake: vec![],
            projectiles: vec![],
            attack_effects: vec![],
            damage_numbers: vec![],
            // Start in a relaxed, invulnerable practice mode; both remain adjustable in Settings.
            enemy_attack_interval_scale: 1000,
            no_damage_mode: true,
            exp: 0,
            next_level_exp: 100,
            equipment: Equipment::new(),
            current_room: None,
            inventory: [0; 8],  // HealthPotion, ManaPotion, PoisonPotion, EnergyDrink, Gem, SkeletonKey, Scroll, GoldenCoin
            eq_inventory: EquipmentInventory {
                // Keep the three prototype weapon styles available from the start.
                weapons: vec![
                    WeaponType::WoodenSword,
                    WeaponType::Spear,
                    WeaponType::Bow,
                    WeaponType::Staff,
                ],
                armors: Vec::new(),
                accessories: Vec::new(),
            },
            floor_states: vec![None; 31],
        }
    }

    fn spawn_enemies(&mut self, is_boss_floor: bool) {
        self.enemies.clear();
        self.enemy_shake.clear();
        let mut rng = LcgRng::new(self.depth.wrapping_mul(9999));

        // 各部屋に複数の敵を配置（階段のある部屋は除外）
        for i in 0..self.rooms.len().min(8) {
            let room = &self.rooms[i];

            // 部屋に階段があるかチェック
            let has_stairs = (room.y..room.y + room.height).any(|y| {
                (room.x..room.x + room.width).any(|x| {
                    matches!(self.map[y as usize][x as usize], TileType::StairUp | TileType::StairDown)
                })
            });

            if has_stairs {
                continue;  // 階段のある部屋はスキップ
            }

            let room_enemy_count = 2 + (rng.next() % 3) as usize;  // 2-4体の敵
            for _ in 0..room_enemy_count {
                if let Some((enemy_type, variant)) = Self::spawn_random_enemy_for_floor(self.depth, &mut rng) {

                    // 敵を配置する位置を探す（重複なし、歩行可能タイル）
                    let mut ex = room.x + (rng.next() as i32 % room.width);
                    let mut ey = room.y + (rng.next() as i32 % room.height);
                    let mut attempts = 0;
                    while attempts < 20 {
                        // その位置に既に敵がいないかチェック
                        let occupied = self.enemies.iter().any(|e| e.x == ex && e.y == ey);
                        if self.is_walkable(ex, ey) && !occupied {
                            break;
                        }
                        ex = room.x + (rng.next() as i32 % room.width);
                        ey = room.y + (rng.next() as i32 % room.height);
                        attempts += 1;
                    }

                    let is_boss = is_boss_floor && i == 0 && self.enemies.is_empty();
                    let (mut hp, mut atk) = Self::get_enemy_stats(enemy_type, variant);

                    if is_boss {
                        hp = (hp as f32 * 3.0) as u32;
                        atk = (atk as f32 * 2.0) as u32;
                    }

                    let data = &ENEMY_MASTER[enemy_type as usize];
                    let mut boss_name = data.name.to_string();
                    if is_boss {
                        boss_name = format!("☆{}", boss_name);
                    }

                    let color = if is_boss {
                        [1.0, 0.85, 0.0]
                    } else {
                        Self::apply_variant_color(data.base_color, variant)
                    };

                    let def = if is_boss { 10 } else { 0 };

                    self.enemies.push(Enemy {
                        x: ex,
                        y: ey,
                        hp,
                        max_hp: hp,
                        color,
                        name: boss_name,
                        enemy_type,
                        variant,
                        atk,
                        def,
                        drop_rate: if is_boss { 100 } else { data.drop_rate },
                        is_boss,
                        pursuit_timer: 0,
                        can_see_player: false,
                        attack_cooldown: 0,
                    });
                    self.enemy_shake.push(0);
                }
            }
        }

        // 道にも敵を配置（スパース配置）
        let corridor_enemy_count = 3 + (rng.next() % 3) as usize;
        for _ in 0..corridor_enemy_count {
            if let Some((enemy_type, variant)) = Self::spawn_random_enemy_for_floor(self.depth, &mut rng) {
                let mut ex = rng.next() as i32 % self.map_width;
                let mut ey = rng.next() as i32 % self.map_height;

                // 道タイルを探す
                for _ in 0..20 {
                    if self.is_walkable(ex, ey) && !matches!(self.map[ey as usize][ex as usize], TileType::Room) {
                        break;
                    }
                    ex = rng.next() as i32 % self.map_width;
                    ey = rng.next() as i32 % self.map_height;
                }

                if self.is_walkable(ex, ey) && !matches!(self.map[ey as usize][ex as usize], TileType::Room) {
                    let (hp, atk) = Self::get_enemy_stats(enemy_type, variant);
                    let data = &ENEMY_MASTER[enemy_type as usize];
                    let color = Self::apply_variant_color(data.base_color, variant);

                    self.enemies.push(Enemy {
                        x: ex,
                        y: ey,
                        hp,
                        max_hp: hp,
                        color,
                        name: data.name.to_string(),
                        enemy_type,
                        variant,
                        atk,
                        def: 0,
                        drop_rate: data.drop_rate,
                        is_boss: false,
                        pursuit_timer: 0,
                        can_see_player: false,
                        attack_cooldown: 0,
                    });
                    self.enemy_shake.push(0);
                }
            }
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
                // 部屋を配置（Room タイルとして）
                for ry in room_y..(room_y + room_height).min(height) {
                    for rx in room_x..(room_x + room_width).min(width) {
                        map[ry as usize][rx as usize] = TileType::Room;
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

            // 水平通路（既に Room なら上書きしない）
            let (start, end) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
            for x in start..=end {
                if x >= 0 && x < width && y1 >= 0 && y1 < height {
                    if map[y1 as usize][x as usize] != TileType::Room {
                        map[y1 as usize][x as usize] = TileType::Floor;
                    }
                }
            }

            // 垂直通路（既に Room なら上書きしない）
            let (start, end) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
            for y in start..=end {
                if x2 >= 0 && x2 < width && y >= 0 && y < height {
                    if map[y as usize][x2 as usize] != TileType::Room {
                        map[y as usize][x2 as usize] = TileType::Floor;
                    }
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

        // Add traversal holes to interior rooms. Normal movement enters a hole;
        // dodge movement can cross it and land on the other side.
        for room in rooms.iter().skip(1).take(5) {
            if room.width < 5 || room.height < 5 {
                continue;
            }
            let candidates = [
                (room.x + 2, room.y + 2),
                (room.x + room.width - 3, room.y + room.height - 3),
            ];
            for (pit_x, pit_y) in candidates {
                if map[pit_y as usize][pit_x as usize] == TileType::Room {
                    map[pit_y as usize][pit_x as usize] = TileType::Pit;
                }
            }
        }

        (map, rooms)
    }


    pub fn start_game(&mut self) {
        self.scene = RogueScene::Playing;
        self.messages.clear();
        self.messages.push("Game started!".to_string());
        self.projectiles.clear();
        self.attack_effects.clear();
        self.damage_numbers.clear();
        self.player_shake = 0;
        self.dodge_animation = 0;
        self.enemy_shake.clear();

        // 最初の部屋にプレイヤーを配置
        if !self.rooms.is_empty() {
            let room = &self.rooms[0];
            self.player_x = room.x + room.width / 2;
            self.player_y = room.y + room.height / 2;
            self.current_room = Some(0);
            self.add_message("📍 Entered Room #1".to_string());
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

        self.spawn_enemies(Self::should_spawn_boss(self.depth));
    }

    fn is_walkable(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.map_width || y < 0 || y >= self.map_height {
            return false;
        }
        matches!(self.map[y as usize][x as usize],
            TileType::Floor | TileType::Room | TileType::StairDown | TileType::StairUp)
    }

    fn is_pit(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.map_width && y >= 0 && y < self.map_height
            && self.map[y as usize][x as usize] == TileType::Pit
    }

    fn enter_pit(&mut self) {
        if self.depth >= 30 {
            self.add_message("🕳️ There is a pit here, but you cannot go any lower.".to_string());
            return;
        }
        self.next_floor();
        self.add_message("🕳️ Fell into a pit and dropped to the next floor.".to_string());
    }

    fn get_room_at(&self, x: i32, y: i32) -> Option<usize> {
        for (idx, room) in self.rooms.iter().enumerate() {
            if x >= room.x && x < room.x + room.width && y >= room.y && y < room.y + room.height {
                return Some(idx);
            }
        }
        None
    }

    fn weapon_name(weapon: WeaponType) -> &'static str {
        match weapon {
            WeaponType::WoodenSword => "Wooden Sword",
            WeaponType::IronSword => "Iron Sword",
            WeaponType::Spear => "Spear",
            WeaponType::Bow => "Bow",
            WeaponType::Staff => "Staff",
            WeaponType::CursedBlade => "Cursed Blade",
        }
    }

    fn armor_name(armor: ArmorType) -> &'static str {
        match armor {
            ArmorType::LeatherArmor => "Leather Armor",
            ArmorType::ChainMail => "Chain Mail",
            ArmorType::SteelPlate => "Steel Plate",
            ArmorType::DragonScale => "Dragon Scale",
            ArmorType::CursedMail => "Cursed Mail",
        }
    }

    fn accessory_name(accessory: AccessoryType) -> &'static str {
        match accessory {
            AccessoryType::GoldRing => "Gold Ring",
            AccessoryType::VampireRing => "Vampire Ring",
            AccessoryType::LuckyRing => "Lucky Ring",
            AccessoryType::HealingNecklace => "Healing Necklace",
            AccessoryType::ManaEarrings => "Mana Earrings",
        }
    }

    fn apply_variant_color(base_color: (f32, f32, f32), variant: EnemyVariant) -> [f32; 3] {
        let (r, g, b) = base_color;
        match variant {
            EnemyVariant::Weak => {
                // 薄い色（0.6倍）
                [(r * 0.6).min(1.0), (g * 0.6).min(1.0), (b * 0.6).min(1.0)]
            }
            EnemyVariant::Normal => {
                // 通常色
                [r, g, b]
            }
            EnemyVariant::Strong => {
                // 濃い色（1.2倍）
                [(r * 1.2).min(1.0), (g * 1.2).min(1.0), (b * 1.2).min(1.0)]
            }
            EnemyVariant::Boss => {
                // ボスは別で処理するので、ここでは使わない
                [r, g, b]
            }
        }
    }

    fn weapon_style(weapon: WeaponType) -> WeaponStyle {
        match weapon {
            WeaponType::WoodenSword | WeaponType::IronSword => WeaponStyle::Sword,
            WeaponType::Spear | WeaponType::CursedBlade => WeaponStyle::Spear,
            WeaponType::Bow => WeaponStyle::Bow,
            WeaponType::Staff => WeaponStyle::Staff,
        }
    }

    fn push_attack_effects(&mut self, cells: &[(i32, i32)], color: &'static str, ttl: u32) {
        for &(x, y) in cells {
            if x >= 0 && x < self.map_width && y >= 0 && y < self.map_height {
                self.attack_effects.push(AttackEffect {
                    x,
                    y,
                    ttl,
                    max_ttl: ttl,
                    color,
                });
            }
        }
    }

    fn push_damage_number(&mut self, x: i32, y: i32, amount: u32, color: &'static str) {
        self.damage_numbers.push(DamageNumber {
            x,
            y,
            amount,
            ttl: 30,
            max_ttl: 30,
            color,
        });
    }

    fn enemy_attack_adjacent(&mut self) {
        let attackers: Vec<(usize, String, u32)> = self.enemies.iter().enumerate()
            .filter(|(_, enemy)| {
                enemy.attack_cooldown == 0
                    && (enemy.x - self.player_x).abs() + (enemy.y - self.player_y).abs() == 1
            })
            .map(|(index, enemy)| (index, enemy.name.clone(), enemy.atk))
            .collect();

        let player_def = self.equipment.get_def_bonus() as i32;
        let guarding = self.guard_timer > 0
            && Self::weapon_style(self.equipment.weapon.unwrap_or(WeaponType::WoodenSword)) == WeaponStyle::Sword;
        for (enemy_index, enemy_name, enemy_atk) in attackers {
            let base_damage = enemy_atk as i32 + 3;
            let reduced_damage = (base_damage - (player_def * 80 / 100)).max(1) as u32;
            let damage = if guarding { (reduced_damage / 3).max(1) } else { reduced_damage };
            let final_damage = if self.no_damage_mode { 0 } else { damage };
            self.push_attack_effects(
                &[(self.player_x, self.player_y)],
                if self.no_damage_mode {
                    "rgba(120, 220, 255, 0.92)"
                } else if guarding {
                    "rgba(100, 180, 255, 0.92)"
                } else {
                    "rgba(255, 80, 80, 0.92)"
                },
                10,
            );
            if !self.no_damage_mode {
                self.hp = (self.hp as i32 - final_damage as i32).max(0) as u32;
            }
            self.push_damage_number(self.player_x, self.player_y, final_damage, if self.no_damage_mode { "#66ccff" } else { "#ff6666" });
            if self.no_damage_mode {
                self.add_message(format!("{} attack blocked by no-damage mode", enemy_name));
            } else if guarding {
                self.add_message(format!("Guard! {}'s attack reduced to {} damage.", enemy_name, damage));
            } else {
                self.add_message(format!("{} attacks! {} damage.", enemy_name, damage));
            }
            self.player_shake = 5;

            let (enemy_type, variant, is_boss) = {
                let enemy = &self.enemies[enemy_index];
                (enemy.enemy_type, enemy.variant, enemy.is_boss)
            };
            let base_interval = match enemy_type % 3 {
                0 => 18,
                1 => 28,
                _ => 38,
            };
            let variant_bonus = match variant {
                EnemyVariant::Weak => 0,
                EnemyVariant::Normal => 4,
                EnemyVariant::Strong => 10,
                EnemyVariant::Boss => 22,
            };
            let base_interval = if is_boss {
                60
            } else {
                base_interval + variant_bonus
            };
            self.enemies[enemy_index].attack_cooldown =
                (base_interval * self.enemy_attack_interval_scale / 100).max(1);
        }

        if self.hp == 0 {
            self.scene = RogueScene::GameOver;
            self.add_message("💀 Game Over".to_string());
        }

        self.enemy_attack_ranged();
    }

    fn has_clear_shot(&self, enemy: &Enemy) -> bool {
        let dx = self.player_x - enemy.x;
        let dy = self.player_y - enemy.y;
        if dx == 0 && dy.abs() <= 1 || dy == 0 && dx.abs() <= 1 {
            return false;
        }
        if dx != 0 && dy != 0 {
            return false;
        }

        let step_x = dx.signum();
        let step_y = dy.signum();
        let mut x = enemy.x + step_x;
        let mut y = enemy.y + step_y;
        while x != self.player_x || y != self.player_y {
            if !self.is_walkable(x, y) {
                return false;
            }
            x += step_x;
            y += step_y;
        }
        true
    }

    fn enemy_attack_ranged(&mut self) {
        let shooters: Vec<(usize, i32)> = self.enemies.iter().enumerate()
            .filter(|(_, enemy)| {
                Self::is_ranged_enemy_type(enemy.enemy_type)
                    && enemy.attack_cooldown == 0
                    && self.has_clear_shot(enemy)
            })
            .map(|(index, enemy)| (index, enemy.atk as i32 + 2))
            .collect();

        for (enemy_index, damage) in shooters {
            let (from_x, from_y) = {
                let enemy = &self.enemies[enemy_index];
                (enemy.x, enemy.y)
            };
            self.projectiles.push(Projectile {
                from_x: from_x as f64,
                from_y: from_y as f64,
                to_x: self.player_x as f64,
                to_y: self.player_y as f64,
                progress: 0.0,
                proj_type: 3,
                damage: damage.max(1) as u32,
                direction: 0,
            });
            self.enemies[enemy_index].attack_cooldown =
                (42 * self.enemy_attack_interval_scale / 100).max(1);
            self.add_message(format!("{} fires a ranged attack!", self.enemies[enemy_index].name));
        }
    }

    fn cast_staff_magic(&mut self) {
        if self.mp < 3 {
            self.add_message("Not enough MP.".to_string());
            return;
        }
        let (target_x, target_y) = self.bow_target();
        self.mp -= 3;
        self.projectiles.push(Projectile {
            from_x: self.player_x as f64,
            from_y: self.player_y as f64,
            to_x: target_x,
            to_y: target_y,
            progress: 0.0,
            proj_type: 1,
            damage: 10 + self.equipment.get_atk_bonus(),
            direction: self.player_direction,
        });
        self.add_message("Cast a spell from the staff! MP -3".to_string());
    }

    fn magic_target_from(&self, from_x: f64, from_y: f64, direction: i32) -> (f64, f64) {
        let mut x = from_x.floor() as i32;
        let mut y = from_y.floor() as i32;
        let (dx, dy) = match direction {
            0 => (0, -1),
            1 => (-1, 0),
            2 => (1, 0),
            3 => (0, 1),
            _ => (0, 0),
        };
        loop {
            let next_x = x + dx;
            let next_y = y + dy;
            if !self.is_walkable(next_x, next_y) {
                break;
            }
            x = next_x;
            y = next_y;
        }
        (x as f64, y as f64)
    }

    fn steer_magic_projectile(&mut self, direction: i32) {
        self.player_direction = direction;
        let projectile_index = self.projectiles.iter().rposition(|projectile| projectile.proj_type == 1);
        let Some(index) = projectile_index else {
            return;
        };

        let projectile = &self.projectiles[index];
        let current_x = projectile.from_x
            + (projectile.to_x - projectile.from_x) * projectile.progress;
        let current_y = projectile.from_y
            + (projectile.to_y - projectile.from_y) * projectile.progress;
        let (target_x, target_y) = self.magic_target_from(current_x, current_y, direction);
        let projectile = &mut self.projectiles[index];
        projectile.from_x = current_x;
        projectile.from_y = current_y;
        projectile.to_x = target_x;
        projectile.to_y = target_y;
        projectile.progress = 0.0;
        projectile.direction = direction;
    }

    fn sword_attack_cells(&self) -> Vec<(i32, i32)> {
        match self.player_direction {
            0 => vec![
                (self.player_x, self.player_y - 1),
                (self.player_x - 1, self.player_y - 1),
                (self.player_x + 1, self.player_y - 1),
                (self.player_x - 1, self.player_y),
                (self.player_x + 1, self.player_y),
            ],
            1 => vec![
                (self.player_x - 1, self.player_y),
                (self.player_x - 1, self.player_y - 1),
                (self.player_x - 1, self.player_y + 1),
                (self.player_x, self.player_y - 1),
                (self.player_x, self.player_y + 1),
            ],
            2 => vec![
                (self.player_x + 1, self.player_y),
                (self.player_x + 1, self.player_y - 1),
                (self.player_x + 1, self.player_y + 1),
                (self.player_x, self.player_y - 1),
                (self.player_x, self.player_y + 1),
            ],
            3 => vec![
                (self.player_x, self.player_y + 1),
                (self.player_x - 1, self.player_y + 1),
                (self.player_x + 1, self.player_y + 1),
                (self.player_x - 1, self.player_y),
                (self.player_x + 1, self.player_y),
            ],
            _ => vec![],
        }
    }

    fn spear_attack_cells(&self) -> Vec<(i32, i32)> {
        match self.player_direction {
            0 => vec![(self.player_x, self.player_y - 1), (self.player_x, self.player_y - 2)],
            1 => vec![(self.player_x - 1, self.player_y), (self.player_x - 2, self.player_y)],
            2 => vec![(self.player_x + 1, self.player_y), (self.player_x + 2, self.player_y)],
            3 => vec![(self.player_x, self.player_y + 1), (self.player_x, self.player_y + 2)],
            _ => vec![],
        }
    }

    fn bow_target(&self) -> (f64, f64) {
        let mut x = self.player_x;
        let mut y = self.player_y;
        loop {
            let (dx, dy) = match self.player_direction {
                0 => (0, -1),
                1 => (-1, 0),
                2 => (1, 0),
                3 => (0, 1),
                _ => (0, 0),
            };
            let next_x = x + dx;
            let next_y = y + dy;
            if next_x < 0 || next_x >= self.map_width || next_y < 0 || next_y >= self.map_height {
                break;
            }
            if self.map[next_y as usize][next_x as usize] == TileType::Wall {
                break;
            }
            x = next_x;
            y = next_y;
        }
        (x as f64, y as f64)
    }

    fn handle_attack_hit(&mut self, target_x: i32, target_y: i32, damage: u32) -> bool {
        for i in 0..self.enemies.len() {
            if self.enemies[i].x == target_x && self.enemies[i].y == target_y {
                let enemy_def = self.enemies[i].def as i32;
                let final_damage = ((damage as i32) - enemy_def).max(1) as u32;
                self.enemies[i].hp = (self.enemies[i].hp as i32 - final_damage as i32).max(0) as u32;
                let enemy_name = self.enemies[i].name.clone();

                self.add_message(format!("{} takes {} damage!", enemy_name, final_damage));
                self.push_damage_number(target_x, target_y, final_damage, "#ffd45c");
                if i < self.enemy_shake.len() {
                    self.enemy_shake[i] = 5;
                }

                if self.enemies[i].hp == 0 {
                    let is_boss = self.enemies[i].is_boss;
                    let exp_gain = if is_boss { 500 } else { 10 };

                    self.add_message(format!("{} defeated! +{} EXP", enemy_name, exp_gain));

                    let mut rng = LcgRng::new((self.depth as u32).wrapping_mul(12345).wrapping_add(self.enemies[i].x as u32));
                    let item_roll = rng.next() % 100;

                    if item_roll < 30 {
                        let potion_type = rng.next() % 4;
                        match potion_type {
                            0 => {
                                self.inventory[ItemType::HealthPotion as usize] += 1;
                                self.add_message("💚 Found a Health Potion!".to_string());
                            }
                            1 => {
                                self.inventory[ItemType::ManaPotion as usize] += 1;
                                self.add_message("💙 Found a Mana Potion!".to_string());
                            }
                            2 => {
                                self.inventory[ItemType::PoisonPotion as usize] += 1;
                                self.add_message("☠️ Found a Poison Potion!".to_string());
                            }
                            _ => {
                                self.inventory[ItemType::EnergyDrink as usize] += 1;
                                self.add_message("⚡ Found an Energy Drink!".to_string());
                            }
                        }
                    } else if item_roll < 60 {
                        self.inventory[ItemType::Gem as usize] += rng.next() % 3 + 1;
                        self.add_message("💎 Found gems!".to_string());
                    } else if item_roll < 80 {
                        self.inventory[ItemType::SkeletonKey as usize] += 1;
                        self.add_message("🔑 Found a key!".to_string());
                    } else if item_roll < 90 {
                        self.inventory[ItemType::Scroll as usize] += 1;
                        self.add_message("📜 Found a scroll!".to_string());
                    } else {
                        self.inventory[ItemType::GoldenCoin as usize] += rng.next() % 5 + 1;
                        self.add_message("🪙 Found gold coins!".to_string());
                    }

                    if is_boss {
                        let drop_type = rng.next() % 3;
                        match drop_type {
                            0 => {
                                let weapons = [WeaponType::IronSword, WeaponType::Spear, WeaponType::Bow, WeaponType::Staff];
                                let weapon = weapons[(rng.next() as usize) % weapons.len()];
                                self.eq_inventory.weapons.push(weapon);
                                self.add_message(format!("⚔️ Found {}", Self::weapon_name(weapon)));
                            }
                            1 => {
                                let armors = [ArmorType::LeatherArmor, ArmorType::ChainMail, ArmorType::SteelPlate];
                                let armor = armors[(rng.next() as usize) % armors.len()];
                                self.eq_inventory.armors.push(armor);
                                self.add_message(format!("🛡️ Found {}", Self::armor_name(armor)));
                            }
                            _ => {
                                let accessories = [AccessoryType::GoldRing, AccessoryType::LuckyRing, AccessoryType::HealingNecklace];
                                let accessory = accessories[(rng.next() as usize) % accessories.len()];
                                self.eq_inventory.accessories.push(accessory);
                                self.add_message(format!("💍 Found {}", Self::accessory_name(accessory)));
                            }
                        }
                    }

                    self.gain_exp(exp_gain);
                    self.enemies.remove(i);
                    self.enemy_shake.remove(i);
                }

                return true;
            }
        }
        false
    }

    pub fn move_player(&mut self, action: i32) {
        if self.scene != RogueScene::Playing {
            return;
        }

        // action: 0=up, 1=left, 2=right, 3=down, 4=attack, 5=dodge, 6-9=weapon slots, 14=guard
        if (6..=9).contains(&action) {
            let slot = (action - 6) as usize;
            if let Some(&weapon) = self.eq_inventory.weapons.get(slot) {
                self.equipment.weapon = Some(weapon);
                self.add_message(format!("⚔️ Switched to {}", Self::weapon_name(weapon)));
            } else {
                self.add_message("You do not have that weapon yet.".to_string());
            }
            return;
        }

        // Shift + arrow steers the most recent magic projectile without moving.
        if (10..=13).contains(&action) {
            self.steer_magic_projectile(action - 10);
            return;
        }

        if action == 14 {
            if Self::weapon_style(self.equipment.weapon.unwrap_or(WeaponType::WoodenSword)) == WeaponStyle::Sword {
                self.guard_timer = 30;
                self.add_message("Sword guard up! Blocking.".to_string());
            } else {
                self.add_message("Equip a sword to guard.".to_string());
            }
            return;
        }

        if action == 4 {
            let weapon = self.equipment.weapon.unwrap_or(WeaponType::WoodenSword);
            let style = Self::weapon_style(weapon);

            match style {
                WeaponStyle::Sword => {
                    let cells = self.sword_attack_cells();
                    self.push_attack_effects(&cells, "rgba(255, 235, 160, 0.90)", 10);
                    let damage = 10 + self.equipment.get_atk_bonus();
                    let mut hit_any = false;
                    for (x, y) in cells {
                        hit_any |= self.handle_attack_hit(x, y, damage);
                    }
                    if hit_any {
                        self.player_shake = 3;
                    }
                    self.add_message("Sword slash!".to_string());
                }
                WeaponStyle::Spear => {
                    let cells = self.spear_attack_cells();
                    self.push_attack_effects(&cells, "rgba(80, 255, 255, 0.90)", 8);
                    let damage = 12 + self.equipment.get_atk_bonus();
                    let mut hit_any = false;
                    for (x, y) in cells {
                        hit_any |= self.handle_attack_hit(x, y, damage);
                    }
                    if hit_any {
                        self.player_shake = 3;
                    }
                    self.add_message("Spear thrust!".to_string());
                }
                WeaponStyle::Bow => {
                    let (target_x, target_y) = self.bow_target();
                    self.projectiles.push(Projectile {
                        from_x: self.player_x as f64,
                        from_y: self.player_y as f64,
                        to_x: target_x,
                        to_y: target_y,
                        progress: 0.0,
                        proj_type: 2,
                        damage: 5 + self.equipment.get_atk_bonus(),
                        direction: self.player_direction,
                    });
                    self.add_message("Bow shot!".to_string());
                }
                WeaponStyle::Staff => {
                    self.cast_staff_magic();
                }
            }
            self.enemy_attack_adjacent();
            return;
        }

        let movement_direction = if action == 5 {
            self.player_direction
        } else {
            self.player_direction = action;
            action
        };
        let move_steps = if action == 5 { 3 } else { 1 };

        let (dx, dy) = match movement_direction {
            0 => (0, -1),
            1 => (-1, 0),
            2 => (1, 0),
            3 => (0, 1),
            _ => return,
        };

        let mut new_x = self.player_x;
        let mut new_y = self.player_y;
        let mut last_safe_x = self.player_x;
        let mut last_safe_y = self.player_y;
        for _ in 0..move_steps {
            let next_x = new_x + dx;
            let next_y = new_y + dy;
            if self.is_walkable(next_x, next_y) || self.is_pit(next_x, next_y) {
                new_x = next_x;
                new_y = next_y;
                if self.is_walkable(next_x, next_y) {
                    last_safe_x = next_x;
                    last_safe_y = next_y;
                }
            } else {
                break;
            }
        }

        // A dodge may cross a pit, but never ends by standing inside one.
        if action == 5 && self.is_pit(new_x, new_y) {
            new_x = last_safe_x;
            new_y = last_safe_y;
        }

        let moved = new_x != self.player_x || new_y != self.player_y;
        if action == 5 && moved {
            self.dodge_animation = 12;
            self.add_message("Spin dodge!".to_string());
        }

        // 敵への攻撃判定
        let mut attacked_enemy = false;
        for i in 0..self.enemies.len() {
            if self.enemies[i].x == new_x && self.enemies[i].y == new_y {
                // 敵に攻撃（DEFを考慮）
                let base_damage = 15u32;
                let enemy_def = self.enemies[i].def as i32;
                let damage = ((base_damage as i32) - enemy_def).max(1) as u32;  // 最小1ダメージ
                let old_hp = self.enemies[i].hp;
                self.push_damage_number(self.enemies[i].x, self.enemies[i].y, damage, "#ffd45c");
                self.enemies[i].hp = (self.enemies[i].hp as i32 - damage as i32).max(0) as u32;
                let enemy_name = self.enemies[i].name.clone();

                self.add_message(format!("{} takes {} damage!", enemy_name, damage));

                // 敵を震わせる
                if i < self.enemy_shake.len() {
                    self.enemy_shake[i] = 5;
                }

                if self.enemies[i].hp == 0 {
                    let is_boss = self.enemies[i].is_boss;
                    let exp_gain = if is_boss { 500 } else { 10 };

                    self.add_message(format!("{} defeated! +{} EXP", enemy_name, exp_gain));

                    // 敵からのドロップ（アイテム）
                    let mut rng = LcgRng::new((self.depth as u32).wrapping_mul(12345).wrapping_add(self.enemies[i].x as u32));
                    let item_roll = rng.next() % 100;

                    if item_roll < 30 {
                        // ポーション系（30%）
                        let potion_type = rng.next() % 4;
                        match potion_type {
                            0 => {
                                self.inventory[ItemType::HealthPotion as usize] += 1;
                                self.add_message("💚 Found a Health Potion!".to_string());
                            }
                            1 => {
                                self.inventory[ItemType::ManaPotion as usize] += 1;
                                self.add_message("💙 Found a Mana Potion!".to_string());
                            }
                            2 => {
                                self.inventory[ItemType::PoisonPotion as usize] += 1;
                                self.add_message("☠️ Found a Poison Potion!".to_string());
                            }
                            _ => {
                                self.inventory[ItemType::EnergyDrink as usize] += 1;
                                self.add_message("⚡ Found an Energy Drink!".to_string());
                            }
                        }
                    } else if item_roll < 60 {
                        // 宝石（30%）
                        self.inventory[ItemType::Gem as usize] += rng.next() % 3 + 1;
                        self.add_message("💎 Found gems!".to_string());
                    } else if item_roll < 80 {
                        // 鍵（20%）
                        self.inventory[ItemType::SkeletonKey as usize] += 1;
                        self.add_message("🔑 Found a key!".to_string());
                    } else if item_roll < 90 {
                        // スクロール（10%）
                        self.inventory[ItemType::Scroll as usize] += 1;
                        self.add_message("📜 Found a scroll!".to_string());
                    } else {
                        // コイン（10%）
                        self.inventory[ItemType::GoldenCoin as usize] += rng.next() % 5 + 1;
                        self.add_message("🪙 Found gold coins!".to_string());
                    }

                    // ボス撃破時の装備ドロップ
                    if is_boss {
                        // ランダムに装備をドロップ
                        let drop_type = rng.next() % 3;  // 武器、防具、アクセサリーから選択

                        match drop_type {
                            0 => {
                                // 武器ドロップ
                                let weapons = [WeaponType::IronSword, WeaponType::Spear, WeaponType::Bow, WeaponType::Staff];
                                let weapon = weapons[(rng.next() as usize) % weapons.len()];
                                self.eq_inventory.weapons.push(weapon);
                                self.add_message(format!("⚔️ Found {}", Self::weapon_name(weapon)));
                            }
                            1 => {
                                // 防具ドロップ
                                let armors = [ArmorType::LeatherArmor, ArmorType::ChainMail, ArmorType::SteelPlate];
                                let armor = armors[(rng.next() as usize) % armors.len()];
                                self.eq_inventory.armors.push(armor);
                                self.add_message(format!("🛡️ Found {}", Self::armor_name(armor)));
                            }
                            _ => {
                                // アクセサリードロップ
                                let accessories = [AccessoryType::GoldRing, AccessoryType::LuckyRing, AccessoryType::HealingNecklace];
                                let accessory = accessories[(rng.next() as usize) % accessories.len()];
                                self.eq_inventory.accessories.push(accessory);
                                self.add_message(format!("💍 Found {}", Self::accessory_name(accessory)));
                            }
                        }
                    }

                    self.gain_exp(exp_gain);
                    self.enemies.remove(i);
                    self.enemy_shake.remove(i);
                }

                attacked_enemy = true;
                break;
            }
        }

        if attacked_enemy {
            self.enemy_attack_adjacent();
            self.mark_visible();
            return;
        }

        if !moved {
            self.add_message("Bumped into a wall.".to_string());
            return;
        }

        // マップの壁判定と階段チェック
        let tile = self.map[new_y as usize][new_x as usize];

        if tile == TileType::Pit {
            self.enter_pit();
            return;
        }

        if tile == TileType::StairDown && self.depth < 30 {
            // 下り階段
            self.add_message(format!("⬇️ Moved down to F{}...", self.depth + 1));
            self.next_floor();
            return;
        }

        if tile == TileType::StairUp && self.depth > 1 {
            // 上り階段
            self.add_message(format!("⬆️ Moved up to F{}...", self.depth - 1));
            self.prev_floor();
            return;
        }

        if self.is_walkable(new_x, new_y) {
            self.player_x = new_x;
            self.player_y = new_y;

            // 部屋の出入り判定
            let new_room = self.get_room_at(new_x, new_y);
            if new_room != self.current_room {
                match new_room {
                    Some(room_idx) => {
                        self.add_message(format!("📍 Entered Room #{}", room_idx + 1));
                    }
                    None => {
                        self.add_message("Entered a corridor.".to_string());
                    }
                }
                self.current_room = new_room;
            }

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
                if Self::is_ranged_enemy_type(self.enemies[i].enemy_type) {
                    continue;
                }
                if self.is_walkable(new_ex, new_ey)
                    && (new_ex != self.player_x || new_ey != self.player_y)
                {
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

            self.enemy_attack_adjacent();

            // 訪問済みをマーク
            self.mark_visible();
        } else {
            self.add_message("Bumped into a wall.".to_string());
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
        self.save_floor_progress();
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
        self.add_message(format!("Level up! LV{}", self.level));
    }

    pub fn tick(&mut self, _ts: f64) {
        // 敵の AI 更新（追跡、プレイヤー検知）
        let enemy_moves: Vec<(usize, i32, i32)> = self.enemies.iter().enumerate().map(|(i, enemy)| {
            let dx = (self.player_x - enemy.x).abs();
            let dy = (self.player_y - enemy.y).abs();
            let distance = dx + dy;  // Manhattan distance

            // プレイヤーが視野内（15タイル以内）なら見かけた状態に
            let can_see = distance <= 15;

            // プレイヤーを見かけていたら追跡 AI
            let mut move_x = 0;
            let mut move_y = 0;
            if Self::is_ranged_enemy_type(enemy.enemy_type) {
                // Ranged enemies hold their position and attack from a clear line.
            } else if can_see || self.enemies[i].can_see_player {
                let pursuit_timer = self.enemies[i].pursuit_timer + 1;
                if pursuit_timer >= 60 {  // 60フレーム毎に移動
                    // プレイヤー方向に移動（Manhattan distance 最小化）
                    let mut best_move = (0, 0);
                    let mut best_distance = distance;

                    for (dx_test, dy_test) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let new_x = enemy.x + dx_test;
                        let new_y = enemy.y + dy_test;

                        if self.is_walkable(new_x, new_y) {
                            let new_distance = ((self.player_x - new_x).abs() + (self.player_y - new_y).abs()) as i32;
                            if new_distance < best_distance {
                                best_distance = new_distance;
                                best_move = (*dx_test, *dy_test);
                            }
                        }
                    }
                    move_x = best_move.0;
                    move_y = best_move.1;
                }
            }
            (i, move_x, move_y)
        }).collect();

        for (i, move_x, move_y) in enemy_moves {
            if move_x != 0 || move_y != 0 {
                let new_x = self.enemies[i].x + move_x;
                let new_y = self.enemies[i].y + move_y;

                // 新しい位置が歩行可能で、他の敵やプレイヤーが占有していないかチェック
                let occupied_by_other = self.enemies.iter().enumerate()
                    .any(|(j, e)| j != i && e.x == new_x && e.y == new_y);
                let occupied_by_player = self.player_x == new_x && self.player_y == new_y;

                if self.is_walkable(new_x, new_y) && !occupied_by_other && !occupied_by_player {
                    self.enemies[i].x = new_x;
                    self.enemies[i].y = new_y;
                }
            }
            let dx = (self.player_x - self.enemies[i].x).abs();
            let dy = (self.player_y - self.enemies[i].y).abs();
            if dx + dy <= 15 {
                self.enemies[i].can_see_player = true;
                self.enemies[i].pursuit_timer += 1;
            }
            if self.enemies[i].pursuit_timer >= 60 {
                self.enemies[i].pursuit_timer = 0;
            }
        }

        // Enemy attacks run from the game loop, so the player does not need to act first.
        self.enemy_attack_adjacent();

        // 震える時間を減らす
        if self.player_shake > 0 {
            self.player_shake -= 1;
        }

        for enemy in self.enemies.iter_mut() {
            if enemy.attack_cooldown > 0 {
                enemy.attack_cooldown -= 1;
            }
        }

        if self.dodge_animation > 0 {
            self.dodge_animation -= 1;
        }

        if self.guard_timer > 0 {
            self.guard_timer -= 1;
        }

        for shake in self.enemy_shake.iter_mut() {
            if *shake > 0 {
                *shake -= 1;
            }
        }

        for effect in self.attack_effects.iter_mut() {
            if effect.ttl > 0 {
                effect.ttl -= 1;
            }
        }

        for number in self.damage_numbers.iter_mut() {
            if number.ttl > 0 {
                number.ttl -= 1;
            }
        }

        // Update projectiles
        for projectile in self.projectiles.iter_mut() {
            let distance = ((projectile.to_x - projectile.from_x).powi(2)
                + (projectile.to_y - projectile.from_y).powi(2))
                .sqrt()
                .max(1.0);
            // Keep arrows at a constant tile-per-frame speed regardless of range.
            let tile_speed = if projectile.proj_type == 2 { 0.18 } else { 0.008 };
            projectile.progress += tile_speed / distance;
        }

        // Check projectile collision with enemies and damage them
        let mut hit_projectiles = std::collections::HashSet::new();
        let mut projectile_damage_numbers = Vec::new();
        let mut player_projectile_hits = Vec::new();
        for (proj_idx, projectile) in self.projectiles.iter().enumerate() {
            if (projectile.proj_type == 1 || projectile.proj_type == 2) && projectile.progress > 0.1 {
                let current_x = projectile.from_x + (projectile.to_x - projectile.from_x) * projectile.progress;
                let current_y = projectile.from_y + (projectile.to_y - projectile.from_y) * projectile.progress;
                let map_x = current_x as i32;
                let map_y = current_y as i32;

                // Check enemy collision and damage
                for i in 0..self.enemies.len() {
                    if self.enemies[i].x == map_x && self.enemies[i].y == map_y {
                        let projectile_damage = projectile.damage;
                        projectile_damage_numbers.push((map_x, map_y, projectile_damage));
                        self.enemies[i].hp = (self.enemies[i].hp as i32 - projectile_damage as i32).max(0) as u32;
                        hit_projectiles.insert(proj_idx);

                        if self.enemies[i].hp == 0 {
                            self.enemy_shake[i] = 5;
                        } else {
                            self.enemy_shake[i] = 3;
                        }
                        break;
                    }
                }
            } else if projectile.proj_type == 3 && projectile.progress > 0.1 {
                let current_x = projectile.from_x + (projectile.to_x - projectile.from_x) * projectile.progress;
                let current_y = projectile.from_y + (projectile.to_y - projectile.from_y) * projectile.progress;
                if current_x as i32 == self.player_x && current_y as i32 == self.player_y {
                    player_projectile_hits.push((proj_idx, projectile.damage));
                }
            }
        }

        for (x, y, amount) in projectile_damage_numbers {
            self.push_damage_number(x, y, amount, "#ffd45c");
        }

        for (proj_idx, base_damage) in player_projectile_hits {
            let player_def = self.equipment.get_def_bonus() as i32;
            let reduced_damage = (base_damage as i32 - (player_def * 80 / 100)).max(1) as u32;
            let guarding = self.guard_timer > 0
                && Self::weapon_style(self.equipment.weapon.unwrap_or(WeaponType::WoodenSword)) == WeaponStyle::Sword;
            let damage = if guarding { (reduced_damage / 3).max(1) } else { reduced_damage };
            let final_damage = if self.no_damage_mode { 0 } else { damage };
            self.push_attack_effects(
                &[(self.player_x, self.player_y)],
                if self.no_damage_mode { "rgba(120, 220, 255, 0.92)" } else { "rgba(255, 80, 80, 0.92)" },
                10,
            );
            if !self.no_damage_mode {
                self.hp = (self.hp as i32 - final_damage as i32).max(0) as u32;
            }
            self.push_damage_number(self.player_x, self.player_y, final_damage, if self.no_damage_mode { "#66ccff" } else { "#ff6666" });
            if self.no_damage_mode {
                self.add_message("Ranged attack blocked by no-damage mode".to_string());
            } else if guarding {
                self.add_message(format!("Guard! Ranged attack reduced to {} damage.", damage));
            } else {
                self.add_message(format!("Ranged attack hits! {} damage.", damage));
            }
            hit_projectiles.insert(proj_idx);
            self.player_shake = 5;
        }

        if self.hp == 0 {
            self.scene = RogueScene::GameOver;
            self.add_message("💀 Game Over".to_string());
        }

        // Add messages after the loop and gain exp
        for i in 0..self.enemies.len() {
            if self.enemy_shake[i] == 5 {
                self.add_message("Enemy defeated!".to_string());
                self.gain_exp(10);
                break;
            }
        }

        // Remove dead enemies
        self.enemies.retain(|e| e.hp > 0);
        self.enemy_shake.truncate(self.enemies.len());

        // Remove projectiles after a hit or when they reach their target.
        let mut projectile_index = 0;
        self.projectiles.retain(|p| {
            let current_index = projectile_index;
            projectile_index += 1;
            if hit_projectiles.contains(&current_index) {
                return false;
            }
            if p.proj_type == 1 || p.proj_type == 2 || p.proj_type == 3 {
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

        self.attack_effects.retain(|effect| effect.ttl > 0);
        self.damage_numbers.retain(|number| number.ttl > 0);
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

    fn save_current_floor(&mut self) {
        self.floor_states[self.depth as usize] = Some(FloorState {
            map: self.map.clone(),
            rooms: self.rooms.clone(),
            visited: self.visited.clone(),
        });
    }

    fn save_floor_progress(&mut self) {
        if let Some(saved) = self.floor_states[self.depth as usize].as_mut() {
            saved.visited = self.visited.clone();
        } else {
            self.save_current_floor();
        }
    }

    fn load_floor(&mut self, depth: u32) {
        if let Some(saved) = self.floor_states[depth as usize].clone() {
            self.map = saved.map;
            self.rooms = saved.rooms;
            self.visited = saved.visited;
        } else {
            let (map_width, map_height) = Self::calc_map_size(depth);
            let (map, rooms) = Self::generate_dungeon(map_width, map_height, depth);
            self.map = map;
            self.rooms = rooms;
            self.visited = vec![vec![false; map_width as usize]; map_height as usize];
            self.floor_states[depth as usize] = Some(FloorState {
                map: self.map.clone(),
                rooms: self.rooms.clone(),
                visited: self.visited.clone(),
            });
        }

        self.map_width = self.map.first().map(|row| row.len() as i32).unwrap_or(0);
        self.map_height = self.map.len() as i32;
        self.place_in_random_room(depth);
    }

    fn place_in_random_room(&mut self, depth: u32) {
        if self.rooms.is_empty() {
            return;
        }

        let mut rng = LcgRng::new(depth.wrapping_mul(7919).wrapping_add(self.player_x as u32));
        let room_index = (rng.next() as usize) % self.rooms.len();
        let room = self.rooms[room_index].clone();

        for _ in 0..30 {
            let x = room.x + 1 + (rng.next() % (room.width.saturating_sub(2) as u32)) as i32;
            let y = room.y + 1 + (rng.next() % (room.height.saturating_sub(2) as u32)) as i32;
            if self.is_walkable(x, y)
                && !self.enemies.iter().any(|enemy| enemy.x == x && enemy.y == y)
            {
                self.player_x = x;
                self.player_y = y;
                self.current_room = Some(room_index);
                self.mark_visible();
                return;
            }
        }

        self.player_x = room.x + room.width / 2;
        self.player_y = room.y + room.height / 2;
        self.current_room = Some(room_index);
        self.mark_visible();
    }

    fn reset_transition_state(&mut self) {
        self.projectiles.clear();
        self.attack_effects.clear();
        self.damage_numbers.clear();
        self.player_shake = 0;
        self.dodge_animation = 0;
        self.guard_timer = 0;
        self.enemy_shake.clear();
    }

    pub fn next_floor(&mut self) {
        if self.depth >= 30 {
            self.add_message("You are already on the deepest floor.".to_string());
            return;
        }

        self.save_current_floor();
        self.depth += 1;
        self.level += 1;
        self.hp = self.max_hp;
        self.mp = self.max_mp;
        self.messages.clear();
        self.messages.push(format!("Arrived at F{}", self.depth));
        self.reset_transition_state();

        self.load_floor(self.depth);

        self.spawn_enemies(Self::should_spawn_boss(self.depth));

        // スタート位置を訪問済みに
        if self.player_y >= 0 && self.player_y < self.map_height
            && self.player_x >= 0 && self.player_x < self.map_width {
            self.visited[self.player_y as usize][self.player_x as usize] = true;
        }
    }

    pub fn prev_floor(&mut self) {
        if self.depth <= 1 {
            self.add_message("You are on the surface.".to_string());
            return;
        }

        self.save_current_floor();
        self.depth -= 1;
        self.level = self.depth;
        self.hp = self.max_hp;
        self.mp = self.max_mp;
        self.messages.clear();
        self.messages.push(format!("Returned to F{}", self.depth));
        self.reset_transition_state();

        self.load_floor(self.depth);

        self.spawn_enemies(Self::should_spawn_boss(self.depth));

        // スタート位置を訪問済みに
        if self.player_y >= 0 && self.player_y < self.map_height
            && self.player_x >= 0 && self.player_x < self.map_width {
            self.visited[self.player_y as usize][self.player_x as usize] = true;
        }
    }

    pub fn jump_to_floor(&mut self, target_depth: u32) {
        if self.scene != RogueScene::Playing {
            self.add_message("Can only jump floors while playing".to_string());
            return;
        }

        let target_depth = target_depth.clamp(1, 30);
        if target_depth == self.depth {
            self.add_message(format!("Already on F{}", self.depth));
            return;
        }

        self.save_current_floor();
        self.depth = target_depth;
        self.level = target_depth;
        self.hp = self.max_hp;
        self.mp = self.max_mp;
        self.messages.clear();
        self.messages.push(format!("Jumped to F{}", self.depth));
        self.reset_transition_state();

        self.load_floor(self.depth);
        self.spawn_enemies(Self::should_spawn_boss(self.depth));

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
                let cell_size = (width as f64 / view_width as f64).max(1.0);

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
                            crate::state::TileType::Pit => "#080812",
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

                        if tile_type == crate::state::TileType::Pit {
                            ctx.save();
                            ctx.set_fill_style(&"#020205".into());
                            ctx.set_shadow_color("rgba(120, 40, 220, 0.8)");
                            ctx.set_shadow_blur(12.0);
                            ctx.begin_path();
                            ctx.arc(
                                screen_x + cell_w * 0.5,
                                screen_y + cell_h * 0.5,
                                cell_w * 0.32,
                                0.0,
                                std::f64::consts::PI * 2.0,
                            ).ok();
                            ctx.fill();
                            ctx.restore();
                        }
                    }
                }

                // Draw enemies
                // Attack effects are rendered as opaque glowing highlights so the
                // player can read weapon range at a glance.
                for effect in &game.attack_effects {
                    if effect.x >= camera_x && effect.x < camera_x + view_width
                        && effect.y >= camera_y && effect.y < camera_y + view_height
                    {
                        let screen_x = (effect.x - camera_x) as f64 * cell_w;
                        let screen_y = (effect.y - camera_y) as f64 * cell_h;
                        let ttl_ratio = if effect.max_ttl == 0 {
                            0.0
                        } else {
                            effect.ttl as f64 / effect.max_ttl as f64
                        };
                        let alpha = (0.30 + ttl_ratio * 0.55).clamp(0.0, 1.0);
                        let inset = 3.0 + (1.0 - ttl_ratio) * 3.0;

                        ctx.save();
                        ctx.set_global_alpha(alpha);
                        ctx.set_fill_style(&effect.color.into());
                        ctx.set_shadow_color(effect.color);
                        ctx.set_shadow_blur(18.0);
                        ctx.fill_rect(
                            screen_x + inset,
                            screen_y + inset,
                            (cell_w - inset * 2.0).max(1.0),
                            (cell_h - inset * 2.0).max(1.0),
                        );

                        ctx.set_global_alpha((alpha + 0.15).min(1.0));
                        ctx.set_fill_style(&"rgba(255,255,255,0.18)".into());
                        ctx.fill_rect(
                            screen_x + cell_w * 0.22,
                            screen_y + cell_h * 0.22,
                            cell_w * 0.56,
                            cell_h * 0.56,
                        );
                        ctx.restore();
                    }
                }

                // Draw enemies
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

                        // Draw enemy icon - select by enemy type (0-29)
                        let icon_id = format!("enemy-{}", enemy.enemy_type);
                        if let Ok(img_elem) = window.document().unwrap()
                            .get_element_by_id(&icon_id)
                            .unwrap()
                            .dyn_into::<web_sys::HtmlImageElement>()
                        {
                            let icon_x = screen_x + cell_w * 0.5 + shake_offset_x;
                            let icon_y = screen_y + cell_h * 0.5 + shake_offset_y;
                            let icon_size = if enemy.is_boss {
                                cell_w * 1.8  // ボスは2×2マスで表示（1.8倍）
                            } else {
                                cell_w * 0.6  // 通常敵
                            };

                            ctx.save();
                            ctx.translate(icon_x, icon_y).ok();

                            // ボス敵は金色の枠を描画
                            if enemy.is_boss {
                                ctx.set_stroke_style(&"gold".into());
                                ctx.set_line_width(2.0);
                                ctx.stroke_rect(-icon_size * 0.5, -icon_size * 0.5, icon_size, icon_size);
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
                    let dodge_progress = if game.dodge_animation > 0 {
                        1.0 - (game.dodge_animation as f64 / 12.0)
                    } else {
                        0.0
                    };
                    let jump_height = (dodge_progress * std::f64::consts::PI).sin() * cell_h * 0.45;
                    let icon_y = player_screen_y + cell_h * 0.5 + shake_offset_y - jump_height;
                    let icon_size = cell_w * 0.6;

                    ctx.save();
                    ctx.translate(icon_x, icon_y).ok();
                    if game.dodge_animation > 0 {
                        ctx.rotate(dodge_progress * std::f64::consts::TAU).ok();
                    }

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

                // Draw projectiles
                for projectile in game.projectiles.iter() {
                    let current_x = projectile.from_x + (projectile.to_x - projectile.from_x) * projectile.progress;
                    let current_y = projectile.from_y + (projectile.to_y - projectile.from_y) * projectile.progress;

                    let screen_x = (current_x - camera_x as f64) * cell_w;
                    let screen_y = (current_y - camera_y as f64) * cell_h;
                    let icon_x = screen_x + cell_w * 0.5;
                    let icon_y = screen_y + cell_h * 0.5;

                    ctx.save();

                    if projectile.proj_type == 2 {
                        let dx = projectile.to_x - projectile.from_x;
                        let dy = projectile.to_y - projectile.from_y;
                        let len = (dx * dx + dy * dy).sqrt().max(1.0);
                        let ux = dx / len;
                        let uy = dy / len;
                        let tail_x = icon_x - ux * cell_w * 0.22;
                        let tail_y = icon_y - uy * cell_h * 0.22;
                        let head_x = icon_x + ux * cell_w * 0.18;
                        let head_y = icon_y + uy * cell_h * 0.18;

                        ctx.set_stroke_style(&"rgba(255, 210, 90, 0.9)".into());
                        ctx.set_line_width(4.0);
                        ctx.begin_path();
                        ctx.move_to(tail_x, tail_y);
                        ctx.line_to(head_x, head_y);
                        ctx.stroke();

                        ctx.set_fill_style(&"rgba(255, 210, 90, 0.35)".into());
                        ctx.begin_path();
                        ctx.arc(icon_x, icon_y, cell_w * 0.07, 0.0, std::f64::consts::PI * 2.0).ok();
                        ctx.fill();

                        ctx.set_fill_style(&"rgba(255, 250, 220, 0.95)".into());
                        ctx.begin_path();
                        ctx.arc(head_x, head_y, cell_w * 0.04, 0.0, std::f64::consts::PI * 2.0).ok();
                        ctx.fill();
                    } else if projectile.proj_type == 3 {
                        // Enemy ranged attacks use a red bolt so they are distinct from staff magic.
                        let dx = projectile.to_x - projectile.from_x;
                        let dy = projectile.to_y - projectile.from_y;
                        let len = (dx * dx + dy * dy).sqrt().max(1.0);
                        let ux = dx / len;
                        let uy = dy / len;
                        ctx.set_stroke_style(&"rgba(255, 70, 70, 0.9)".into());
                        ctx.set_line_width(4.0);
                        ctx.begin_path();
                        ctx.move_to(icon_x - ux * cell_w * 0.25, icon_y - uy * cell_h * 0.25);
                        ctx.line_to(icon_x + ux * cell_w * 0.18, icon_y + uy * cell_h * 0.18);
                        ctx.stroke();
                        ctx.set_fill_style(&"rgba(255, 40, 40, 0.35)".into());
                        ctx.begin_path();
                        ctx.arc(icon_x, icon_y, cell_w * 0.18, 0.0, std::f64::consts::PI * 2.0).ok();
                        ctx.fill();
                        ctx.set_fill_style(&"#ff7777".into());
                        ctx.begin_path();
                        ctx.arc(icon_x, icon_y, cell_w * 0.09, 0.0, std::f64::consts::PI * 2.0).ok();
                        ctx.fill();
                    } else {
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
                    }

                    ctx.restore();
                }

                // Draw floating damage numbers above the hit target.
                for number in &game.damage_numbers {
                    if number.x >= camera_x && number.x < camera_x + view_width
                        && number.y >= camera_y && number.y < camera_y + view_height
                    {
                        let ttl_ratio = number.ttl as f64 / number.max_ttl as f64;
                        let rise = (1.0 - ttl_ratio) * cell_h * 0.65;
                        let screen_x = (number.x - camera_x) as f64 * cell_w + cell_w * 0.5;
                        let screen_y = (number.y - camera_y) as f64 * cell_h + cell_h * 0.35 - rise;

                        ctx.save();
                        ctx.set_global_alpha(ttl_ratio.clamp(0.0, 1.0));
                        ctx.set_font("bold 16px monospace");
                        ctx.set_text_align("center");
                        ctx.set_stroke_style(&"rgba(0, 0, 0, 0.9)".into());
                        ctx.set_line_width(3.0);
                        ctx.stroke_text(&format!("-{}", number.amount), screen_x, screen_y).ok();
                        ctx.set_fill_style(&number.color.into());
                        ctx.fill_text(&format!("-{}", number.amount), screen_x, screen_y).ok();
                        ctx.restore();
                    }
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
