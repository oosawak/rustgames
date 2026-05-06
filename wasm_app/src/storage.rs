// storage.rs — localStorageを使ったスコア・設定の永続化

const KEY_BEST_STEPS: &str  = "maze3d_best_steps";
const KEY_MAX_LEVEL: &str   = "maze3d_max_level";
const KEY_TOTAL_PLAYS: &str = "maze3d_total_plays";
const KEY_VOL_SE: &str      = "maze3d_vol_se";
const KEY_VOL_AMB: &str     = "maze3d_vol_amb";

fn get_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

fn load_u32(key: &str, default: u32) -> u32 {
    get_storage()
        .and_then(|s| s.get_item(key).ok()?)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn load_f32(key: &str, default: f32) -> f32 {
    get_storage()
        .and_then(|s| s.get_item(key).ok()?)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn save_str(key: &str, val: &str) {
    if let Some(s) = get_storage() {
        let _ = s.set_item(key, val);
    }
}

pub fn save_best_score(total_steps: u32, level: u32) {
    save_str(KEY_BEST_STEPS, &total_steps.to_string());
    save_str(KEY_MAX_LEVEL, &level.to_string());
}

pub fn load_best_score() -> (u32, u32) {
    let best_steps = load_u32(KEY_BEST_STEPS, 0);
    let max_level  = load_u32(KEY_MAX_LEVEL, 0);
    (best_steps, max_level)
}

pub fn increment_play_count() {
    let count = load_u32(KEY_TOTAL_PLAYS, 0);
    save_str(KEY_TOTAL_PLAYS, &(count + 1).to_string());
}

pub fn load_play_count() -> u32 {
    load_u32(KEY_TOTAL_PLAYS, 0)
}

pub fn save_audio_volume(se: f32, amb: f32) {
    save_str(KEY_VOL_SE,  &se.to_string());
    save_str(KEY_VOL_AMB, &amb.to_string());
}

pub fn load_audio_volume() -> (f32, f32) {
    let se  = load_f32(KEY_VOL_SE,  1.0);
    let amb = load_f32(KEY_VOL_AMB, 0.4);
    (se, amb)
}
