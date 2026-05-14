// エントリーポイント: モジュール宣言と #[wasm_bindgen] エクスポート関数のみを含む薄いファイル

pub mod msx;
pub mod constants;
pub mod shader;
pub mod math;
pub mod particle;
pub mod maze;
pub mod geometry;
pub mod gpu;
pub mod game;
pub mod audio;
pub mod camera;
pub mod engine;
pub mod input;
pub mod enemy;
pub mod theme;
pub mod scene;
pub mod storage;
pub mod audio_tool;
pub mod font;
pub mod blaster;
pub mod earthdef;
pub mod pacman;

use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use game::GameState;

thread_local! {
    static STATE: RefCell<Option<GameState>> = RefCell::new(None);
}

#[wasm_bindgen]
pub async fn init_maze3d(canvas_id: &str) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let s = GameState::new(canvas_id).await.map_err(|e| JsValue::from_str(&e))?;
    STATE.with(|st| *st.borrow_mut() = Some(s));
    Ok(())
}

#[wasm_bindgen] pub fn tick_maze3d(ts:f64){ STATE.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.tick(ts);}}); }
#[wasm_bindgen] pub fn move_maze3d(a:i32){ STATE.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.act(a);}}); }
#[wasm_bindgen] pub fn next_level_maze3d(){ STATE.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.next_level();}}); }
#[wasm_bindgen] pub fn reset_maze3d(){ STATE.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.reset();}}); }
#[wasm_bindgen] pub fn start_game_maze3d(){ STATE.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.start_game();}}); }
#[wasm_bindgen] pub fn scene_maze3d() -> u8 { STATE.with(|s| s.borrow().as_ref().map(|g| g.scene.as_u8()).unwrap_or(0)) }
#[wasm_bindgen] pub fn steps_maze3d()->u32       { STATE.with(|s|s.borrow().as_ref().map(|g|g.steps).unwrap_or(0)) }
#[wasm_bindgen] pub fn total_steps_maze3d()->u32 { STATE.with(|s|s.borrow().as_ref().map(|g|g.total_steps).unwrap_or(0)) }
#[wasm_bindgen] pub fn level_maze3d()->u32       { STATE.with(|s|s.borrow().as_ref().map(|g|g.level).unwrap_or(1)) }
#[wasm_bindgen] pub fn level_clear_maze3d()->bool{ STATE.with(|s|s.borrow().as_ref().map(|g|g.level_clear).unwrap_or(false)) }
#[wasm_bindgen] pub fn warp_maze3d()->f32        { STATE.with(|s|s.borrow().as_ref().map(|g|g.warp_amount()).unwrap_or(0.0)) }
#[wasm_bindgen] pub fn warp_done_maze3d()->bool  { STATE.with(|s|s.borrow().as_ref().map(|g|g.level_clear && g.warp_timer>=1.5).unwrap_or(false)) }

/// 音声イベントフラグ (0=なし 1=足音 2=壁衝突 3=レベルクリア 4=ゴール付近)
#[wasm_bindgen]
pub fn audio_event_maze3d() -> u8 {
    STATE.with(|s| s.borrow_mut().as_mut().map(|g| g.audio.consume() as u8).unwrap_or(0))
}

/// 足音の左右パリティ (true=左足)
#[wasm_bindgen]
pub fn audio_step_parity_maze3d() -> bool {
    STATE.with(|s| s.borrow().as_ref().map(|g| g.audio.step_parity).unwrap_or(false))
}

#[wasm_bindgen]
pub fn game_over_maze3d() -> bool {
    STATE.with(|s| s.borrow().as_ref().map(|g| g.game_over).unwrap_or(false))
}

#[wasm_bindgen]
pub fn theme_name_maze3d() -> String {
    STATE.with(|s| s.borrow().as_ref()
        .map(|g| crate::theme::get_theme(g.level).name.to_string())
        .unwrap_or_default())
}

#[wasm_bindgen] pub fn best_steps_maze3d() -> u32  { storage::load_best_score().0 }
#[wasm_bindgen] pub fn best_level_maze3d() -> u32  { storage::load_best_score().1 }
#[wasm_bindgen] pub fn play_count_maze3d() -> u32  { storage::load_play_count() }
#[wasm_bindgen] pub fn load_se_vol_maze3d()  -> f32 { storage::load_audio_volume().0 }
#[wasm_bindgen] pub fn load_amb_vol_maze3d() -> f32 { storage::load_audio_volume().1 }
#[wasm_bindgen] pub fn save_audio_vol_maze3d(se: f32, amb: f32) { storage::save_audio_volume(se, amb); }

#[wasm_bindgen]
pub fn enemy_x_maze3d() -> f32 {
    STATE.with(|s| s.borrow().as_ref().map(|g| g.enemy.vis_x).unwrap_or(-1.0))
}

#[wasm_bindgen]
pub fn enemy_z_maze3d() -> f32 {
    STATE.with(|s| s.borrow().as_ref().map(|g| g.enemy.vis_z).unwrap_or(-1.0))
}

// ── ミニマップ用エクスポート ───────────────────────────────────────────────────
// 迷路セルフラグ (N=1,E=2,S=4,W=8) を MAZE_W×MAZE_H の平坦なVec<u8>で返す
#[wasm_bindgen] pub fn maze_data_maze3d() -> Vec<u8> {
    STATE.with(|s| s.borrow().as_ref().map(|g| g.maze.cells.to_vec()).unwrap_or_default())
}
#[wasm_bindgen] pub fn player_x_maze3d()      -> u32 { STATE.with(|s|s.borrow().as_ref().map(|g|g.px as u32).unwrap_or(0)) }
#[wasm_bindgen] pub fn player_z_maze3d()      -> u32 { STATE.with(|s|s.borrow().as_ref().map(|g|g.pz as u32).unwrap_or(0)) }
#[wasm_bindgen] pub fn player_facing_maze3d() -> u8  { STATE.with(|s|s.borrow().as_ref().map(|g|g.facing).unwrap_or(4)) }

/// AudioEventに対応するサウンド定義JSONを返す
/// event: 1=step_left, 2=step_right, 3=wall_hit, 4=level_clear, 5=goal_near, 6=enemy_near, 7=game_over
#[wasm_bindgen]
pub fn sound_def_maze3d(event: u8) -> String {
    use crate::audio_tool::*;
    match event {
        1 => sound_step_left().to_json(),
        2 => sound_step_right().to_json(),
        3 => sound_wall_hit().to_json(),
        4 => sound_level_clear().to_json(),
        5 => sound_goal_near().to_json(),
        6 => sound_enemy_near().to_json(),
        7 => sound_game_over().to_json(),
        _ => "{}".to_string(),
    }
}

/// 全サウンド定義をJSON配列で返す（デバッグ・ツール用）
#[wasm_bindgen]
pub fn all_sound_defs_maze3d() -> String {
    use crate::audio_tool::*;
    let defs = vec![
        sound_step_left(), sound_step_right(), sound_wall_hit(),
        sound_level_clear(), sound_goal_near(), sound_enemy_near(), sound_game_over(),
    ];
    format!("[{}]", defs.iter().map(|d| d.to_json()).collect::<Vec<_>>().join(","))
}

// ─── フォント埋め込みエクスポート ────────────────────────────────────────────

/// Regular フォントバイト列を返す。
/// `embed-font` feature でビルドした場合のみデータが入る。
/// feature なしビルドでは長さ0の Uint8Array が返り、
/// JS 側は外部ファイル（docs/fonts/）へフォールバックする。
#[wasm_bindgen]
pub fn engine_font_regular() -> Vec<u8> {
    crate::font::regular_bytes().to_vec()
}

/// Bold フォントバイト列を返す（embed-font 時のみ）。
#[wasm_bindgen]
pub fn engine_font_bold() -> Vec<u8> {
    crate::font::bold_bytes().to_vec()
}

/// フォントが WASM バイナリに埋め込まれているかどうかを返す。
/// JS 初期化時にこの値で分岐すること。
#[wasm_bindgen]
pub fn engine_font_embedded() -> bool {
    crate::font::is_embedded()
}

// ─── Neon Blast 3D ──────────────────────────────────────────────────────────

thread_local! {
    static BLASTER: std::cell::RefCell<Option<blaster::BlasterGame>> = std::cell::RefCell::new(None);
}

#[wasm_bindgen]
pub async fn init_blaster3d(canvas_id: &str) {
    console_error_panic_hook::set_once();
    let game = blaster::BlasterGame::new(canvas_id).await.unwrap();
    BLASTER.with(|s| *s.borrow_mut() = Some(game));
}

#[wasm_bindgen] pub fn tick_blaster3d(ts: f64) { BLASTER.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.tick(ts);}}); }
#[wasm_bindgen] pub fn start_blaster3d() { BLASTER.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.start();}}); }
#[wasm_bindgen] pub fn move_blaster3d(dx:f32,dz:f32) { BLASTER.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.set_move(dx,dz);}}); }
#[wasm_bindgen] pub fn shoot_blaster3d(on:bool) { BLASTER.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.set_shoot(on);}}); }
#[wasm_bindgen] pub fn turret_rotate_blaster3d(rot:f32) { BLASTER.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.set_turret_rotate(rot);}}); }
#[wasm_bindgen] pub fn toggle_auto_fire_blaster3d() { BLASTER.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.toggle_auto_fire();}}); }
#[wasm_bindgen] pub fn auto_fire_blaster3d() -> bool { BLASTER.with(|s|s.borrow().as_ref().map(|g|g.auto_fire).unwrap_or(false)) }
#[wasm_bindgen] pub fn switch_camera_blaster3d() { BLASTER.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.switch_camera();}}); }
#[wasm_bindgen] pub fn scene_blaster3d() -> u8 { BLASTER.with(|s|s.borrow().as_ref().map(|g|g.scene_u8()).unwrap_or(0)) }
#[wasm_bindgen] pub fn camera_mode_blaster3d() -> u8 { BLASTER.with(|s|s.borrow().as_ref().map(|g|g.camera_u8()).unwrap_or(0)) }
#[wasm_bindgen] pub fn camera_name_blaster3d() -> String { BLASTER.with(|s|s.borrow().as_ref().map(|g|g.camera_name().to_string()).unwrap_or_default()) }
#[wasm_bindgen] pub fn player_hp_blaster3d() -> i32 { BLASTER.with(|s|s.borrow().as_ref().map(|g|g.player_hp).unwrap_or(0)) }
#[wasm_bindgen] pub fn player_max_hp_blaster3d() -> i32 { BLASTER.with(|s|s.borrow().as_ref().map(|g|g.player_max_hp).unwrap_or(5)) }
#[wasm_bindgen] pub fn score_blaster3d() -> u32 { BLASTER.with(|s|s.borrow().as_ref().map(|g|g.score).unwrap_or(0)) }
#[wasm_bindgen] pub fn wave_blaster3d() -> u32 { BLASTER.with(|s|s.borrow().as_ref().map(|g|g.wave).unwrap_or(0)) }
#[wasm_bindgen] pub fn boss_hp_blaster3d() -> i32 { BLASTER.with(|s|s.borrow().as_ref().map(|g|g.boss.hp).unwrap_or(0)) }
#[wasm_bindgen] pub fn boss_max_hp_blaster3d() -> i32 { BLASTER.with(|s|s.borrow().as_ref().map(|g|g.boss.max_hp).unwrap_or(60)) }
#[wasm_bindgen] pub fn bullet_count_blaster3d() -> u32 { BLASTER.with(|s|s.borrow().as_ref().map(|g|g.bullet_count()).unwrap_or(0)) }
#[wasm_bindgen] pub fn audio_event_blaster3d() -> u8 { BLASTER.with(|s|s.borrow().as_ref().map(|g|g.audio_event).unwrap_or(0)) }
#[wasm_bindgen] pub fn is_boss_wave_blaster3d() -> bool { BLASTER.with(|s|s.borrow().as_ref().map(|g|g.is_boss_wave).unwrap_or(false)) }

/// サウンド定義 JSON を返す
/// 1=shoot, 2=enemy_shoot, 3=enemy_hit, 4=explosion, 5=stage_clear, 6=player_hit, 7=boss_appear, 8=game_over
#[wasm_bindgen]
pub fn sound_def_blaster3d(event: u8) -> String {
    use crate::blaster::audio_defs::*;
    match event {
        1 => sound_shoot().to_json(),
        2 => sound_shoot().to_json(),
        3 => sound_enemy_hit().to_json(),
        4 => sound_explosion().to_json(),
        5 => sound_stage_clear().to_json(),
        6 => sound_player_hit().to_json(),
        7 => sound_boss_appear().to_json(),
        8 => sound_game_over().to_json(),
        _ => "{}".to_string(),
    }
}


// ─── Earth Defense ──────────────────────────────────────────────────────────

thread_local! {
    static EARTHDEF: std::cell::RefCell<Option<earthdef::EarthDefGame>> = std::cell::RefCell::new(None);
}

#[wasm_bindgen]
pub async fn init_earthdef(canvas_id: &str) {
    console_error_panic_hook::set_once();
    match earthdef::EarthDefGame::new(canvas_id).await {
        Ok(g) => EARTHDEF.with(|s| *s.borrow_mut() = Some(g)),
        Err(e) => web_sys::console::error_1(&e.into()),
    }
}
#[wasm_bindgen] pub fn tick_earthdef(ts: f64) { EARTHDEF.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.tick(ts);}}); }
#[wasm_bindgen] pub fn start_earthdef() { EARTHDEF.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.start();}}); }
#[wasm_bindgen] pub fn set_cam_input_earthdef(x:f32,y:f32) { EARTHDEF.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.set_cam_input(x,y);}}); }
#[wasm_bindgen] pub fn set_aim_input_earthdef(x:f32,y:f32) { EARTHDEF.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.set_aim_input(x,y);}}); }
#[wasm_bindgen] pub fn fire_earthdef() { EARTHDEF.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.fire();}}); }
#[wasm_bindgen] pub fn fire_at_screen_earthdef(nx:f32,ny:f32) { EARTHDEF.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.fire_at_screen(nx,ny);}}); }
#[wasm_bindgen] pub fn flash_bomb_earthdef() { EARTHDEF.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.flash_bomb();}}); }
#[wasm_bindgen] pub fn set_laser_type_earthdef(t:u8) { EARTHDEF.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.set_laser_type(t);}}); }
#[wasm_bindgen] pub fn set_cam_distance_earthdef(d:f32) { EARTHDEF.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.set_cam_distance(d);}}); }
#[wasm_bindgen] pub fn scene_earthdef() -> u8 { EARTHDEF.with(|s|s.borrow().as_ref().map(|g|g.scene as u8).unwrap_or(0)) }
#[wasm_bindgen] pub fn score_earthdef() -> u32 { EARTHDEF.with(|s|s.borrow().as_ref().map(|g|g.score).unwrap_or(0)) }
#[wasm_bindgen] pub fn wave_earthdef() -> u32 { EARTHDEF.with(|s|s.borrow().as_ref().map(|g|g.wave).unwrap_or(0)) }
#[wasm_bindgen] pub fn earth_hp_earthdef() -> i32 { EARTHDEF.with(|s|s.borrow().as_ref().map(|g|g.earth_hp).unwrap_or(0)) }
#[wasm_bindgen] pub fn earth_max_hp_earthdef() -> i32 { EARTHDEF.with(|s|s.borrow().as_ref().map(|g|g.earth_max_hp).unwrap_or(100)) }
#[wasm_bindgen] pub fn flash_charges_earthdef() -> u32 { EARTHDEF.with(|s|s.borrow().as_ref().map(|g|g.flash_charges).unwrap_or(0)) }
#[wasm_bindgen] pub fn laser_type_earthdef() -> u8 { EARTHDEF.with(|s|s.borrow().as_ref().map(|g|g.laser_type as u8).unwrap_or(0)) }
#[wasm_bindgen] pub fn audio_event_earthdef() -> u8 { EARTHDEF.with(|s|s.borrow().as_ref().map(|g|g.audio_event).unwrap_or(0)) }

// ---- Pac-Man ----
thread_local! {
    static PACMAN: RefCell<Option<pacman::PacmanGame>> = RefCell::new(None);
}

#[wasm_bindgen]
pub fn init_pacman() {
    use wasm_bindgen::JsCast;
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("pacman-canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();
    let ctx = canvas.get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    let cell = 20.0_f64;
    PACMAN.with(|p| {
        *p.borrow_mut() = Some(pacman::PacmanGame::new(ctx, cell));
    });
}

#[wasm_bindgen]
pub fn tick_pacman(dt: f32) {
    PACMAN.with(|p| {
        if let Some(g) = p.borrow_mut().as_mut() {
            g.tick(dt);
        }
    });
}

#[wasm_bindgen]
pub fn draw_pacman() {
    PACMAN.with(|p| {
        if let Some(g) = p.borrow().as_ref() {
            g.draw();
        }
    });
}

#[wasm_bindgen]
pub fn set_input_pacman(dir: i32) {
    PACMAN.with(|p| {
        if let Some(g) = p.borrow_mut().as_mut() {
            g.set_input(dir);
        }
    });
}

#[wasm_bindgen]
pub fn score_pacman() -> u32 {
    PACMAN.with(|p| p.borrow().as_ref().map(|g| g.score).unwrap_or(0))
}

#[wasm_bindgen]
pub fn lives_pacman() -> u8 {
    PACMAN.with(|p| p.borrow().as_ref().map(|g| g.lives).unwrap_or(0))
}

/// Returns game phase: 0=Playing, 1=Dying, 2=GameOver, 3=LevelClear
#[wasm_bindgen]
pub fn phase_pacman() -> u8 {
    PACMAN.with(|p| p.borrow().as_ref().map(|g| match g.state {
        pacman::GamePhase::Playing   => 0,
        pacman::GamePhase::Dying     => 1,
        pacman::GamePhase::GameOver  => 2,
        pacman::GamePhase::LevelClear => 3,
    }).unwrap_or(0))
}

#[wasm_bindgen]
pub fn audio_event_pacman() -> u8 {
    PACMAN.with(|p| p.borrow().as_ref().map(|g| g.audio_event).unwrap_or(0))
}

// ── MSX エミュレータ ─────────────────────────────────────────────────────────

thread_local! {
    static MSX: std::cell::RefCell<Option<msx::MsxState>> = std::cell::RefCell::new(None);
}

#[wasm_bindgen]
pub fn init_msx() {
    console_error_panic_hook::set_once();
    MSX.with(|s| *s.borrow_mut() = Some(msx::MsxState::new()));
}

#[wasm_bindgen]
pub fn load_bios_msx(data: &[u8]) {
    MSX.with(|s| { if let Some(m) = s.borrow_mut().as_mut() { m.bus.load_bios(data); } });
}

#[wasm_bindgen]
pub fn load_sub_rom_msx(data: &[u8]) {
    MSX.with(|s| { if let Some(m) = s.borrow_mut().as_mut() { m.bus.load_sub_rom(data); } });
}

#[wasm_bindgen]
pub fn load_rom_msx(data: &[u8]) {
    MSX.with(|s| { if let Some(m) = s.borrow_mut().as_mut() { m.bus.load_rom(data); } });
}

#[wasm_bindgen]
pub fn fast_boot_msx(frames: u32) {
    MSX.with(|s| {
        if let Some(m) = s.borrow_mut().as_mut() {
            for _ in 0..frames {
                m.tick_frame();
                // BIOS HALT loop at $108A: DEC B / JR NZ, $108A / HALT
                // BIOS patch sets B=1 at $0DD4, so DEC B→B=0→JR NZ skips→HALT executes
                // No manual intervention needed — BIOS patch handles it
            }
        }
    });
}

#[wasm_bindgen]
pub fn tick_msx() {
    MSX.with(|s| { if let Some(m) = s.borrow_mut().as_mut() { m.tick_frame(); } });
}

#[wasm_bindgen]
pub fn frame_buffer_msx() -> Vec<u8> {
    MSX.with(|s| s.borrow().as_ref().map(|m| m.frame_buffer.clone()).unwrap_or_default())
}

#[wasm_bindgen]
pub fn audio_samples_msx() -> Vec<f32> {
    MSX.with(|s| s.borrow_mut().as_mut().map(|m| m.bus.psg.take_samples()).unwrap_or_default())
}

#[wasm_bindgen]
pub fn key_down_msx(code: &str) {
    MSX.with(|s| { if let Some(m) = s.borrow_mut().as_mut() { m.bus.keyboard.key_down(code); } });
}

#[wasm_bindgen]
pub fn key_up_msx(code: &str) {
    MSX.with(|s| { if let Some(m) = s.borrow_mut().as_mut() { m.bus.keyboard.key_up(code); } });
}

#[wasm_bindgen]
pub fn debug_info_msx() -> String {
    MSX.with(|s| {
                if let Some(m) = s.borrow().as_ref() {
            let regs = &m.bus.vdp.regs;
            let ss = m.bus.slot_select;
            let pc = m.cpu.pc;
            let sp = m.cpu.sp;
            // CPU registers
            let a = m.cpu.a; let b = m.cpu.b; let c = m.cpu.c; let d = m.cpu.d;
            let e = m.cpu.e; let h = m.cpu.h; let l = m.cpu.l; let f = m.cpu.f;
            let halted = m.cpu.halted;
            // Sample name table ($1800-$1808) and pattern for 'L' ($0260-$0267)
            let nt: Vec<String> = (0..8).map(|i| format!("{:02X}", m.bus.vdp.vram[0x1800+i])).collect();
            let pt_l: Vec<String> = (0..8).map(|i| format!("{:02X}", m.bus.vdp.vram[0x0260+i])).collect();
            // $FCC1-$FCC4: sub-slot info table filled by C-BIOS $1043
            let fcc1 = m.bus.ram[0xFCC1];
            let fcc2 = m.bus.ram[0xFCC2];
            let fcc3 = m.bus.ram[0xFCC3];
            let fcc4 = m.bus.ram[0xFCC4];
            // $F380: RDSLT hook (set by C-BIOS init)
            let f380 = m.bus.ram[0xF380];
            format!(
                "VDP R0={:02X} R1={:02X} R2={:02X} R3={:02X} R4={:02X} R5={:02X} R6={:02X} R7={:02X} | \
                slot={:02X} PC={:04X} SP={:04X} | CPU A={:02X} B={:02X} C={:02X} D={:02X} E={:02X} H={:02X} L={:02X} F={:02X} halted={} |\
                NT:{} | PAT-L:{} | PL=({},{}) GM={} | FCC1={:02X}{:02X}{:02X}{:02X} F380={:02X}",
                regs[0], regs[1], regs[2], regs[3], regs[4], regs[5], regs[6], regs[7],
                ss, pc, sp,
                a, b, c, d, e, h, l, f, halted,
                nt.join(""),
                pt_l.join(""),
                m.bus.ram[0xE000], m.bus.ram[0xE001], m.bus.ram[0xE050],
                fcc1, fcc2, fcc3, fcc4, f380
            )
        } else {
            "no MSX state".to_string()
        }
    })
}

#[wasm_bindgen]
pub fn force_slot_select(val: u8) {
    MSX.with(|s| {
        if let Some(m) = s.borrow_mut().as_mut() {
            m.bus.slot_select = val;
            m.bus.ppi.port_a = val;
            m.bus.sub_slot_select = [0; 4];
            m.bus.add_log(format!("[FORCE] slot_select=${:02X}", val));
        }
    });
}

#[wasm_bindgen]
pub fn debug_log_msx() -> String {
    MSX.with(|s| {
        if let Some(m) = s.borrow().as_ref() {
            m.bus.debug_log.join("\n")
        } else {
            "no MSX state".to_string()
        }
    })
}

#[wasm_bindgen]
pub fn debug_log_clear_msx() {
    MSX.with(|s| {
        if let Some(m) = s.borrow_mut().as_mut() {
            m.bus.debug_log.clear();
        }
    });
}

