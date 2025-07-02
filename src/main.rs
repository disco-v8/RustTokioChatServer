// RustTokioChatServer - 非同期チャットサーバー メインプログラム
// MIT License
//
// クレート説明:
// - tokio: 非同期ランタイム、TCP通信、シグナル処理など
// - chrono, chrono-tz: 日時・タイムゾーン処理
// - std: 標準ライブラリ、スレッド同期や入出力
//
// 必要なクレートを読み込み
use tokio::{net::TcpListener, sync::broadcast}; // Tokio: TCPリスナーとブロードキャストチャネル
#[cfg(windows)]
use tokio::io::AsyncReadExt; // Tokio: 非同期read（Windowsのみ）
use std::{sync::{Arc, RwLock}}; // std: スレッド安全な参照カウント・ロック
use chrono_tz::Asia::Tokyo; // chrono-tz: JSTタイムゾーン
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind}; // Tokio: Unixシグナル受信（UNIXのみ）

mod init; // 設定読み込み用モジュール
use init::load_config; // 設定ファイル読込関数のみuse
mod client; // クライアント処理モジュール

// JSTタイムスタンプ付きログ出力マクロ（クレート全体で利用可能）
#[macro_export] // クレート全体で利用できるようにエクスポート
macro_rules! printdaytimeln { // ログ出力用マクロ定義
    ($($arg:tt)*) => {{ // 可変引数を受け取る
        let now = chrono::Local::now().with_timezone(&Tokyo); // 現在時刻をJSTで取得
        let log_time = now.format("[%Y/%m/%d %H:%M:%S]"); // タイムスタンプを整形
        println!("{} {}", log_time, format!($($arg)*)); // タイムスタンプ付きでログ出力
    }};
}

// メイン関数（Tokioランタイム）
#[tokio::main] // Tokioランタイムで非同期実行
async fn main() { // メイン関数本体
    // 設定ファイルを初回読み込み
    let config = Arc::new(RwLock::new(load_config())); // 設定をスレッド安全に共有

    // メッセージ用ブロードキャストチャネルを作成
    let (msg_tx, _) = broadcast::channel::<String>(100); // 全クライアント間メッセージ用
    // 接続済クライアントへの通知用ブロードキャストチャネルを作成
    let (shutdown_tx, _) = broadcast::channel::<()>(100); // シャットダウン通知用

    // SIGHUPを受信するための非同期タスクを起動（UNIXのみ）
    #[cfg(unix)]
    {
        let config = Arc::clone(&config); // 設定の参照をクローン
        let shutdown_tx_hup = shutdown_tx.clone(); // SIGHUP用
        let shutdown_tx_term = shutdown_tx.clone(); // SIGTERM用

        // SIGHUPハンドラ
        tokio::spawn(async move {
            let mut hup = signal(SignalKind::hangup()).expect("SIGHUP登録失敗"); // SIGHUPシグナル受信設定
            while hup.recv().await.is_some() { // SIGHUP受信ループ
                printdaytimeln!("SIGHUP受信：設定ファイルを再読み込み"); // ログ出力
                let new_config = load_config(); // 設定再読込
                *config.write().unwrap() = new_config; // 設定を更新
                let _ = shutdown_tx_hup.send(()); // 全クライアントに通知
            }
        });

        // SIGTERMハンドラ
        tokio::spawn(async move {
            let mut term = signal(SignalKind::terminate()).expect("SIGTERM登録失敗"); // SIGTERMシグナル受信設定
            while term.recv().await.is_some() { // SIGTERM受信ループ
                printdaytimeln!("SIGTERM受信：サーバーを安全に終了します"); // ログ出力
                let _ = shutdown_tx_term.send(()); // 全クライアントに通知
                std::process::exit(0); // プロセス終了
            }
        });
    }
    // Windows用：CTRL-Y/CTRL-Cで再読込・終了
    #[cfg(windows)]
    {
        let config = Arc::clone(&config); // 設定の参照をクローン
        let shutdown_tx = shutdown_tx.clone(); // チャネルをクローン
        tokio::spawn(async move { // 非同期タスクを生成
            let mut stdin = tokio::io::stdin(); // 標準入力ハンドルを取得
            let mut buf = [0u8; 1]; // 1バイトバッファ
            loop {
                if let Ok(n) = stdin.read(&mut buf).await { // 標準入力から1バイト読む
                    if n == 1 && buf[0] == 0x19 { // 0x19はCTRL-Y
                        printdaytimeln!("CTRL-Y受信：設定ファイルを再読み込み"); // ログ出力
                        let new_config = load_config(); // 設定再読込
                        *config.write().unwrap() = new_config; // 設定を更新
                        let _ = shutdown_tx.send(()); // 全クライアントに通知
                    } else if n == 1 && buf[0] == 0x03 { // 0x03はCTRL-C
                        printdaytimeln!("CTRL-C受信：サーバーを終了します"); // ログ出力
                        std::process::exit(0); // 正常終了
                    }
                }
            }
        }); // タスク終了
    }

    loop { // メインループ
        // 現在の設定を読み取る
        let current_config = config.read().unwrap().clone(); // 設定を取得
        printdaytimeln!("設定読込: {}", current_config.address); // ログ出力

        // TCP待受開始
        let bind_result = TcpListener::bind(&current_config.address).await; // 指定アドレスでバインド

        let listener = match bind_result { // バインド結果で分岐
            Ok(listener) => {
                printdaytimeln!("待受開始: {}", current_config.address); // バインド成功時に再度ログ
                listener // リスナーを返す
            },
            Err(e) => {
                eprintln!(
                    "ポートバインドに失敗しました: {}\n既に他のプロセスが {} を使用中かもしれません。",
                    e,
                    current_config.address
                ); // エラー出力
                std::process::exit(1); // 異常終了
            }
        };

        // 接続ごとに処理を分ける
        let mut shutdown_rx = shutdown_tx.subscribe(); // ループ外でレシーバを作成
        loop {
            tokio::select! {
                // 新しい接続を受け付けた場合
                Ok((stream, addr)) = listener.accept() => { // 新規接続受信
                    printdaytimeln!("接続: {}", addr); // ログ出力
                    let shutdown_rx = shutdown_tx.subscribe(); // クライアントごとにレシーバ作成
                    let msg_tx = msg_tx.clone(); // メッセージ用Senderをクローン
                    tokio::spawn(client::handle_client(stream, shutdown_rx, msg_tx)); // クライアント処理を非同期で開始
                }
                // 再起動通知を受けたら、bindし直すためループを抜ける
                _ = shutdown_rx.recv() => { // 再起動通知受信
                    printdaytimeln!("再起動のためリスナー再バインド"); // ログ出力
                    break; // 内部ループを抜けて再バインド
                }
            }
        }
    }
}
