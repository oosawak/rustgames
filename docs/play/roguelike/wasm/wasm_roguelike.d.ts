/* tslint:disable */
/* eslint-disable */

export function clear_messages_roguelike(): void;

export function depth_roguelike(): number;

export function enemy_count_roguelike(): number;

export function enemy_data_roguelike(index: number): Int32Array;

export function hp_roguelike(): number;

export function init_roguelike(): void;

export function inventory_roguelike(): Uint32Array;

export function level_roguelike(): number;

export function map_data_roguelike(): Uint8Array;

export function map_height_roguelike(): number;

export function map_width_roguelike(): number;

export function max_hp_roguelike(): number;

export function max_mp_roguelike(): number;

export function messages_roguelike(): string[];

export function move_roguelike(action: number): void;

export function mp_roguelike(): number;

export function player_atk_roguelike(): number;

export function player_def_roguelike(): number;

export function player_direction_roguelike(): number;

export function player_equipped_accessory_roguelike(): number;

export function player_equipped_armor_roguelike(): number;

export function player_equipped_weapon_roguelike(): number;

export function player_x_roguelike(): number;

export function player_y_roguelike(): number;

export function render_roguelike(canvas_id: string, width: number, height: number): void;

export function scene_roguelike(): number;

export function start_game_roguelike(): void;

export function tick_roguelike(ts: number): void;

export function visited_data_roguelike(): Uint8Array;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly clear_messages_roguelike: () => void;
    readonly depth_roguelike: () => number;
    readonly enemy_count_roguelike: () => number;
    readonly enemy_data_roguelike: (a: number) => [number, number];
    readonly hp_roguelike: () => number;
    readonly init_roguelike: () => void;
    readonly inventory_roguelike: () => [number, number];
    readonly level_roguelike: () => number;
    readonly map_data_roguelike: () => [number, number];
    readonly map_height_roguelike: () => number;
    readonly map_width_roguelike: () => number;
    readonly max_hp_roguelike: () => number;
    readonly max_mp_roguelike: () => number;
    readonly messages_roguelike: () => [number, number];
    readonly move_roguelike: (a: number) => void;
    readonly mp_roguelike: () => number;
    readonly player_atk_roguelike: () => number;
    readonly player_def_roguelike: () => number;
    readonly player_direction_roguelike: () => number;
    readonly player_equipped_accessory_roguelike: () => number;
    readonly player_equipped_armor_roguelike: () => number;
    readonly player_equipped_weapon_roguelike: () => number;
    readonly player_x_roguelike: () => number;
    readonly player_y_roguelike: () => number;
    readonly render_roguelike: (a: number, b: number, c: number, d: number) => void;
    readonly scene_roguelike: () => number;
    readonly start_game_roguelike: () => void;
    readonly tick_roguelike: (a: number) => void;
    readonly visited_data_roguelike: () => [number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_drop_slice: (a: number, b: number) => void;
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
