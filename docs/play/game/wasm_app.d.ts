/* tslint:disable */
/* eslint-disable */

export class GameInstance {
    free(): void;
    [Symbol.dispose](): void;
    get_cube_position(): string;
    get_goal_position(): string;
    get_moves(): number;
    get_score(): number;
    get_time(): number;
    is_won(): boolean;
    move_cube(x: number, y: number, z: number): boolean;
    constructor();
    reset(): void;
    update(delta_time: number): void;
}

export function start(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_gameinstance_free: (a: number, b: number) => void;
    readonly gameinstance_get_cube_position: (a: number) => [number, number];
    readonly gameinstance_get_goal_position: (a: number) => [number, number];
    readonly gameinstance_get_moves: (a: number) => number;
    readonly gameinstance_get_score: (a: number) => number;
    readonly gameinstance_get_time: (a: number) => number;
    readonly gameinstance_is_won: (a: number) => number;
    readonly gameinstance_move_cube: (a: number, b: number, c: number, d: number) => number;
    readonly gameinstance_new: () => number;
    readonly gameinstance_reset: (a: number) => void;
    readonly gameinstance_update: (a: number, b: number) => void;
    readonly start: () => void;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
