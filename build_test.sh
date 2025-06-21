#!/bin/bash

echo "🔧 Saten画像ビューア - MVC リファクタリング完了後のビルドテスト"
echo "=========================================================="

# プロジェクトディレクトリに移動
cd "$(dirname "$0")"

echo "📁 現在のディレクトリ: $(pwd)"
echo ""

echo "1️⃣  構文チェック実行中..."
cargo check --verbose

if [ $? -eq 0 ]; then
    echo "✅ 構文チェック成功！"
    echo ""
    
    echo "2️⃣  テスト実行中..."
    cargo test --verbose
    
    if [ $? -eq 0 ]; then
        echo "✅ テスト成功！"
        echo ""
        
        echo "3️⃣  リリースビルド実行中..."
        cargo build --release --verbose
        
        if [ $? -eq 0 ]; then
            echo "✅ リリースビルド成功！"
            echo ""
            echo "🎉 すべてのチェックが完了しました！"
            echo ""
            echo "次のコマンドでアプリケーションを実行できます:"
            echo "  cargo run"
            echo ""
            echo "🏗️  MVCリファクタリングが完全に成功しました！"
        else
            echo "❌ リリースビルドに失敗しました"
            exit 1
        fi
    else
        echo "❌ テストに失敗しました"
        exit 1
    fi
else
    echo "❌ 構文チェックに失敗しました"
    echo ""
    echo "🔍 一般的なライフタイム問題の対処法:"
    echo "  1. Element<'static, Message> の使用を確認"
    echo "  2. Text::new() で動的文字列を使用する際は .to_string() を追加"
    echo "  3. &Vec<u8> ではなく &[u8] を使用"
    echo "  4. モジュールインポートが正しいか確認"
    exit 1
fi
