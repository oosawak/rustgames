#!/bin/bash

# GitHub にプッシュするスクリプト
# PAT トークンを対話的に入力

set -e

echo "🚀 GitHub プッシュスクリプト"
echo "================================"
echo ""

# リポジトリディレクトリに移動
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$REPO_DIR"

# GitHub ユーザー名
GITHUB_USER="oosawak"

# トークンを隠し入力
echo "GitHub Personal Access Token を入力してください："
echo "(入力は画面に表示されません)"
read -sp "Token: " PAT_TOKEN
echo ""

# 空のトークンをチェック
if [ -z "$PAT_TOKEN" ]; then
    echo "❌ エラー: トークンが入力されていません"
    exit 1
fi

# git config で認証情報をセット（メモリ内に24時間のみ保存）
echo ""
echo "認証情報を設定中（キャッシュ: 24時間）..."
git config --global credential.helper 'cache --timeout=86400'
git config --global user.name "oosawak" 2>/dev/null || true
git config --global user.email "your_email@example.com" 2>/dev/null || true

# リモートの URL を HTTPS に変更（必要に応じて）
REMOTE_URL=$(git config --get remote.origin.url)
if [[ $REMOTE_URL == git@github.com:* ]]; then
    # SSH URL を HTTPS に変換
    # git@github.com:oosawak/rustgames.git -> https://github.com/oosawak/rustgames.git
    REPO_NAME="${REMOTE_URL##*:}"
    HTTPS_URL="https://github.com/${REPO_NAME}"
    echo "リモート URL を HTTPS に変更: $HTTPS_URL"
    git remote set-url origin "$HTTPS_URL"
fi

# 認証ヘルパーに認証情報を渡す
echo "プッシュ中..."
if git -c credential.helper='cache --timeout=86400' push origin main; then
    echo ""
    echo "✅ プッシュ成功！"
    echo ""
    echo "📚 次のステップ:"
    echo "1. GitHub リポジトリに移動"
    echo "2. Settings → Pages"
    echo "3. Source: deploy from a branch"
    echo "4. Branch: main, Folder: /docs"
    echo "5. Save"
    echo ""
    echo "その後、数分で https://github.com/oosawak/rustgames/settings/pages で確認できます"
else
    echo "❌ プッシュに失敗しました"
    exit 1
fi

# トークンをメモリからクリア
unset PAT_TOKEN
