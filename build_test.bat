@echo off
chcp 65001 > nul

echo 🔧 Saten画像ビューア - MVC リファクタリング完了後のビルドテスト
echo ==========================================================

echo 📁 現在のディレクトリ: %cd%
echo.

echo 1️⃣  構文チェック実行中...
cargo check --verbose

if %errorlevel% equ 0 (
    echo ✅ 構文チェック成功！
    echo.
    
    echo 2️⃣  テスト実行中...
    cargo test --verbose
    
    if %errorlevel% equ 0 (
        echo ✅ テスト成功！
        echo.
        
        echo 3️⃣  リリースビルド実行中...
        cargo build --release --verbose
        
        if %errorlevel% equ 0 (
            echo ✅ リリースビルド成功！
            echo.
            echo 🎉 すべてのチェックが完了しました！
            echo.
            echo 次のコマンドでアプリケーションを実行できます:
            echo   cargo run
            echo.
            echo 🏗️  MVCリファクタリングが完全に成功しました！
        ) else (
            echo ❌ リリースビルドに失敗しました
            pause
            exit /b 1
        )
    ) else (
        echo ❌ テストに失敗しました
        pause
        exit /b 1
    )
) else (
    echo ❌ 構文チェックに失敗しました
    echo.
    echo 🔍 一般的なライフタイム問題の対処法:
    echo   1. Element^<'static, Message^> の使用を確認
    echo   2. Text::new^(^) で動的文字列を使用する際は .to_string^(^) を追加
    echo   3. ^&Vec^<u8^> ではなく ^&[u8] を使用
    echo   4. モジュールインポートが正しいか確認
    pause
    exit /b 1
)

pause
