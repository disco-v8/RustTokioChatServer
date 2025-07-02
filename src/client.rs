// RustTokioChatServer - クライアント通信処理モジュール
// MIT License
//
// クレート説明:
// - tokio: 非同期TCP通信・I/O・ブロードキャスト
// - chrono-tz: JSTタイムゾーン処理
// - std: 標準ライブラリ（コレクション・同期）
// - lazy_static: グローバル静的変数
//
// client.rs: クライアントとの通信処理を分離
// 必要なクレートをインポート
use tokio::{net::TcpStream, io::{AsyncReadExt, AsyncWriteExt}, sync::broadcast}; // Tokio: TCPストリーム・非同期I/O・ブロードキャスト
use chrono_tz::Asia::Tokyo; // chrono-tz: JSTタイムゾーン
use crate::init; // 設定管理モジュール
use std::collections::HashSet; // std: ハンドルネーム一覧用コレクション
use std::sync::Mutex; // std: スレッド安全なミューテックス
use lazy_static::lazy_static; // lazy_static: グローバル静的変数

// グローバルなハンドルネーム一覧
lazy_static! {
    static ref HANDLE_NAMES: Mutex<HashSet<String>> = Mutex::new(HashSet::new()); // ハンドルネームを保持
}

// クライアントとの通信処理（1接続あたり1スレッド）
pub async fn handle_client(
    mut stream: TcpStream, // クライアントとのTCPストリーム
    mut shutdown_rx: broadcast::Receiver<()>, // サーバーからのシャットダウン通知受信用
    msg_tx: broadcast::Sender<String>, // メッセージ送信用
) {
    let mut msg_rx = msg_tx.subscribe(); // メッセージ受信用Receiver
    let mut buf = [0u8; 1024]; // 受信バッファ
    let mut handle_name = String::new(); // ハンドルネーム
    let peer_addr = match stream.peer_addr() { // クライアントアドレス取得
        Ok(addr) => addr.to_string(), // アドレス取得成功
        Err(_) => "unknown".to_string(), // 失敗時はunknown
    };
    let mut line_buf = Vec::new(); // 受信データを一時的に溜めるバッファ
    let mut phase = 0; // 0:ハンドルネーム未定義, 1:通常エコー
    let config = init::CONFIG.read().unwrap().clone(); // 設定値を取得
    let welcome_msg = format!("\
##############################################\n\
#### Welcome to Rust Simple Chat Server\n\
#### You must be set HandleName, And Enjoy!\n\
#### MaxHandleName Length : {}\n\
#### MaxMessageLength Length : {}\n\
#### CTRL-Y : Reset your HandleName.\n\
#### CTRL-D : Disconnect\n\
##############################################\n\
", config.max_handle_name, config.max_message_length); // ウェルカムメッセージ生成
    if stream.write_all(welcome_msg.as_bytes()).await.is_err() { // クライアントに送信し失敗したら
        return; // 切断
    }
    // ここで現在の他クライアントのハンドルネーム一覧を送信
    let list_msg = {
        let names = HANDLE_NAMES.lock().unwrap(); // ハンドルネーム一覧をロック
        if names.is_empty() {
            "現在他のクライアントはいません\n".to_string() // 他に誰もいない場合
        } else {
            let list = names.iter().cloned().collect::<Vec<_>>().join(", "); // 一覧をカンマ区切りで連結
            format!("現在接続中の他クライアント: {}\n", list) // 一覧メッセージ生成
        }
    }; // MutexGuardはここでドロップされる
    let _ = stream.write_all(list_msg.as_bytes()).await; // 一覧をクライアントに送信
    loop { // メインループ
        if phase == 0 && handle_name.is_empty() { // ハンドルネーム未定義なら入力促し
            let prompt = "SYSTEM> ハンドルネームを入力してください\n"; // 入力促しメッセージ
            if stream.write_all(prompt.as_bytes()).await.is_err() { // 送信失敗時は切断
                return;
            }
        }
        let config = init::CONFIG.read().unwrap().clone(); // 設定を都度取得
        tokio::select! {
            // クライアントからの入力
            Ok(n) = stream.read(&mut buf) => {
                if n == 0 {
                    crate::printdaytimeln!("切断: {} {}", peer_addr, handle_name); // 切断ログ
                    // 切断時にハンドルネームを一覧から削除
                    if !handle_name.is_empty() {
                        HANDLE_NAMES.lock().unwrap().remove(&handle_name); // 削除
                    }
                    break;
                }
                line_buf.extend_from_slice(&buf[..n]); // バッファに追記
                while line_buf.len() < config.max_message_length {
                    if line_buf.contains(&0x03) || line_buf.contains(&0x04) { // CTRL-C/CTRL-D検出
                        crate::printdaytimeln!("切断: {} {} (CTRL-C/CTRL-D検出)", peer_addr, handle_name); // ログ
                        if !handle_name.is_empty() {
                            HANDLE_NAMES.lock().unwrap().remove(&handle_name); // 削除
                        }
                        return;
                    }
                    if let Some(pos) = line_buf.iter().position(|&b| b == b'\n' || b == b'\r') { // 改行検出
                        let line = line_buf.drain(..=pos).collect::<Vec<u8>>(); // 1行分取り出し
                        let msg = String::from_utf8_lossy(&line).trim().to_string(); // UTF-8変換
                        if line.contains(&0x03) || line.contains(&0x04) { // CTRL-C/CTRL-D検出
                            crate::printdaytimeln!("切断: {} {}", peer_addr, handle_name); // ログ
                            if !handle_name.is_empty() {
                                HANDLE_NAMES.lock().unwrap().remove(&handle_name); // 削除
                            }
                            return;
                        }
                        if phase == 0 {
                            if msg.is_empty() {
                                continue; // 空行は無視
                            }
                            if !msg.chars().all(|c| !c.is_control() && !c.is_whitespace()) {
                                let _ = stream.write_all("SYSTEM> ハンドルネームに使えない文字が含まれています\n".as_bytes()).await; // バリデーション
                                continue;
                            }
                            if msg.as_bytes().len() > config.max_handle_name {
                                let _ = stream.write_all("SYSTEM> ハンドルネームが長すぎます\n".as_bytes()).await; // 長さ超過
                                crate::printdaytimeln!("切断: {} ハンドルネーム長オーバー", peer_addr); // ログ
                                return;
                            }
                            handle_name = msg.clone(); // ハンドルネーム確定
                            // ハンドルネームを一覧に追加
                            HANDLE_NAMES.lock().unwrap().insert(handle_name.clone());
                            phase = 1; // 通常モードへ
                            crate::printdaytimeln!("確定: {} {}", peer_addr, handle_name); // ログ
                            let welcome = format!("SYSTEM> {}さん、ようこそ\n", handle_name); // ウェルカム
                            let _ = stream.write_all(welcome.as_bytes()).await;
                            continue;
                        }
                        if phase == 1 && line.contains(&0x19) { // CTRL-Yで再定義
                            let old = handle_name.clone();
                            // 再定義時は古いハンドルネームを削除
                            HANDLE_NAMES.lock().unwrap().remove(&old);
                            handle_name.clear();
                            phase = 0;
                            crate::printdaytimeln!("再定義: {} {} -> (未定義)", peer_addr, old); // ログ
                            continue;
                        }
                        if !msg.is_empty() {
                            let now = chrono::Local::now().with_timezone(&Tokyo); // 現在時刻
                            let time_str = now.format("%Y/%m/%d %H:%M").to_string(); // タイムスタンプ
                            let echo = format!("{}> {} ({})\n", handle_name, msg, time_str); // メッセージ整形
                            // 自分のメッセージを全体にブロードキャスト
                            let _ = msg_tx.send(format!("{}", echo));
                        }
                    } else {
                        break; // 改行がなければ抜ける
                    }
                }
                if line_buf.len() >= config.max_message_length {
                    let _ = stream.write_all("SYSTEM> 一行が長すぎます\n".as_bytes()).await; // 長さ超過
                    line_buf.clear(); // バッファクリア
                }
            }
            // 他クライアントからのメッセージを受信して自分に送信
            Ok(broadcast_msg) = msg_rx.recv() => {
                // 自分の送信分はスキップ
//                if !broadcast_msg.starts_with(&handle_name) {
//                    let _ = stream.write_all(broadcast_msg.as_bytes()).await;
//                }
                // フィルタせず全てのメッセージを自分にも送信
                let _ = stream.write_all(broadcast_msg.as_bytes()).await;            }
            // サーバー再起動通知受信時
            _ = shutdown_rx.recv() => {
                let _ = stream.write_all("サーバーを再起動するので切断します\n".as_bytes()).await; // 通知
                // シャットダウン時もハンドルネームを削除
                if !handle_name.is_empty() {
                    HANDLE_NAMES.lock().unwrap().remove(&handle_name); // 削除
                }
                break; // ループ終了
            }
        }
    }
}
