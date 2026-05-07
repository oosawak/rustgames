---
layout: default
title: 実行例
---

# 実行例

## Neon Maze 3D をローカルで動かす

```bash
# 1. クローン
git clone https://github.com/oosawak/rustgames.git
cd rustgames

# 2. WASM ビルド
wasm-pack build wasm_app --target web --out-dir docs/play/maze3d/wasm
rm -f docs/play/maze3d/wasm/.gitignore

# 3. ローカルサーバー起動
python3 -m http.server 8080 --directory docs

# 4. ブラウザで確認
# http://localhost:8080/play/maze3d/
```

## フォント埋め込みビルド

WASM バイナリにフォントを同梱する場合（日本語フォントが OS に入っていない環境向け）:

```bash
wasm-pack build wasm_app --target web --out-dir docs/play/maze3d/wasm -- --features embed-font
rm -f docs/play/maze3d/wasm/.gitignore
```

> ⚠️ WASM バイナリが約 6MB 増加します。

## GitHub Pages へデプロイ

```bash
git add docs/
git commit -m "Update WASM build"
git push
# → https://oosawak.github.io/rustgames/play/maze3d/ に自動反映
```

## engine.js の使い方サンプル

```html
<!DOCTYPE html>
<html>
<head>
  <link rel="stylesheet" href="../../engine.css">
</head>
<body>
  <canvas id="game-canvas"></canvas>
  <script type="module">
    import init, * as wasm from './wasm/wasm_app.js';
    import { FontLoader, AudioEngine, InputManager, CanvasManager } from '../../engine.js';

    const audio  = new AudioEngine();
    const input  = new InputManager({ swipeMin: 35 });
    const canvas = new CanvasManager('game-canvas', { hudHeight: 0, dpadHeight: 0 });

    async function start() {
      await init('./wasm/wasm_app_bg.wasm');
      canvas.resize();
      canvas.bindResize();
      await FontLoader.load('GenInterfaceJP', wasm);
      // ゲーム初期化...
      requestAnimationFrame(loop);
    }

    function loop(ts) {
      wasm.tick_game(ts);
      requestAnimationFrame(loop);
    }

    start();
  </script>
</body>
</html>
```
