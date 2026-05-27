/* tslint:disable */
/* eslint-disable */

export function act_penpen(key: number): void;

export function act_penpen2(key: number): void;

export function audio_event_penpen(): number;

export function audio_event_penpen2(): number;

export function camera_name_penpen(): string;

export function camera_name_penpen2(): string;

export function fish_count_penpen(): number;

export function fish_count_penpen2(): number;

export function hp_penpen(): number;

export function hp_penpen2(): number;

export function init_penpen(canvas_id: string): Promise<void>;

export function init_penpen2(canvas_id: string): Promise<void>;

export function init_penpen_demo(canvas_id: string): Promise<void>;

export function jump_penpen(on: boolean): void;

export function jump_penpen2(on: boolean): void;

export function level_penpen(): number;

export function level_penpen2(): number;

export function max_hp_penpen(): number;

export function max_hp_penpen2(): number;

export function move_penpen(dx: number): void;

export function move_penpen2(dx: number): void;

export function next_level_penpen(): void;

export function next_level_penpen2(): void;

export function progress_penpen(): number;

export function progress_penpen2(): number;

export function pull_dist_penpen2(): number;

export function pull_penpen2(on: boolean): void;

export function reset_game_penpen(): void;

export function reset_game_penpen2(): void;

export function resize_penpen(w: number, h: number): void;

export function resize_penpen2(w: number, h: number): void;

export function scene_penpen(): number;

export function scene_penpen2(): number;

export function score_penpen(): number;

export function score_penpen2(): number;

export function set_accel_input_penpen2(on: boolean): void;

export function set_brake_input_penpen2(on: boolean): void;

export function sound_def_penpen(event: number): string;

export function sound_def_penpen2(event: number): string;

export function speed_penpen(): number;

export function speed_penpen2(): number;

export function start_penpen(): void;

export function start_penpen2(): void;

export function switch_camera_penpen(): void;

export function switch_camera_penpen2(): void;

export function tick_penpen(ts: number): void;

export function tick_penpen2(ts: number): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly act_penpen: (a: number) => void;
    readonly act_penpen2: (a: number) => void;
    readonly audio_event_penpen: () => number;
    readonly audio_event_penpen2: () => number;
    readonly camera_name_penpen: (a: number) => void;
    readonly camera_name_penpen2: (a: number) => void;
    readonly fish_count_penpen: () => number;
    readonly fish_count_penpen2: () => number;
    readonly hp_penpen: () => number;
    readonly hp_penpen2: () => number;
    readonly init_penpen: (a: number, b: number) => number;
    readonly init_penpen2: (a: number, b: number) => number;
    readonly init_penpen_demo: (a: number, b: number) => number;
    readonly jump_penpen: (a: number) => void;
    readonly jump_penpen2: (a: number) => void;
    readonly level_penpen: () => number;
    readonly level_penpen2: () => number;
    readonly max_hp_penpen: () => number;
    readonly max_hp_penpen2: () => number;
    readonly move_penpen: (a: number) => void;
    readonly move_penpen2: (a: number) => void;
    readonly next_level_penpen: () => void;
    readonly next_level_penpen2: () => void;
    readonly progress_penpen: () => number;
    readonly progress_penpen2: () => number;
    readonly pull_dist_penpen2: () => number;
    readonly pull_penpen2: (a: number) => void;
    readonly reset_game_penpen: () => void;
    readonly reset_game_penpen2: () => void;
    readonly resize_penpen: (a: number, b: number) => void;
    readonly resize_penpen2: (a: number, b: number) => void;
    readonly scene_penpen: () => number;
    readonly scene_penpen2: () => number;
    readonly score_penpen: () => number;
    readonly score_penpen2: () => number;
    readonly set_accel_input_penpen2: (a: number) => void;
    readonly set_brake_input_penpen2: (a: number) => void;
    readonly sound_def_penpen: (a: number, b: number) => void;
    readonly speed_penpen: () => number;
    readonly speed_penpen2: () => number;
    readonly start_penpen: () => void;
    readonly start_penpen2: () => void;
    readonly switch_camera_penpen: () => void;
    readonly switch_camera_penpen2: () => void;
    readonly tick_penpen: (a: number) => void;
    readonly tick_penpen2: (a: number) => void;
    readonly wgpu_compute_pass_set_pipeline: (a: number, b: bigint) => void;
    readonly wgpu_compute_pass_set_bind_group: (a: number, b: number, c: bigint, d: number, e: number) => void;
    readonly wgpu_compute_pass_set_push_constant: (a: number, b: number, c: number, d: number) => void;
    readonly wgpu_compute_pass_insert_debug_marker: (a: number, b: number, c: number) => void;
    readonly wgpu_compute_pass_push_debug_group: (a: number, b: number, c: number) => void;
    readonly wgpu_compute_pass_pop_debug_group: (a: number) => void;
    readonly wgpu_compute_pass_write_timestamp: (a: number, b: bigint, c: number) => void;
    readonly wgpu_compute_pass_begin_pipeline_statistics_query: (a: number, b: bigint, c: number) => void;
    readonly wgpu_compute_pass_end_pipeline_statistics_query: (a: number) => void;
    readonly wgpu_compute_pass_dispatch_workgroups: (a: number, b: number, c: number, d: number) => void;
    readonly wgpu_compute_pass_dispatch_workgroups_indirect: (a: number, b: bigint, c: bigint) => void;
    readonly wgpu_render_bundle_set_pipeline: (a: number, b: bigint) => void;
    readonly wgpu_render_bundle_set_bind_group: (a: number, b: number, c: bigint, d: number, e: number) => void;
    readonly wgpu_render_bundle_set_vertex_buffer: (a: number, b: number, c: bigint, d: bigint, e: bigint) => void;
    readonly wgpu_render_bundle_set_push_constants: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wgpu_render_bundle_draw: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wgpu_render_bundle_draw_indexed: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly wgpu_render_bundle_draw_indirect: (a: number, b: bigint, c: bigint) => void;
    readonly wgpu_render_bundle_draw_indexed_indirect: (a: number, b: bigint, c: bigint) => void;
    readonly wgpu_render_pass_set_pipeline: (a: number, b: bigint) => void;
    readonly wgpu_render_pass_set_bind_group: (a: number, b: number, c: bigint, d: number, e: number) => void;
    readonly wgpu_render_pass_set_vertex_buffer: (a: number, b: number, c: bigint, d: bigint, e: bigint) => void;
    readonly wgpu_render_pass_set_push_constants: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wgpu_render_pass_draw: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wgpu_render_pass_draw_indexed: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly wgpu_render_pass_draw_indirect: (a: number, b: bigint, c: bigint) => void;
    readonly wgpu_render_pass_draw_indexed_indirect: (a: number, b: bigint, c: bigint) => void;
    readonly wgpu_render_pass_multi_draw_indirect: (a: number, b: bigint, c: bigint, d: number) => void;
    readonly wgpu_render_pass_multi_draw_indexed_indirect: (a: number, b: bigint, c: bigint, d: number) => void;
    readonly wgpu_render_pass_multi_draw_indirect_count: (a: number, b: bigint, c: bigint, d: bigint, e: bigint, f: number) => void;
    readonly wgpu_render_pass_multi_draw_indexed_indirect_count: (a: number, b: bigint, c: bigint, d: bigint, e: bigint, f: number) => void;
    readonly wgpu_render_pass_set_blend_constant: (a: number, b: number) => void;
    readonly wgpu_render_pass_set_scissor_rect: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wgpu_render_pass_set_viewport: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly wgpu_render_pass_set_stencil_reference: (a: number, b: number) => void;
    readonly wgpu_render_pass_insert_debug_marker: (a: number, b: number, c: number) => void;
    readonly wgpu_render_pass_push_debug_group: (a: number, b: number, c: number) => void;
    readonly wgpu_render_pass_pop_debug_group: (a: number) => void;
    readonly wgpu_render_pass_write_timestamp: (a: number, b: bigint, c: number) => void;
    readonly wgpu_render_pass_begin_occlusion_query: (a: number, b: number) => void;
    readonly wgpu_render_pass_end_occlusion_query: (a: number) => void;
    readonly wgpu_render_pass_begin_pipeline_statistics_query: (a: number, b: bigint, c: number) => void;
    readonly wgpu_render_pass_end_pipeline_statistics_query: (a: number) => void;
    readonly wgpu_render_pass_execute_bundles: (a: number, b: number, c: number) => void;
    readonly wgpu_render_bundle_insert_debug_marker: (a: number, b: number) => void;
    readonly wgpu_render_bundle_pop_debug_group: (a: number) => void;
    readonly wgpu_render_bundle_set_index_buffer: (a: number, b: bigint, c: number, d: bigint, e: bigint) => void;
    readonly wgpu_render_pass_set_index_buffer: (a: number, b: bigint, c: number, d: bigint, e: bigint) => void;
    readonly sound_def_penpen2: (a: number, b: number) => void;
    readonly wgpu_render_bundle_push_debug_group: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_833: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_891: (a: number, b: number, c: number, d: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export5: (a: number, b: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
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
