# Contributing to RustTokioChatServer

RustTokioChatServerへの貢献を歓迎します！

## 開発環境の準備

1. Rust（1.70以降）とCargoをインストール
2. リポジトリをフォーク・クローン
3. ブランチを作成して開発

```bash
git clone https://github.com/disco-v8/RustTokioChatServer.git
cd RustTokioChatServer
git checkout -b feature/new-feature
```

## コーディング規約

- Rustの標準的なフォーマット（`cargo fmt`）を使用
- Clippyの警告に対応（`cargo clippy`）
- テストの追加（`cargo test`）
- 適切なコメント・ドキュメントの追加

## プルリクエストの手順

1. フォークしたリポジトリで変更を実装
2. テストが通ることを確認
3. コミットメッセージは分かりやすく記述
4. プルリクエストを作成
5. レビューコメントに対応

## 問題の報告

バグや改善提案はIssuesで報告してください。以下の情報を含めてください：

- OS・Rustバージョン
- 再現手順
- 期待される動作と実際の動作
- エラーメッセージ（あれば）

## ライセンス

貢献されたコードはMIT Licenseでライセンスされます。
