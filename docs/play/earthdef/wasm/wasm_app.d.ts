/* tslint:disable */
/* eslint-disable */

/**
 * 全サウンド定義をJSON配列で返す（デバッグ・ツール用）
 */
export function all_sound_defs_maze3d(): string;

export function audio_event_blaster3d(): number;

export function audio_event_earthdef(): number;

/**
 * 音声イベントフラグ (0=なし 1=足音 2=壁衝突 3=レベルクリア 4=ゴール付近)
 */
export function audio_event_maze3d(): number;

/**
 * 足音の左右パリティ (true=左足)
 */
export function audio_step_parity_maze3d(): boolean;

export function auto_fire_blaster3d(): boolean;

export function best_level_maze3d(): number;

export function best_steps_maze3d(): number;

export function boss_hp_blaster3d(): number;

export function boss_max_hp_blaster3d(): number;

export function bullet_count_blaster3d(): number;

export function camera_mode_blaster3d(): number;

export function camera_name_blaster3d(): string;

export function earth_hp_earthdef(): number;

export function earth_max_hp_earthdef(): number;

export function enemy_x_maze3d(): number;

export function enemy_z_maze3d(): number;

/**
 * Bold フォントバイト列を返す（embed-font 時のみ）。
 */
export function engine_font_bold(): Uint8Array;

/**
 * フォントが WASM バイナリに埋め込まれているかどうかを返す。
 * JS 初期化時にこの値で分岐すること。
 */
export function engine_font_embedded(): boolean;

/**
 * Regular フォントバイト列を返す。
 * `embed-font` feature でビルドした場合のみデータが入る。
 * feature なしビルドでは長さ0の Uint8Array が返り、
 * JS 側は外部ファイル（docs/fonts/）へフォールバックする。
 */
export function engine_font_regular(): Uint8Array;

export function fire_earthdef(): void;

export function flash_bomb_earthdef(): void;

export function flash_charges_earthdef(): number;

export function game_over_maze3d(): boolean;

export function init_blaster3d(canvas_id: string): Promise<void>;

export function init_earthdef(canvas_id: string): Promise<void>;

export function init_maze3d(canvas_id: string): Promise<void>;

export function is_boss_wave_blaster3d(): boolean;

export function laser_type_earthdef(): number;

export function level_clear_maze3d(): boolean;

export function level_maze3d(): number;

export function load_amb_vol_maze3d(): number;

export function load_se_vol_maze3d(): number;

export function maze_data_maze3d(): Uint8Array;

export function move_blaster3d(dx: number, dz: number): void;

export function move_maze3d(a: number): void;

export function next_level_maze3d(): void;

export function play_count_maze3d(): number;

export function player_facing_maze3d(): number;

export function player_hp_blaster3d(): number;

export function player_max_hp_blaster3d(): number;

export function player_x_maze3d(): number;

export function player_z_maze3d(): number;

export function reset_maze3d(): void;

export function save_audio_vol_maze3d(se: number, amb: number): void;

export function scene_blaster3d(): number;

export function scene_earthdef(): number;

export function scene_maze3d(): number;

export function score_blaster3d(): number;

export function score_earthdef(): number;

export function set_aim_input_earthdef(x: number, y: number): void;

export function set_cam_input_earthdef(x: number, y: number): void;

export function set_laser_type_earthdef(t: number): void;

export function shoot_blaster3d(on: boolean): void;

/**
 * サウンド定義 JSON を返す
 * 1=shoot, 2=enemy_shoot, 3=enemy_hit, 4=explosion, 5=stage_clear, 6=player_hit, 7=boss_appear, 8=game_over
 */
export function sound_def_blaster3d(event: number): string;

/**
 * AudioEventに対応するサウンド定義JSONを返す
 * event: 1=step_left, 2=step_right, 3=wall_hit, 4=level_clear, 5=goal_near, 6=enemy_near, 7=game_over
 */
export function sound_def_maze3d(event: number): string;

export function start_blaster3d(): void;

export function start_earthdef(): void;

export function start_game_maze3d(): void;

export function steps_maze3d(): number;

export function switch_camera_blaster3d(): void;

export function theme_name_maze3d(): string;

export function tick_blaster3d(ts: number): void;

export function tick_earthdef(ts: number): void;

export function tick_maze3d(ts: number): void;

export function toggle_auto_fire_blaster3d(): void;

export function total_steps_maze3d(): number;

export function turret_rotate_blaster3d(rot: number): void;

export function warp_done_maze3d(): boolean;

export function warp_maze3d(): number;

export function wave_blaster3d(): number;

export function wave_earthdef(): number;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly all_sound_defs_maze3d: () => [number, number];
    readonly audio_event_blaster3d: () => number;
    readonly audio_event_earthdef: () => number;
    readonly audio_event_maze3d: () => number;
    readonly audio_step_parity_maze3d: () => number;
    readonly auto_fire_blaster3d: () => number;
    readonly best_level_maze3d: () => number;
    readonly best_steps_maze3d: () => number;
    readonly boss_hp_blaster3d: () => number;
    readonly boss_max_hp_blaster3d: () => number;
    readonly bullet_count_blaster3d: () => number;
    readonly camera_mode_blaster3d: () => number;
    readonly camera_name_blaster3d: () => [number, number];
    readonly earth_hp_earthdef: () => number;
    readonly earth_max_hp_earthdef: () => number;
    readonly enemy_x_maze3d: () => number;
    readonly enemy_z_maze3d: () => number;
    readonly engine_font_bold: () => [number, number];
    readonly engine_font_embedded: () => number;
    readonly fire_earthdef: () => void;
    readonly flash_bomb_earthdef: () => void;
    readonly flash_charges_earthdef: () => number;
    readonly game_over_maze3d: () => number;
    readonly init_blaster3d: (a: number, b: number) => any;
    readonly init_earthdef: (a: number, b: number) => any;
    readonly init_maze3d: (a: number, b: number) => any;
    readonly is_boss_wave_blaster3d: () => number;
    readonly laser_type_earthdef: () => number;
    readonly level_clear_maze3d: () => number;
    readonly level_maze3d: () => number;
    readonly load_amb_vol_maze3d: () => number;
    readonly load_se_vol_maze3d: () => number;
    readonly maze_data_maze3d: () => [number, number];
    readonly move_blaster3d: (a: number, b: number) => void;
    readonly move_maze3d: (a: number) => void;
    readonly next_level_maze3d: () => void;
    readonly play_count_maze3d: () => number;
    readonly player_facing_maze3d: () => number;
    readonly player_hp_blaster3d: () => number;
    readonly player_max_hp_blaster3d: () => number;
    readonly player_x_maze3d: () => number;
    readonly player_z_maze3d: () => number;
    readonly reset_maze3d: () => void;
    readonly save_audio_vol_maze3d: (a: number, b: number) => void;
    readonly scene_blaster3d: () => number;
    readonly scene_earthdef: () => number;
    readonly scene_maze3d: () => number;
    readonly score_blaster3d: () => number;
    readonly score_earthdef: () => number;
    readonly set_aim_input_earthdef: (a: number, b: number) => void;
    readonly set_cam_input_earthdef: (a: number, b: number) => void;
    readonly set_laser_type_earthdef: (a: number) => void;
    readonly shoot_blaster3d: (a: number) => void;
    readonly sound_def_blaster3d: (a: number) => [number, number];
    readonly sound_def_maze3d: (a: number) => [number, number];
    readonly start_blaster3d: () => void;
    readonly start_earthdef: () => void;
    readonly start_game_maze3d: () => void;
    readonly steps_maze3d: () => number;
    readonly switch_camera_blaster3d: () => void;
    readonly theme_name_maze3d: () => [number, number];
    readonly tick_blaster3d: (a: number) => void;
    readonly tick_earthdef: (a: number) => void;
    readonly tick_maze3d: (a: number) => void;
    readonly toggle_auto_fire_blaster3d: () => void;
    readonly total_steps_maze3d: () => number;
    readonly turret_rotate_blaster3d: (a: number) => void;
    readonly warp_done_maze3d: () => number;
    readonly warp_maze3d: () => number;
    readonly wave_blaster3d: () => number;
    readonly wave_earthdef: () => number;
    readonly engine_font_regular: () => [number, number];
    readonly wgpu_render_pass_draw: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wgpu_render_pass_draw_indexed: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly wgpu_render_pass_set_pipeline: (a: number, b: bigint) => void;
    readonly wgpu_render_pass_set_viewport: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly wgpu_compute_pass_set_pipeline: (a: number, b: bigint) => void;
    readonly wgpu_render_pass_draw_indirect: (a: number, b: bigint, c: bigint) => void;
    readonly wgpu_render_bundle_draw: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wgpu_render_pass_set_bind_group: (a: number, b: number, c: bigint, d: number, e: number) => void;
    readonly wgpu_compute_pass_set_bind_group: (a: number, b: number, c: bigint, d: number, e: number) => void;
    readonly wgpu_render_pass_execute_bundles: (a: number, b: number, c: number) => void;
    readonly wgpu_render_pass_pop_debug_group: (a: number) => void;
    readonly wgpu_render_pass_write_timestamp: (a: number, b: bigint, c: number) => void;
    readonly wgpu_compute_pass_pop_debug_group: (a: number) => void;
    readonly wgpu_compute_pass_write_timestamp: (a: number, b: bigint, c: number) => void;
    readonly wgpu_render_pass_push_debug_group: (a: number, b: number, c: number) => void;
    readonly wgpu_render_pass_set_scissor_rect: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wgpu_compute_pass_push_debug_group: (a: number, b: number, c: number) => void;
    readonly wgpu_render_pass_set_vertex_buffer: (a: number, b: number, c: bigint, d: bigint, e: bigint) => void;
    readonly wgpu_render_pass_set_blend_constant: (a: number, b: number) => void;
    readonly wgpu_render_pass_set_push_constants: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wgpu_compute_pass_set_push_constant: (a: number, b: number, c: number, d: number) => void;
    readonly wgpu_render_pass_end_occlusion_query: (a: number) => void;
    readonly wgpu_render_pass_insert_debug_marker: (a: number, b: number, c: number) => void;
    readonly wgpu_render_pass_multi_draw_indirect: (a: number, b: bigint, c: bigint, d: number) => void;
    readonly wgpu_compute_pass_dispatch_workgroups: (a: number, b: number, c: number, d: number) => void;
    readonly wgpu_compute_pass_insert_debug_marker: (a: number, b: number, c: number) => void;
    readonly wgpu_render_pass_begin_occlusion_query: (a: number, b: number) => void;
    readonly wgpu_render_pass_draw_indexed_indirect: (a: number, b: bigint, c: bigint) => void;
    readonly wgpu_render_pass_set_stencil_reference: (a: number, b: number) => void;
    readonly wgpu_render_bundle_draw_indexed: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly wgpu_render_bundle_set_pipeline: (a: number, b: bigint) => void;
    readonly wgpu_render_bundle_draw_indirect: (a: number, b: bigint, c: bigint) => void;
    readonly wgpu_render_bundle_set_bind_group: (a: number, b: number, c: bigint, d: number, e: number) => void;
    readonly wgpu_render_pass_multi_draw_indirect_count: (a: number, b: bigint, c: bigint, d: bigint, e: bigint, f: number) => void;
    readonly wgpu_render_bundle_set_vertex_buffer: (a: number, b: number, c: bigint, d: bigint, e: bigint) => void;
    readonly wgpu_render_pass_multi_draw_indexed_indirect: (a: number, b: bigint, c: bigint, d: number) => void;
    readonly wgpu_render_bundle_set_push_constants: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wgpu_compute_pass_dispatch_workgroups_indirect: (a: number, b: bigint, c: bigint) => void;
    readonly wgpu_render_pass_end_pipeline_statistics_query: (a: number) => void;
    readonly wgpu_compute_pass_end_pipeline_statistics_query: (a: number) => void;
    readonly wgpu_render_bundle_draw_indexed_indirect: (a: number, b: bigint, c: bigint) => void;
    readonly wgpu_render_pass_begin_pipeline_statistics_query: (a: number, b: bigint, c: number) => void;
    readonly wgpu_compute_pass_begin_pipeline_statistics_query: (a: number, b: bigint, c: number) => void;
    readonly wgpu_render_pass_multi_draw_indexed_indirect_count: (a: number, b: bigint, c: bigint, d: bigint, e: bigint, f: number) => void;
    readonly wgpu_render_bundle_insert_debug_marker: (a: number, b: number) => void;
    readonly wgpu_render_bundle_pop_debug_group: (a: number) => void;
    readonly wgpu_render_bundle_set_index_buffer: (a: number, b: bigint, c: number, d: bigint, e: bigint) => void;
    readonly wgpu_render_bundle_push_debug_group: (a: number, b: number) => void;
    readonly wgpu_render_pass_set_index_buffer: (a: number, b: bigint, c: number, d: bigint, e: bigint) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h337539ba828a8639: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen__convert__closures_____invoke__h4c4817024f9b650e: (a: number, b: number, c: any, d: any) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_destroy_closure: (a: number, b: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
