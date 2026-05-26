#!/bin/bash

# SERVER マシンから SSH キーをコピーするスクリプト
# expect コマンドを使ってパスワード認証を自動化

set -e

echo "🔑 SSH キーコピー（server → ローカル）"
echo "====================================="
echo ""

# SERVER の情報
SERVER_HOST="server"
SERVER_USER="user"
SSH_KEY_LOCAL="$HOME/.ssh/id_ed25519"
SSH_KEY_PUB_LOCAL="$HOME/.ssh/id_ed25519.pub"

# expect コマンドが利用可能か確認
if ! command -v expect &> /dev/null; then
    echo "❌ エラー: 'expect' コマンドが見つかりません"
    echo "インストール: sudo apt-get install expect"
    exit 1
fi

# ローカルの ~/.ssh ディレクトリをチェック
if [ ! -d "$HOME/.ssh" ]; then
    echo "📁 ~/.ssh ディレクトリを作成中..."
    mkdir -p "$HOME/.ssh"
    chmod 700 "$HOME/.ssh"
fi

# 既存のキーがあるかチェック
if [ -f "$SSH_KEY_LOCAL" ]; then
    echo "⚠️  既にローカルに SSH キーがあります："
    echo "   $SSH_KEY_LOCAL"
    echo ""
    read -p "上書きしていいですか？ (yes/no): " CONFIRM
    if [ "$CONFIRM" != "yes" ]; then
        echo "キャンセルしました"
        exit 0
    fi
fi

echo "SERVER マシンへ接続するパスワードを入力してください："
echo "(入力は画面に表示されません)"
read -sp "Password: " SERVER_PASSWORD
echo ""

# expect スクリプトでパスワード認証を行い、秘密鍵をコピー
expect -c "
set timeout 10
set password \"$SERVER_PASSWORD\"

# 秘密鍵をコピー
spawn scp -o StrictHostKeyChecking=no $SERVER_USER@$SERVER_HOST:~/.ssh/id_ed25519 $SSH_KEY_LOCAL
expect {
    \"password:\" { send \"\$password\r\"; exp_continue }
    \"yes/no\" { send \"yes\r\"; exp_continue }
    eof { }
    timeout { exit 1 }
}
" || {
    echo "❌ 秘密鍵のコピーに失敗しました"
    unset SERVER_PASSWORD
    exit 1
}

echo "✅ 秘密鍵をコピーしました"

# expect スクリプトで公開鍵もコピー
expect -c "
set timeout 10
set password \"$SERVER_PASSWORD\"

spawn scp -o StrictHostKeyChecking=no $SERVER_USER@$SERVER_HOST:~/.ssh/id_ed25519.pub $SSH_KEY_PUB_LOCAL
expect {
    \"password:\" { send \"\$password\r\"; exp_continue }
    \"yes/no\" { send \"yes\r\"; exp_continue }
    eof { }
    timeout { exit 1 }
}
" || {
    echo "❌ 公開鍵のコピーに失敗しました"
    unset SERVER_PASSWORD
    exit 1
}

echo "✅ 公開鍵をコピーしました"

# パーミッション設定
chmod 600 "$SSH_KEY_LOCAL"
chmod 644 "$SSH_KEY_PUB_LOCAL"

echo ""
echo "✅ SSH キーのセットアップ完了！"
echo ""
echo "📚 次のステップ:"
echo "1. リモート URL を SSH に変更"
echo "   cd /home/user/Workspace/rustgames"
echo "   git remote set-url origin git@github.com:user/rustgames.git"
echo ""
echo "2. GitHub へプッシュ"
echo "   git push origin main"
echo ""

# パスワードをクリア
unset SERVER_PASSWORD
