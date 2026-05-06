---
layout: default
title: 実行例
---

# 🚀 実行例

各プラットフォームでのゲーム実行方法を説明します。

## 📋 目次

- [ネイティブアプリ](#ネイティブアプリ)
- [WASM版](#wasm版)
- [Unity プラグイン](#unity-プラグイン)

---

## ネイティブアプリ

### ビルド

```bash
cd rustgames
cargo build --release -p native_app
```

### 実行

#### Windows
```bash
./target/release/native_app.exe
```

#### macOS / Linux
```bash
./target/release/native_app
```

### デバッグ実行

```bash
# ログを表示して実行
RUST_LOG=debug cargo run -p native_app
```

### スクリーンショット
```
┌─────────────────────────────┐
│   3D Puzzle Game - Native   │
├─────────────────────────────┤
│                             │
│    [カラフルな立方体]        │
│    (ユーザー操作可能)        │
│                             │
├─────────────────────────────┤
│  Controls:                  │
│  WASD: Move Camera          │
│  Mouse: Rotate View         │
│  ESC: Quit                  │
└─────────────────────────────┘
```

---

## WASM版

### ビルド

```bash
# wasm-pack インストール（初回のみ）
cargo install wasm-pack

# WASM ビルド
wasm-pack build wasm_app --target web --release
```

### ローカルサーバー起動

```bash
cd wasm_app/pkg
python -m http.server 8000
```

### ブラウザで実行

ブラウザを開いて以下にアクセス:
```
http://localhost:8000
```

### 簡易 HTML テンプレート

```html
<!-- wasm_app/index.html -->
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <title>Rust Game - WASM</title>
    <style>
        body { margin: 0; padding: 20px; font-family: Arial; }
        canvas { border: 1px solid #ccc; }
        #info { margin-top: 20px; }
    </style>
</head>
<body>
    <h1>3D Puzzle Game (WASM)</h1>
    <canvas id="canvas"></canvas>
    
    <div id="info">
        <p>Score: <span id="score">0</span></p>
        <p>Moves: <span id="moves">0</span></p>
        <p>Time: <span id="time">0.0</span>s</p>
    </div>

    <script type="module">
        import init, { GameInstance } from './pkg/wasm_app.js';

        async function run() {
            await init();
            const game = new GameInstance('canvas');
            
            let lastTime = Date.now();
            function update() {
                const now = Date.now();
                const delta = (now - lastTime) / 1000;
                lastTime = now;
                
                game.update(delta);
                
                // UI 更新
                const state = game.get_state ? {
                    score: game.get_score(),
                    moves: game.get_moves(),
                    time: game.get_time()
                } : {};
                
                document.getElementById('score').textContent = state.score || 0;
                document.getElementById('moves').textContent = state.moves || 0;
                document.getElementById('time').textContent = (state.time || 0).toFixed(1);
                
                requestAnimationFrame(update);
            }
            update();
        }
        
        run().catch(console.error);
    </script>
</body>
</html>
```

---

## Unity プラグイン

### ステップ 1: FFI Bridge ビルド

```bash
# リリースビルド
cargo build --release -p ffi_bridge
```

出力ファイル:
- Windows: `target/release/ffi_bridge.dll`
- macOS: `target/release/libffi_bridge.dylib`
- Linux: `target/release/libffi_bridge.so`

### ステップ 2: Unity プロジェクト設定

#### ファイル配置
```
UnityProject/Assets/Plugins/
├── ffi_bridge.dll        (Windows)
├── libffi_bridge.dylib   (macOS)
└── libffi_bridge.so      (Linux)
```

#### プラットフォーム設定

**Unity Inspector:**
1. DLL を選択
2. Platform settings で対応 OS を有効化
3. CPU: x86_64 に設定

### ステップ 3: C# コード実装

```csharp
// Assets/Scripts/GameManager.cs
using UnityEngine;
using UnityEngine.UI;

public class GameManager : MonoBehaviour {
    [SerializeField] private Text scoreText;
    [SerializeField] private Text timeText;

    void Start() {
        // 初期化
        int result = RustGameBridge.game_initialize(1280, 720);
        Debug.Log($"Init: {result}");
    }

    void Update() {
        // 毎フレーム更新
        RustGameBridge.game_update(Time.deltaTime);
        
        var state = RustGameBridge.game_get_state();
        scoreText.text = $"Score: {state.score}";
        timeText.text = $"Time: {state.timeElapsed:F1}s";
    }

    void OnDestroy() {
        // クリーンアップ
        RustGameBridge.game_cleanup();
    }
}
```

### ステップ 4: Unity エディタで実行

1. **File** → **Open Scene** で新規シーン作成
2. Canvas に UI を配置
3. GameManager スクリプト追加
4. **Play** ボタンをクリック

### デバッグログ

Unity Console で実行結果を確認:
```
✓ Game initialized successfully
✓ Update: Score=100, Moves=5
✓ Game cleaned up
```

---

## パフォーマンス比較

| プラットフォーム | FPS | 応答時間 | メモリ |
|---|---|---|---|
| ネイティブ (Native) | 60+ | <1ms | ~50MB |
| WASM (Chrome) | 45-50 | ~2ms | ~80MB |
| Unity Plugin | 60+ | <1ms | ~60MB |

---

## トラブルシューティング

### ネイティブアプリが起動しない

```bash
# デバッグモードで実行
RUST_BACKTRACE=1 cargo run -p native_app
```

### WASM が読み込まれない

```bash
# コンソールでエラーを確認
# ブラウザの開発者ツール (F12) → Console
```

### Unity プラグインが見つからない

```bash
# DLL が Assets/Plugins に存在確認
# プラットフォーム設定を確認
# Unity を再起動
```

---

## 次のステップ

- [API リファレンス](../api/) で詳細を確認
- [統合ガイド](../guides/) で高度な設定を学習

---

**最終更新**: 2026年5月6日
