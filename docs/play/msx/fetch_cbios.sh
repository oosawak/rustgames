#!/bin/bash
# C-BIOS ROM ファイルをダウンロードするスクリプト
# C-BIOS は https://cbios.sourceforge.net/ から入手可能

set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CBIOS_DIR="$SCRIPT_DIR/cbios"

mkdir -p "$CBIOS_DIR"
echo "C-BIOS is available at https://cbios.sourceforge.net/"
echo ""
echo "Download the latest release (e.g., cbios-0.29a.tar.bz2) and extract:"
echo ""
echo "  tar xjf cbios-0.29a.tar.bz2"
echo "  cp cbios-0.29a/cbios_main_msx1.rom $CBIOS_DIR/"
echo "  cp cbios-0.29a/cbios_sub.rom       $CBIOS_DIR/"
echo ""
echo "Or use wget/curl if a direct URL is known:"
echo "  https://sourceforge.net/projects/cbios/files/"
echo ""
echo "After placing the files, verify:"
echo "  ls -la $CBIOS_DIR/"
