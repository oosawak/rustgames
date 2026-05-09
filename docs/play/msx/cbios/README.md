# C-BIOS ROM ファイルの入手方法

MSX エミュレータの動作には C-BIOS ROM が必要です。

## C-BIOS とは

C-BIOS はオープンソースの MSX BIOS 実装です。  
ライセンス: Modified BSD License (商用利用可)

公式サイト: https://cbios.sourceforge.net/

## ダウンロード手順

1. https://sourceforge.net/projects/cbios/files/ から最新リリースをダウンロード
2. アーカイブを展開:
   ```bash
   tar xjf cbios-0.29a.tar.bz2
   ```
3. 以下のファイルをこのディレクトリ (`docs/play/msx/cbios/`) にコピー:
   - `cbios_main_msx1.rom` (32KB — MSX1 メイン ROM)
   - `cbios_sub.rom`       (16KB — サブ ROM)

または付属スクリプトを実行:
```bash
cd docs/play/msx
bash fetch_cbios.sh
```

## ファイル構成

```
docs/play/msx/cbios/
├── README.md              ← このファイル
├── cbios_main_msx1.rom    ← 要配置 (32768 bytes)
└── cbios_sub.rom          ← 要配置 (16384 bytes)
```

## BIOS なしで使う場合

ROM ファイルをドロップするとカートリッジのみで起動を試みます。  
ただし BIOS ルーチンを呼び出すゲームは動作しません。

## MSX ROM カートリッジの入手

- 著作権フリーのホームブリューゲームを MSX Resource Center (www.msx.org) で探せます
- C-BIOS テストプログラム (`cbios_logo_msx1.rom`) が同梱されています
