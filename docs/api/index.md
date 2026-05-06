---
layout: default
title: API リファレンス
---

# 📚 API リファレンス

Rust Game Engine の主要 API ドキュメントです。

## 📑 目次

- [Engine API](#engine-api)
- [Scene API](#scene-api)
- [Input API](#input-api)
- [Graphics API](#graphics-api)
- [Game Logic API](#game-logic-api)

---

## Engine API

### Engine::new

ゲームエンジンを初期化します。

```rust
pub async fn new(config: RendererConfig) -> Result<Self, String>
```

**パラメータ:**
- `config`: レンダラー設定

**戻り値:**
- `Engine`: 初期化されたエンジンインスタンス

**例:**
```rust
let config = RendererConfig {
    width: 1280,
    height: 720,
    title: "My Game".to_string(),
};
let engine = Engine::new(config).await?;
```

### Engine::update

ゲーム状態を更新します（毎フレーム呼び出し）。

```rust
pub async fn update(&mut self, delta_time: f32)
```

**パラメータ:**
- `delta_time`: フレーム時間（秒）

---

## Scene API

### Scene::new

新規シーンを作成します。

```rust
pub fn new() -> Self
```

### Scene::create_object

ゲームオブジェクトを作成します。

```rust
pub fn create_object(&mut self, name: String) -> u32
```

**パラメータ:**
- `name`: オブジェクト名

**戻り値:**
- `u32`: オブジェクト ID

**例:**
```rust
let scene = Scene::new();
let cube_id = scene.create_object("Cube".to_string());
```

### Scene::set_camera

カメラを設定します。

```rust
pub fn set_camera(&mut self, position: Vector3, target: Vector3)
```

**パラメータ:**
- `position`: カメラ位置
- `target`: カメラ注視点

---

## Input API

### InputState

入力状態を管理します。

```rust
pub struct InputState {
    pub mouse: MouseState,
    pub keyboard: KeyboardState,
    pub scroll_delta: f32,
}
```

### InputState::set_key

キー入力を設定します。

```rust
pub fn set_key(&mut self, key: Key, pressed: bool)
```

**パラメータ:**
- `key`: キー種別 (W, A, S, D, Space, Escape, Enter)
- `pressed`: 押下状態

**例:**
```rust
input_state.set_key(Key::W, true);  // W キー押下
input_state.set_key(Key::W, false); // W キー解放
```

---

## Graphics API

### Mesh::create_cube

立方体メッシュを生成します。

```rust
pub fn create_cube() -> Mesh
```

**戻り値:**
- `Mesh`: 立方体メッシュ

**例:**
```rust
let cube = graphics::create_cube();
let mesh_data = cube.to_mesh_data(&device);
```

### ShaderModule

シェーダーモジュール。

```rust
pub struct ShaderModule {
    pub vertex_shader: wgpu::ShaderModule,
    pub fragment_shader: wgpu::ShaderModule,
}
```

### ShaderModule::new

シェーダーを作成します。

```rust
pub fn new(device: &wgpu::Device) -> Self
```

---

## Game Logic API

### GameState

ゲーム全体の状態を管理します。

```rust
pub struct GameState {
    pub score: u32,
    pub moves: u32,
    pub time_elapsed: f32,
    pub puzzle: PuzzleLogic,
    pub particles: ParticleSystem,
    pub physics: PhysicsWorld,
}
```

### PuzzleLogic::move_cube

立方体を移動します。

```rust
pub fn move_cube(&mut self, cube_id: u32, new_position: (i32, i32, i32)) -> bool
```

**パラメータ:**
- `cube_id`: 立方体 ID
- `new_position`: 移動先座標 (x, y, z)

**戻り値:**
- `bool`: 移動成功（true）/失敗（false）

**例:**
```rust
let success = puzzle.move_cube(1, (0, 0, 1));
if success {
    println!("Cube moved!");
}
```

### PuzzleLogic::is_won

パズルがクリアされたか確認します。

```rust
pub fn is_won(&self) -> bool
```

**戻り値:**
- `bool`: クリア状態

### ParticleSystem::emit_burst

パーティクル放出。

```rust
pub fn emit_burst(
    &mut self,
    position: Vector3<f32>,
    count: usize,
    velocity_magnitude: f32,
    lifetime: f32,
    color: (f32, f32, f32, f32),
)
```

**パラメータ:**
- `position`: 放出位置
- `count`: パーティクル数
- `velocity_magnitude`: 速度の大きさ
- `lifetime`: 生存時間（秒）
- `color`: RGBA カラー

**例:**
```rust
particles.emit_burst(
    Vector3::new(0.0, 1.0, 0.0),
    20,
    2.0,
    1.0,
    (1.0, 1.0, 0.0, 1.0),  // 黄色
);
```

---

## FFI Bridge API (Unity 統合)

### GameStateFFI

C 互換ゲーム状態構造体。

```rust
#[repr(C)]
pub struct GameStateFFI {
    pub score: u32,
    pub moves: u32,
    pub time_elapsed: f32,
    pub is_won: bool,
    pub puzzle_move_count: u32,
}
```

### game_initialize

ゲームを初期化（FFI）。

```c
int game_initialize(uint32_t width, uint32_t height);
```

**戻り値:** 0 = 成功, -1 = 失敗

### game_update

ゲームを更新（FFI）。

```c
int game_update(float delta_time);
```

### game_get_state

ゲーム状態取得（FFI）。

```c
GameStateFFI game_get_state(void);
```

### game_move_cube

立方体移動（FFI）。

```c
int game_move_cube(uint32_t cube_id, int32_t x, int32_t y, int32_t z);
```

### game_emit_particles

パーティクル放出（FFI）。

```c
int game_emit_particles(float x, float y, float z, uint32_t count);
```

---

## 型リファレンス

### Vector3

3D ベクトル (glam::Vec3)

```rust
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
```

### Matrix4

4x4 行列 (glam::Mat4)

```rust
pub struct Matrix4 { /* ... */ }
```

### Color

RGBA カラー

```rust
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
```

---

## エラーハンドリング

すべての初期化関数は `Result<T, String>` を返します。

```rust
match Engine::new(config).await {
    Ok(engine) => { /* 成功 */ },
    Err(e) => println!("Error: {}", e),
}
```

---

## 次のステップ

- [統合ガイド](../guides/) で実装例を確認
- [実行例](../examples/) でデモを試す

---

**最終更新**: 2026年5月6日
