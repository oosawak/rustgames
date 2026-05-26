# 3D Penguin Sliding Game (PenPen Glide 3D)

Create a 3D penguin sliding game under the workspace using the `rustgames` engine template as the foundation. The penguin slides down a winding icy half-pipe, dodging obstacles (snowmen, trees, rocks) and collecting fish for speed boosts and points.

## User Review Required

> [!IMPORTANT]
> The game uses the `wgpu` WebGL backend in Rust and WebAssembly, compiled with `wasm-pack`. It will be integrated into the existing `docs/play/` directory alongside the other games in the repository.

> [!NOTE]
> We will restructure the workspace by moving all files from `rustgames_repo` directly to the root of `/home/oosawak/Workspace/PenPen`. This makes the workspace root the project repository itself, resolving any compilation issues.

## Proposed Changes

### Restructure Workspace

#### [MODIFY] Move repository contents to workspace root
We will execute shell commands to move all files from `/home/oosawak/Workspace/PenPen/rustgames_repo` to `/home/oosawak/Workspace/PenPen/` and delete the empty directory `rustgames_repo`.

---

### Rust WASM Engine (`wasm_app`)

#### [NEW] [penpen.rs](file:///home/oosawak/Workspace/PenPen/wasm_app/src/penpen.rs)
Create the core module for the penguin game. This will contain:
- `PenPenGame` struct: manages physics (gravity on half-pipe, speed boosts, momentum), player state (HP, score, level, Z-distance), obstacles (snowmen, pine trees, ice crystals), collectibles (fish floating at different heights), and particles (snow trail, sparkle bursts).
- `Obstacle` & `Collectible` structs.
- `penpen::geometry` module (or inline): generates the 3D meshes for:
  - Winding half-pipe track with alternating light-blue and dark-blue ice quads, and neon boundaries (Cyan on left, Magenta on right).
  - 3D blocky penguin (black body, white belly, orange feet, yellow beak, and a glowing neon pink scarf).
  - Obstacles (blocky snowmen, pine trees with green pyramid layers, cyan ice crystals).
  - Collectibles (pink fish models).
  - Sparkle and snow particles.
- Camera system: a Smooth TPS camera following the penguin, tilting slightly during carving turns.

#### [MODIFY] [lib.rs](file:///home/oosawak/Workspace/PenPen/wasm_app/src/lib.rs)
- Register `pub mod penpen;`
- Export WASM bindings:
  - `init_penpen(canvas_id: &str)`
  - `tick_penpen(ts: f64)`
  - `move_penpen(dx: f32)` (receives left/right inputs)
  - `jump_penpen(active: bool)` (receives jump input)
  - `start_penpen()`
  - `scene_penpen() -> u8`
  - `score_penpen() -> u32`
  - `speed_penpen() -> f32`
  - `hp_penpen() -> i32`
  - `max_hp_penpen() -> i32`
  - `level_penpen() -> u32`
  - `fish_count_penpen() -> u32`
  - `audio_event_penpen() -> u8`
  - `sound_def_penpen(event: u8) -> String`
  - `all_sound_defs_penpen() -> String`
  - `progress_penpen() -> f32` (Z progress percentage)

---

### HTML/JS Web Application

#### [NEW] [index.html](file:///home/oosawak/Workspace/PenPen/docs/play/penpen/index.html)
The web page for the game. Includes:
- A fullscreen `<canvas id="penpen-canvas">`.
- Premium retro-neon styled HUD using CSS glassmorphism, featuring:
  - Glowing speed gauge (simulated speed in km/h derived from Rust engine).
  - Score, level, and fish count.
  - Heart icons representing player HP.
  - Interactive start overlay, game-over screen, and level clear overlay.
- Event listeners for WASD/Arrow keys (movement and jumping).
- Integration of `engine.js` classes (`AudioEngine`, `CanvasManager`, `FontLoader`).
- Mobile touch support using swipe or virtual buttons.

#### [NEW] [style.css](file:///home/oosawak/Workspace/PenPen/docs/play/penpen/style.css)
- Custom CSS styling using retro-neon gradients, glowing shadows, smooth transitions, and responsive layouts.

#### [MODIFY] [index.html](file:///home/oosawak/Workspace/PenPen/docs/play/index.html)
- Add a new card linking to the "PenPen Glide 3D" game, complete with a description and tags.

---

### Operations Log

#### [NEW] [operation_log.md](file:///home/oosawak/Workspace/PenPen/log/admin/operation_log.md)
Create an operation log as requested by user rules to document all code modifications.

## Verification Plan

### Automated Tests
- Run `cargo check` to verify Rust code compiles correctly.
- Compile to WASM using `wasm-pack`:
  `wasm-pack build wasm_app --target web --out-dir docs/play/penpen/wasm`

### Manual Verification
- Run a local static server: `npx serve docs` or `python3 -m http.server 8080 --directory docs`
- Launch the browser and play the game to verify:
  - Left/right sliding physics and the half-pipe carving feel.
  - Obstacle collisions (reduction of HP, knockback, and particle explosions).
  - Collecting fish (speed boost, points, sparkle particles, audio trigger).
  - Sound effects playing through Web Audio API.
  - Speed gauge and HUD updating correctly.
