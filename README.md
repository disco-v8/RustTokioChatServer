# RustTokioChatServer

非同期チャットサーバー（Rust + Tokio実装）

## 概要

RustTokioChatServerは、Rustの非同期ランタイム「Tokio」を使用したシンプルなチャットサーバーです。
IPv4/IPv6デュアルスタック対応で、複数クライアントが同時接続してリアルタイムチャットを楽しめます。

## 特徴

- **非同期処理**: Tokioによる高性能な非同期TCP通信
- **デュアルスタック対応**: IPv4/IPv6両対応（設定により選択可能）
- **設定ファイル対応**: `RustTokioChatServer.conf`で簡単設定
- **クロスプラットフォーム**: Windows/Linux/macOS対応
- **シグナル処理**: SIGHUP/SIGTERMによる設定再読込・安全終了（Unix系）
- **リアルタイム**: ブロードキャストによる即座のメッセージ配信

## システム要件

- Rust 1.70以降
- Cargo

## インストール・ビルド

```bash
# リポジトリをクローン
git clone https://github.com/YOUR_USERNAME/RustTokioChatServer.git
cd RustTokioChatServer

# ビルド
cargo build --release

# 実行
cargo run --release
```

## 設定

設定ファイル `src/RustTokioChatServer.conf` を編集してサーバーの動作をカスタマイズできます。

```
# チャットサーバー設定ファイル
# ポート番号のみを指定した場合、[::]:ポートでデュアルスタック対応
Listen = 8080

# 特定のIPアドレスとポートを指定することも可能
# Listen = 127.0.0.1:8080    # IPv4のみ
# Listen = [::1]:8080        # IPv6のみ
# Listen = [::]:8080         # デュアルスタック（明示的）
```

### Listen設定の仕様

- **ポート番号のみ** (例: `8080`): `[::]:8080`として解釈され、IPv4/IPv6デュアルスタック対応
- **IPv4アドレス:ポート** (例: `127.0.0.1:8080`): IPv4のみでバインド
- **IPv6アドレス:ポート** (例: `[::1]:8080`): IPv6のみでバインド
- **[::]:ポート**: OS設定に依存するデュアルスタック動作

## 使用方法

1. サーバーを起動
```bash
cargo run --release
```

2. telnetやncコマンドでクライアント接続
```bash
# IPv4接続
telnet localhost 8080

# IPv6接続
telnet ::1 8080
```

3. メッセージを入力してエンターキーを押すと、接続中の全クライアントにブロードキャスト

## 動作環境での操作

### Unix系OS（Linux/macOS）での操作
- **設定再読込**: `kill -HUP <プロセスID>`
- **安全終了**: `kill -TERM <プロセスID>` または `Ctrl+C`

### Windows での操作
- **安全終了**: `Ctrl+C`

## 依存クレート

- `tokio`: 非同期ランタイム（TCP通信、シグナル処理など）
- `chrono`: 日時処理
- `chrono-tz`: タイムゾーン処理
- `lazy_static`: 静的変数管理

## アーキテクチャ

```
src/
├── main.rs               # メインプログラム（サーバー起動・シグナル処理）
├── init.rs               # 設定ファイル読み込み
├── client.rs             # クライアント接続・メッセージ処理
└── RustTokioChatServer.conf  # 設定ファイル
```

## 技術仕様

- **非同期処理**: Tokioのasync/await
- **同期プリミティブ**: Arc<RwLock<T>>によるスレッドセーフなデータ共有
- **通信**: TCP（IPv4/IPv6対応）
- **メッセージ配信**: tokio::sync::broadcastチャネル
- **ログ出力**: JSTタイムスタンプ付きマクロ

## ライセンス

MIT License - 詳細は [LICENSE](LICENSE) ファイルを参照してください。

## 開発・貢献

Issues、Pull Requestsを歓迎します。

---

**Note**: このプロジェクトは学習・実験目的で作成されています。本格的な製品環境での使用には、追加のセキュリティ対策や機能拡張が必要な場合があります。

# ビルド
cargo build --release

# 実行
cargo run
```

## 設定ファイル

`RustTokioChatServer.conf`で動作設定を変更できます：

```properties
# ポート番号のみ指定（IPv4/IPv6デュアルスタック）
Listen 8667

# IPv4専用
Listen 0.0.0.0:8667

# IPv6専用
Listen [::]:8667

# 特定IPアドレス
Listen 192.168.1.100:8667

# ハンドルネーム最大長
MaxHandleName 32

# メッセージ最大長
MaxMessageLength 255
```

## 使用方法

1. サーバー起動
   ```bash
   cargo run
   ```

2. クライアント接続（telnet等）
   ```bash
   # IPv4接続
   telnet 127.0.0.1 8667
   
   # IPv6接続
   telnet ::1 8667
   ```

3. ハンドルネーム入力後、チャット開始

## 操作方法

### サーバー側
- **CTRL-Y**: 設定ファイル再読込（Windows）
- **CTRL-C**: サーバー終了
- **SIGHUP**: 設定ファイル再読込（Unix系）
- **SIGTERM**: 安全終了（Unix系）

### クライアント側
- **CTRL-Y**: ハンドルネーム再設定
- **CTRL-C/CTRL-D**: 切断

## 依存クレート

- **tokio**: 非同期ランタイム・TCP通信・シグナル処理
- **chrono**: 日時処理
- **chrono-tz**: タイムゾーン処理（JST対応）
- **lazy_static**: グローバル変数初期化

## ライセンス

MIT License - 詳細は[LICENSE](LICENSE)をご覧ください。

## 作者

T.Kabu/MyDNS.JP

## 貢献

プルリクエストやIssueでの改善提案を歓迎します。
