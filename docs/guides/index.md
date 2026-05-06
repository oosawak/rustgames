---
layout: default
title: 統合ガイド
---

# 🔗 統合ガイド

各プラットフォームへの詳細な統合方法を説明します。

## 📋 目次

- [ネイティブアプリ統合](#ネイティブアプリ統合)
- [WASM ブラウザ統合](#wasm-ブラウザ統合)
- [Unity プラグイン統合](#unity-プラグイン統合)
- [カスタムエンジン拡張](#カスタムエンジン拡張)

---

## ネイティブアプリ統合

### 基本的なセットアップ

```rust
// native_app/src/main.rs
use engine::{Engine, RendererConfig};
use game_logic::GameState;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // ウィンドウ作成
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("3D Puzzle Game")
        .with_inner_size(LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)?;

    // エンジン初期化
    let config = RendererConfig {
        width: 1280,
        height: 720,
        title: "Game".to_string(),
    };
    let mut engine = Engine::new(config).await?;
    let mut game = GameState::new();

    // ゲームループ
    event_loop.run(move |event, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => control_flow.exit(),
                // ... 入力処理
                _ => {}
            },
            Event::AboutToWait => {
                // ゲーム更新
                game.update(0.016);
                window.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}
```

### オリジナルゲームエンジンの作成

エンジンを拡張して独自の機能を追加:

```rust
// my_game/src/lib.rs
use engine::{Engine, RendererConfig, Scene, GameObject};
use game_logic::GameState;

pub struct MyGameEngine {
    engine: Engine,
    game_state: GameState,
    custom_data: CustomData,
}

pub struct CustomData {
    level: u32,
    difficulty: Difficulty,
}

pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl MyGameEngine {
    pub async fn new() -> Result<Self, String> {
        let config = RendererConfig::default();
        let engine = Engine::new(config).await?;
        
        Ok(MyGameEngine {
            engine,
            game_state: GameState::new(),
            custom_data: CustomData {
                level: 1,
                difficulty: Difficulty::Normal,
            },
        })
    }

    pub async fn update(&mut self, delta: f32) {
        self.engine.update(delta).await;
        self.game_state.update(delta);
        
        // カスタムロジック
        self.handle_difficulty();
    }

    fn handle_difficulty(&mut self) {
        match self.custom_data.difficulty {
            Difficulty::Easy => {
                // イージー向けのロジック
            }
            Difficulty::Normal => {
                // ノーマル向けのロジック
            }
            Difficulty::Hard => {
                // ハード向けのロジック
            }
        }
    }
}
```

---

## WASM ブラウザ統合

### HTML セットアップ

```html
<!-- index.html -->
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width">
    <title>Rust Game - WASM</title>
    <style>
        body {
            margin: 0;
            padding: 20px;
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
        }
        
        .container {
            background: white;
            border-radius: 10px;
            padding: 20px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.3);
            text-align: center;
        }
        
        canvas {
            border: 2px solid #667eea;
            border-radius: 5px;
            max-width: 100%;
            background: #f0f0f0;
        }
        
        .stats {
            display: flex;
            justify-content: space-around;
            margin-top: 20px;
            gap: 10px;
        }
        
        .stat {
            flex: 1;
            padding: 10px;
            background: #f5f5f5;
            border-radius: 5px;
        }
        
        .stat label {
            font-weight: bold;
            color: #667eea;
        }
        
        .stat value {
            display: block;
            font-size: 24px;
            color: #333;
            margin-top: 5px;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎮 3D Puzzle Game (WASM)</h1>
        <canvas id="gameCanvas" width="1280" height="720"></canvas>
        
        <div class="stats">
            <div class="stat">
                <label>スコア</label>
                <value id="score">0</value>
            </div>
            <div class="stat">
                <label>移動数</label>
                <value id="moves">0</value>
            </div>
            <div class="stat">
                <label>時間</label>
                <value id="time">0.0s</value>
            </div>
        </div>
    </div>

    <script type="module">
        import init, { GameInstance } from './pkg/wasm_app.js';

        async function main() {
            await init();
            
            const game = new GameInstance('gameCanvas');
            game.init();
            
            let lastTime = performance.now();
            
            function frame(currentTime) {
                const dt = Math.min((currentTime - lastTime) / 1000, 0.1);
                lastTime = currentTime;
                
                game.update(dt);
                
                // UI 更新
                document.getElementById('score').textContent = game.get_score();
                document.getElementById('moves').textContent = game.get_moves();
                document.getElementById('time').textContent = game.get_time().toFixed(1) + 's';
                
                requestAnimationFrame(frame);
            }
            
            requestAnimationFrame(frame);
        }
        
        main().catch(err => {
            console.error('Failed to initialize game:', err);
            document.body.innerHTML = `<h1>⚠️ Game initialization failed</h1><p>${err}</p>`;
        });
    </script>
</body>
</html>
```

### Rust WASM コード

```rust
// wasm_app/src/lib.rs
use wasm_bindgen::prelude::*;
use engine::{Engine, RendererConfig};
use game_logic::GameState;
use std::cell::RefCell;
use std::rc::Rc;

#[wasm_bindgen]
pub struct GameInstance {
    game: Rc<RefCell<GameState>>,
}

#[wasm_bindgen]
impl GameInstance {
    #[wasm_bindgen(constructor)]
    pub fn new(_canvas_id: &str) -> GameInstance {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
        
        web_sys::console::log_1(&"Game initialized".into());
        
        GameInstance {
            game: Rc::new(RefCell::new(GameState::new())),
        }
    }
    
    pub fn init(&self) -> Result<(), JsValue> {
        Ok(())
    }
    
    pub fn update(&self, delta: f32) -> Result<(), JsValue> {
        let mut game = self.game.borrow_mut();
        game.update(delta);
        Ok(())
    }
    
    pub fn get_score(&self) -> u32 {
        self.game.borrow().score
    }
    
    pub fn get_moves(&self) -> u32 {
        self.game.borrow().moves
    }
    
    pub fn get_time(&self) -> f32 {
        self.game.borrow().time_elapsed
    }
}
```

---

## Unity プラグイン統合

### 完全な Unity プロジェクト設定

```
UnityGame/Assets/
├── Plugins/
│   ├── ffi_bridge.dll
│   ├── libffi_bridge.dylib
│   └── libffi_bridge.so
├── Scripts/
│   ├── RustGameBridge.cs
│   ├── GameManager.cs
│   ├── CubeController.cs
│   └── UIManager.cs
├── Scenes/
│   └── GameScene.unity
└── Resources/
    └── Cubes/
```

### GameManager 実装

```csharp
// Assets/Scripts/GameManager.cs
using UnityEngine;
using UnityEngine.UI;

public class GameManager : MonoBehaviour
{
    [SerializeField] private CubeController[] cubes;
    [SerializeField] private UIManager uiManager;
    [SerializeField] private float updateInterval = 0.016f;

    private float timeSinceLastUpdate = 0f;
    private bool isGameRunning = false;

    void Start()
    {
        int result = RustGameBridge.game_initialize(1280, 720);
        if (result != 0)
        {
            Debug.LogError("Failed to initialize game");
            return;
        }
        
        isGameRunning = true;
        Debug.Log("Game initialized successfully");
    }

    void Update()
    {
        if (!isGameRunning) return;

        timeSinceLastUpdate += Time.deltaTime;
        
        if (timeSinceLastUpdate >= updateInterval)
        {
            UpdateGame(timeSinceLastUpdate);
            timeSinceLastUpdate = 0f;
        }
    }

    void UpdateGame(float deltaTime)
    {
        RustGameBridge.game_update(deltaTime);
        
        var state = RustGameBridge.game_get_state();
        
        // UI 更新
        uiManager.UpdateScore(state.score);
        uiManager.UpdateMoves(state.moves);
        uiManager.UpdateTime(state.timeElapsed);
        
        if (state.isWon)
        {
            OnGameWon();
        }
    }

    public void MoveCube(uint cubeId, Vector3Int position)
    {
        int result = RustGameBridge.game_move_cube(
            cubeId,
            position.x,
            position.y,
            position.z
        );
        
        if (result == 0)
        {
            // パーティクル放出
            RustGameBridge.game_emit_particles(
                position.x,
                position.y,
                position.z,
                15
            );
            
            // スコア加算
            RustGameBridge.game_add_score(10);
        }
    }

    void OnGameWon()
    {
        isGameRunning = false;
        uiManager.ShowWinMessage();
        Debug.Log("🎉 Game Won!");
    }

    void OnDestroy()
    {
        if (isGameRunning)
        {
            RustGameBridge.game_cleanup();
        }
    }
}
```

### CubeController スクリプト

```csharp
// Assets/Scripts/CubeController.cs
using UnityEngine;

public class CubeController : MonoBehaviour
{
    [SerializeField] private uint cubeId;
    [SerializeField] private GameManager gameManager;
    
    private Vector3Int currentGridPosition;

    void OnMouseDown()
    {
        // クリック検出時に立方体を移動
        Vector3Int newPosition = GetNewPosition();
        gameManager.MoveCube(cubeId, newPosition);
    }

    Vector3Int GetNewPosition()
    {
        // UI や入力から新しいグリッド位置を決定
        return new Vector3Int(
            Random.Range(-2, 3),
            Random.Range(-2, 3),
            Random.Range(-2, 3)
        );
    }
}
```

---

## カスタムエンジン拡張

### オリジナルゲーム種の実装

```rust
// custom_game/src/lib.rs
use engine::{Engine, Scene, GameObject};
use game_logic::{GameState, ParticleSystem};

pub trait GameLogic {
    fn update(&mut self, delta: f32);
    fn on_input(&mut self, input: InputEvent);
    fn get_state(&self) -> GameStateData;
}

pub struct InputEvent {
    pub event_type: InputEventType,
    pub position: (f32, f32),
}

pub enum InputEventType {
    MouseDown,
    MouseUp,
    KeyPressed(char),
}

pub struct GameStateData {
    pub score: u32,
    pub level: u32,
    pub is_complete: bool,
}

// ゲーム実装例
pub struct PuzzleGame {
    engine: Engine,
    game_state: GameState,
    particles: ParticleSystem,
}

impl GameLogic for PuzzleGame {
    fn update(&mut self, delta: f32) {
        self.game_state.update(delta);
        self.particles.update(delta);
    }
    
    fn on_input(&mut self, event: InputEvent) {
        match event.event_type {
            InputEventType::MouseDown => {
                self.particles.emit_burst(
                    cgmath::Vector3::new(event.position.0, event.position.1, 0.0),
                    20,
                    1.5,
                    0.5,
                    (1.0, 0.5, 0.0, 1.0),
                );
            }
            _ => {}
        }
    }
    
    fn get_state(&self) -> GameStateData {
        GameStateData {
            score: self.game_state.score,
            level: 1,
            is_complete: self.game_state.puzzle.is_won(),
        }
    }
}
```

---

## トラブルシューティング

### Unity P/Invoke エラー

```csharp
// DLL が見つからない場合のフォールバック
#if UNITY_EDITOR
    private const string DLL_NAME = "ffi_bridge";
#else
    private const string DLL_NAME = "ffi_bridge";
#endif

// またはプラットフォーム条件付き
#if UNITY_STANDALONE_WIN
    private const string DLL_NAME = "ffi_bridge";
#elif UNITY_STANDALONE_OSX
    private const string DLL_NAME = "libffi_bridge";
#endif
```

### メモリリーク対策

```csharp
// 確実にクリーンアップ
void OnApplicationQuit()
{
    RustGameBridge.game_cleanup();
}
```

---

## 次のステップ

- [API リファレンス](../api/) で詳細な API を確認
- [実行例](../examples/) でデモを試す
- GitHub Issues で質問・報告

---

**最終更新**: 2026年5月6日
