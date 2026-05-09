#!/bin/bash
# =============================================================================
# MSX ゲーム開発環境セットアップスクリプト
# 対象: Ubuntu/Debian / macOS (Homebrew)
#
# インストールされるもの:
#   - pasmo        : Z80 アセンブラ (MSX ROM 作成に最適)
#   - nasm         : 汎用アセンブラ (サブツールとして)
#   - z88dk        : Z80 向け C コンパイラ + ライブラリ (C で MSX ゲームを書く場合)
#   - hex2bin      : バイナリ変換ツール
#   - python3      : ローカル HTTP サーバー (エミュレータ確認用)
#   - wasm-pack    : Rust → WASM ビルドツール (rustgames ビルド用)
#   - xxd          : バイナリダンプ (ROM デバッグ用)
# =============================================================================

set -e

# カラー出力
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()    { echo -e "${CYAN}[INFO]${NC}  $1"; }
success() { echo -e "${GREEN}[OK]${NC}    $1"; }
warn()    { echo -e "${YELLOW}[WARN]${NC}  $1"; }
error()   { echo -e "${RED}[ERROR]${NC} $1"; }

echo ""
echo "============================================="
echo "  MSX ゲーム開発環境 セットアップ"
echo "============================================="
echo ""

# OS 判定
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v apt &>/dev/null; then
            echo "ubuntu"
        elif command -v dnf &>/dev/null; then
            echo "fedora"
        else
            echo "linux"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    else
        echo "unknown"
    fi
}

OS=$(detect_os)
info "検出された OS: $OS"
echo ""

# ── Ubuntu/Debian ──────────────────────────────────────────────────────────
install_ubuntu() {
    info "パッケージリストを更新中..."
    sudo apt update -qq

    # pasmo (Z80 アセンブラ)
    if command -v pasmo &>/dev/null; then
        success "pasmo はすでにインストール済み ($(pasmo --version 2>&1 | head -1))"
    else
        info "pasmo をインストール中..."
        sudo apt install -y pasmo
        success "pasmo インストール完了"
    fi

    # nasm
    if command -v nasm &>/dev/null; then
        success "nasm はすでにインストール済み ($(nasm --version | head -1))"
    else
        info "nasm をインストール中..."
        sudo apt install -y nasm
        success "nasm インストール完了"
    fi

    # z88dk (Z80 C コンパイラ)
    if command -v z88dk-z80asm &>/dev/null; then
        success "z88dk はすでにインストール済み"
    else
        info "z88dk をインストール中..."
        sudo apt install -y z88dk 2>/dev/null || {
            warn "apt での z88dk インストールに失敗。snap を試します..."
            if command -v snap &>/dev/null; then
                sudo snap install z88dk --classic 2>/dev/null && success "z88dk (snap) インストール完了" || warn "z88dk のインストールをスキップ (手動インストール: https://z88dk.org/)"
            else
                warn "z88dk のインストールをスキップ (手動インストール: https://z88dk.org/)"
            fi
        }
    fi

    # hex2bin
    if command -v hex2bin &>/dev/null; then
        success "hex2bin はすでにインストール済み"
    else
        info "hex2bin をインストール中..."
        sudo apt install -y hex2bin 2>/dev/null || warn "hex2bin はスキップ"
    fi

    # python3 (ローカルサーバー)
    if command -v python3 &>/dev/null; then
        success "python3 はすでにインストール済み ($(python3 --version))"
    else
        sudo apt install -y python3
        success "python3 インストール完了"
    fi

    # xxd (バイナリダンプ)
    if command -v xxd &>/dev/null; then
        success "xxd はすでにインストール済み"
    else
        sudo apt install -y xxd
        success "xxd インストール完了"
    fi
}

# ── macOS ──────────────────────────────────────────────────────────────────
install_macos() {
    if ! command -v brew &>/dev/null; then
        error "Homebrew が見つかりません。先にインストールしてください:"
        echo "  /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
        exit 1
    fi

    if command -v pasmo &>/dev/null; then
        success "pasmo はすでにインストール済み"
    else
        info "pasmo をインストール中..."
        brew install pasmo 2>/dev/null || {
            warn "Homebrew に pasmo がありません。ソースからビルドします..."
            install_pasmo_from_source
        }
    fi

    if command -v nasm &>/dev/null; then
        success "nasm はすでにインストール済み"
    else
        brew install nasm
        success "nasm インストール完了"
    fi

    if command -v z88dk-z80asm &>/dev/null; then
        success "z88dk はすでにインストール済み"
    else
        info "z88dk をインストール中..."
        brew install z88dk 2>/dev/null || warn "z88dk のインストールをスキップ (https://z88dk.org/)"
    fi
}

# ── pasmo ソースビルド (フォールバック) ────────────────────────────────────
install_pasmo_from_source() {
    info "pasmo をソースからビルドします..."
    TMP=$(mktemp -d)
    cd "$TMP"
    curl -sL "http://pasmo.speccy.org/bin/pasmo-0.5.4.tar.bz2" -o pasmo.tar.bz2 || {
        warn "pasmo ダウンロード失敗。手動インストール: http://pasmo.speccy.org/"
        return
    }
    tar xjf pasmo.tar.bz2
    cd pasmo-*
    ./configure && make -j$(nproc 2>/dev/null || sysctl -n hw.ncpu) && sudo make install
    success "pasmo ソースビルド完了"
    cd /tmp && rm -rf "$TMP"
}

# ── Rust / wasm-pack ───────────────────────────────────────────────────────
install_rust_tools() {
    echo ""
    info "Rust ツールチェーン確認..."

    # Rust
    if command -v cargo &>/dev/null; then
        success "Rust はすでにインストール済み ($(rustc --version))"
    else
        info "Rust をインストール中..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        success "Rust インストール完了"
    fi

    # wasm32 ターゲット
    if rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
        success "wasm32-unknown-unknown ターゲット追加済み"
    else
        info "wasm32-unknown-unknown ターゲットを追加中..."
        rustup target add wasm32-unknown-unknown
        success "wasm32 ターゲット追加完了"
    fi

    # wasm-pack
    if command -v wasm-pack &>/dev/null; then
        success "wasm-pack はすでにインストール済み ($(wasm-pack --version))"
    else
        info "wasm-pack をインストール中..."
        curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
        success "wasm-pack インストール完了"
    fi
}

# ── C-BIOS セットアップ ────────────────────────────────────────────────────
setup_cbios() {
    echo ""
    info "C-BIOS セットアップ..."

    CBIOS_DIR="$(dirname "$0")/docs/play/msx/cbios"
    mkdir -p "$CBIOS_DIR"

    if [[ -f "$CBIOS_DIR/cbios_main_msx1.rom" ]]; then
        success "C-BIOS はすでにセットアップ済み"
        return
    fi

    # OpenMSX の share から C-BIOS を探す
    OPENMSX_CBIOS=""
    for path in \
        "/usr/share/openmsx/cbios" \
        "$HOME/.openMSX/share/cbios" \
        "/opt/openMSX/share/cbios"; do
        if [[ -d "$path" ]]; then
            OPENMSX_CBIOS="$path"
            break
        fi
    done

    if [[ -n "$OPENMSX_CBIOS" ]]; then
        cp "$OPENMSX_CBIOS"/*.rom "$CBIOS_DIR/" 2>/dev/null && success "openMSX から C-BIOS をコピーしました"
    else
        warn "C-BIOS が見つかりません"
        echo ""
        echo "  以下のいずれかの方法で C-BIOS を入手してください:"
        echo ""
        echo "  方法1: openMSX をインストール (C-BIOS 同梱)"
        echo "    sudo apt install openmsx"
        echo "    cp /usr/share/openmsx/cbios/*.rom $CBIOS_DIR/"
        echo ""
        echo "  方法2: 手動ダウンロード"
        echo "    https://cbios.sourceforge.net/"
        echo "    → cbios_main_msx1.rom と cbios_sub.rom を"
        echo "      $CBIOS_DIR/ に置く"
    fi
}

# ── README 作成 ────────────────────────────────────────────────────────────
create_msx_readme() {
    cat > "$(dirname "$0")/docs/play/msx/cbios/README.md" 2>/dev/null <<'EOF'
# C-BIOS セットアップ

MSX エミュレータのデフォルト BIOS として **C-BIOS** を使用します。

## 必要なファイル

```
docs/play/msx/cbios/
├── cbios_main_msx1.rom   (MSX1 メイン ROM, 32KB)
└── cbios_sub.rom         (サブ ROM, 16KB)
```

## 入手方法

### 方法1: openMSX 経由（推奨）
```bash
sudo apt install openmsx
cp /usr/share/openmsx/cbios/cbios_main_msx1.rom docs/play/msx/cbios/
cp /usr/share/openmsx/cbios/cbios_sub.rom docs/play/msx/cbios/
```

### 方法2: 公式サイト
https://cbios.sourceforge.net/ からダウンロード

## ライセンス
C-BIOS は Creative Commons Attribution 2.5 ライセンスです。
EOF
}

# ── バージョン確認 ─────────────────────────────────────────────────────────
print_summary() {
    echo ""
    echo "============================================="
    echo "  インストール済みツール"
    echo "============================================="
    command -v pasmo      &>/dev/null && echo "  ✅ pasmo      : $(pasmo --version 2>&1 | head -1)" || echo "  ❌ pasmo      : 未インストール"
    command -v nasm       &>/dev/null && echo "  ✅ nasm       : $(nasm --version | head -1)" || echo "  ❌ nasm       : 未インストール"
    command -v z88dk-z80asm &>/dev/null && echo "  ✅ z88dk      : インストール済み" || echo "  ⚠️  z88dk      : 未インストール (任意)"
    command -v python3    &>/dev/null && echo "  ✅ python3    : $(python3 --version)" || echo "  ❌ python3    : 未インストール"
    command -v rustc      &>/dev/null && echo "  ✅ rustc      : $(rustc --version)" || echo "  ❌ Rust       : 未インストール"
    command -v wasm-pack  &>/dev/null && echo "  ✅ wasm-pack  : $(wasm-pack --version)" || echo "  ❌ wasm-pack  : 未インストール"
    echo ""
    echo "============================================="
    echo "  次のステップ"
    echo "============================================="
    echo ""
    echo "  1. C-BIOS をセットアップ (上記 README 参照)"
    echo "     docs/play/msx/cbios/README.md"
    echo ""
    echo "  2. WASM ビルド"
    echo "     wasm-pack build wasm_app --target web \\"
    echo "       --out-dir docs/play/msx/wasm"
    echo ""
    echo "  3. ローカルサーバーで確認"
    echo "     python3 -m http.server 8080 --directory docs"
    echo "     → http://localhost:8080/play/msx/"
    echo ""
    echo "  4. MSX ゲーム (Lineboy) をアセンブル"
    echo "     pasmo --msx lineboy.asm lineboy.rom"
    echo ""
}

# ── メイン実行 ─────────────────────────────────────────────────────────────
case "$OS" in
    ubuntu)  install_ubuntu ;;
    macos)   install_macos ;;
    *)
        warn "未対応の OS です: $OS"
        warn "手動で以下をインストールしてください: pasmo, nasm, python3"
        ;;
esac

install_rust_tools
setup_cbios
create_msx_readme
print_summary
