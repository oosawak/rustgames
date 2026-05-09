#!/bin/bash
# =============================================================================
# setup_cbios.sh — C-BIOS 取得スクリプト（要 sudo）
# 実行方法: sudo bash setup_cbios.sh
# =============================================================================

set -e

DEST_DIR="$(dirname "$0")/docs/play/msx/cbios"
mkdir -p "$DEST_DIR"

echo "=== C-BIOS セットアップ ==="

# 1. openMSX をインストール（C-BIOS が同梱されている）
echo "[1/3] openMSX をインストール中..."
apt install -y openmsx

# 2. C-BIOS ROM をコピー
echo "[2/3] C-BIOS ROM をコピー中..."
CBIOS_SRC="/usr/share/openmsx/cbios"

if [ -d "$CBIOS_SRC" ]; then
    cp "$CBIOS_SRC/cbios_main_msx1.rom" "$DEST_DIR/"
    cp "$CBIOS_SRC/cbios_sub.rom"       "$DEST_DIR/"
    echo "  コピー完了: $DEST_DIR/"
    ls -lh "$DEST_DIR/"
else
    # フォールバック: 別パスを試す
    FALLBACK=$(find /usr/share/openmsx -name "cbios_main_msx1.rom" 2>/dev/null | head -1)
    if [ -n "$FALLBACK" ]; then
        cp "$(dirname "$FALLBACK")/cbios_main_msx1.rom" "$DEST_DIR/"
        cp "$(dirname "$FALLBACK")/cbios_sub.rom"       "$DEST_DIR/" 2>/dev/null || true
        echo "  コピー完了 (フォールバック): $DEST_DIR/"
    else
        echo "  [ERROR] C-BIOS ROM が見つかりません"
        echo "  openMSX パッケージの中身を確認してください:"
        find /usr/share/openmsx -name "*.rom" 2>/dev/null | head -10
        exit 1
    fi
fi

# 3. 所有者を元に戻す（sudo 実行時に root 所有になるのを防ぐ）
echo "[3/3] ファイル所有者を修正中..."
SUDO_USER_HOME=$(getent passwd "${SUDO_USER:-$USER}" | cut -d: -f6)
chown -R "${SUDO_USER:-$USER}:${SUDO_USER:-$USER}" "$DEST_DIR"

echo ""
echo "=== 完了！==="
echo "C-BIOS ROM の場所: $DEST_DIR"
echo ""
echo "次のステップ:"
echo "  cd /home/oosawak/Workspace/rustgames"
echo "  wasm-pack build wasm_app --target web --out-dir docs/play/msx/wasm"
echo "  rm -f docs/play/msx/wasm/.gitignore"
echo "  python3 -m http.server 8080 --directory docs"
echo "  → ブラウザで http://localhost:8080/play/msx/ を開く"
