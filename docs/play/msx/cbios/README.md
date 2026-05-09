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
