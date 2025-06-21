# Saten 画像ビューア - MVCリファクタリング完了

## 🎉 リファクタリング完了報告

Saten画像ビューアのコードベースを**Model-View-Controller (MVC)** パターンに完全リファクタリングしました！

### 📊 改善成果
- **main.rs**: 700行 → 80行（約88%削減）
- **保守性**: 関心の分離により大幅向上
- **拡張性**: 新機能追加が容易に
- **テスト可能性**: 各層を独立してテスト可能

## 🏗️ 新しいアーキテクチャ

### Model層 (データ・ビジネスロジック)
```
src/model/
├── app_state.rs      # アプリケーション状態管理
├── archive_manager.rs # アーカイブファイル処理
├── image_manager.rs   # 画像データ管理
└── page_manager.rs    # ページナビゲーション
```

### View層 (ユーザーインターフェース)
```
src/view/
├── app_view.rs       # メインビュー構築
├── image_view.rs     # 画像表示コンポーネント
└── layout.rs         # レイアウトヘルパー
```

### Controller層 (イベント処理・制御)
```
src/controller/
├── app_controller.rs    # 中央制御
├── keyboard_handler.rs  # キーボード操作
└── file_handler.rs      # ファイル処理
```

## 🚀 使用方法

### ビルド・実行
```bash
# プロジェクトディレクトリに移動
cd C:\kakiwork\dev\rust\saten

# 構文チェック
cargo check

# ビルド
cargo build

# 実行
cargo run
```

### 基本操作
| キー | 機能 |
|------|------|
| `←` | 次のページ |
| `→` | 前のページ |
| `↑` | 前のファイル |
| `↓` | 次のファイル |
| `1` | シングルページモード |
| `2` | ダブルページモード |
| `R` | 回転モード切り替え |
| `Home` | 最初のページ |
| `End` | 最後のページ |
| `Space` | 次のページ |
| `Backspace` | 前のページ |

### 高度な操作
| キー組み合わせ | 機能 |
|----------------|------|
| `Shift + ←` | 10ページ進む |
| `Shift + →` | 10ページ戻る |
| `Ctrl + R` | アプリケーションリセット |

### 対応フォーマット
- **アーカイブ**: RAR4, RAR5, ZIP
- **画像**: JPEG, PNG, GIF, BMP, WebP, TIFF

## 🧪 テスト実行

```bash
# 全テスト実行
cargo test

# 特定の層のテスト
cargo test model::
cargo test controller::

# テスト詳細表示
cargo test -- --nocapture
```

## 🔧 開発者向け情報

### 新機能の追加方法

#### 1. Model層への追加
```rust
// src/model/your_feature.rs
pub struct YourFeature {
    // フィールド定義
}

impl YourFeature {
    pub fn new() -> Self {
        // 実装
    }
}
```

#### 2. View層への追加
```rust
// src/view/your_view.rs
impl YourView {
    pub fn create_component<Message>() -> Element<Message> {
        // UI コンポーネント実装
    }
}
```

#### 3. Controller層への追加
```rust
// src/controller/your_handler.rs
impl YourHandler {
    pub fn handle_event(event: YourEvent) -> Command<Message> {
        // イベント処理実装
    }
}
```

### コーディング規約
1. **エラーハンドリング**: `Result<T, ArchiveError>` を使用
2. **ログ出力**: `log::info!`, `log::error!` を活用
3. **テスト**: 各機能に対応するテストを作成
4. **ドキュメント**: パブリック関数には説明コメントを付与

## 📚 ファイル説明

### Core Files
- `main.rs` - アプリケーションエントリーポイント
- `lib.rs` - テストファイル

### Model Layer
- `app_state.rs` - アプリケーション状態（ページ、モード等）
- `archive_manager.rs` - アーカイブファイル読み込み・解凍
- `image_manager.rs` - 画像処理・フォーマット検証
- `page_manager.rs` - ページナビゲーション制御

### View Layer  
- `app_view.rs` - メインビュー・レイアウト構築
- `image_view.rs` - 画像表示・ビューア作成
- `layout.rs` - 共通レイアウトヘルパー

### Controller Layer
- `app_controller.rs` - イベント振り分け・中央制御
- `keyboard_handler.rs` - キーボードショートカット処理
- `file_handler.rs` - ファイルドロップ・読み込み処理

### Legacy Files (既存機能)
- `archive_reader.rs` - アーカイブ読み込み共通定義
- `file_checker.rs` - ファイル形式判定
- `reader_*.rs` - 各フォーマット固有の読み込み処理
- `sort_filename.rs` - ファイル名ソート
- `compress_deflate.rs` - Deflate圧縮解除

## 🐛 トラブルシューティング

### コンパイルエラー
```bash
# 依存関係の問題
cargo clean
cargo build

# 構文チェック
cargo check
```

### 実行時エラー
1. **ファイルが開けない**: 対応フォーマット（RAR/ZIP）か確認
2. **画像が表示されない**: アーカイブ内に画像ファイルがあるか確認
3. **動作が重い**: 大きなファイルの場合は時間がかかる場合があります

## 🛣️ 今後のロードマップ

### Phase 1 (短期)
- [ ] 設定ファイル対応
- [ ] ショートカットカスタマイズ
- [ ] テーマシステム

### Phase 2 (中期)
- [ ] サムネイル表示
- [ ] フルスクリーンモード
- [ ] アニメーション効果

### Phase 3 (長期)
- [ ] プラグインシステム
- [ ] クラウド連携
- [ ] AI機能統合

## 📞 サポート

問題や質問がある場合：
1. まず[トラブルシューティング](#-トラブルシューティング)を確認
2. `cargo test`でテストが通るか確認
3. ログ出力を確認（`log::debug!`等）

---

**🎊 MVCリファクタリング完了！より良いコードベースで開発を継続できます！**
