---
layout: default
title: 統合ガイド
---

# 統合ガイド

## Unity FFI Bridge

Rust ゲームエンジンのロジックを Unity から呼び出す C 互換インターフェース。

### ビルド

```bash
# Windows
cargo build --release -p ffi_bridge --target x86_64-pc-windows-msvc
# → target/release/ffi_bridge.dll

# macOS
cargo build --release -p ffi_bridge --target x86_64-apple-darwin
# → target/release/libffi_bridge.dylib

# Linux
cargo build --release -p ffi_bridge
# → target/release/libffi_bridge.so
```

### Unity プロジェクトへの配置

```
Unity Project/Assets/
├── Plugins/
│   ├── ffi_bridge.dll        (Windows)
│   ├── libffi_bridge.dylib   (macOS)
│   └── libffi_bridge.so      (Linux)
└── Scripts/
    ├── RustGameBridge.cs     (FFI インターフェース)
    └── GameManager.cs        (統合管理)
```

### 使い方

```csharp
using UnityEngine;

public class GameManager : MonoBehaviour {
    void Start() {
        RustGameBridge.game_initialize(1280, 720);
    }

    void Update() {
        RustGameBridge.game_update(Time.deltaTime);
        var state = RustGameBridge.game_get_state();
        Debug.Log($"Score: {state.score}, Moves: {state.moves}");
    }

    void OnDestroy() {
        RustGameBridge.game_cleanup();
    }
}
```

---

## 新しいゲームを追加する

1. `docs/play/` 以下にゲームディレクトリを作成
2. `index.html` で `engine.js` をインポート
3. `wasm-pack build` で `wasm/` を生成

```js
import init, * as wasm from './wasm/wasm_app.js';
import { FontLoader, AudioEngine, InputManager, CanvasManager } from '../../engine.js';

const audio  = new AudioEngine();
const input  = new InputManager();
const canvas = new CanvasManager('game-canvas');

async function start() {
  await init('./wasm/wasm_app_bg.wasm');
  canvas.resize();
  canvas.bindResize();
  await FontLoader.load('GenInterfaceJP', wasm);
  // ゲーム固有の初期化...
}
start();
```
