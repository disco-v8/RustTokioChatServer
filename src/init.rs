// RustTokioChatServer - 設定管理モジュール
// MIT License
//
// クレート説明:
// - std: 標準ライブラリ、ファイル入出力・同期
// - lazy_static: グローバル変数の初期化
//
// init.rs: 初期化処理を分離
#[derive(Debug, Clone)] // Debug出力とCloneを可能にする属性
pub struct Config { // サーバー設定情報を格納する構造体
    pub address: String, // 待受アドレス
    pub max_handle_name: usize, // ハンドルネーム最大長
    pub max_message_length: usize, // メッセージ最大長
}

pub fn load_config() -> Config { // 設定ファイルからConfigを生成する関数
    let text = std::fs::read_to_string("RustTokioChatServer.conf").expect("設定ファイル読み込み失敗"); // 設定ファイルを読み込む（失敗時はpanic）
    let mut address = None; // アドレス初期値（未設定）
    let mut max_handle_name = 32; // ハンドルネーム最大長の初期値
    let mut max_message_length = 256; // メッセージ最大長の初期値
    for line in text.lines() { // 各行をループ
        let line = line.trim(); // 前後の空白を除去
        if let Some(rest) = line.strip_prefix("Listen ") { // Listen行を検出
            let addr = rest.trim(); // アドレス部分を取得
            if addr.contains(':') {
                // IPアドレス:ポート形式
                address = Some(addr.to_string()); // 指定アドレスでバインド（IPv4/IPv6どちらでも可）
            } else {
                // ポート番号のみ指定時はIPv4/IPv6両対応の[::]:ポートでバインド
                address = Some(format!("[::]:{}", addr));
            }
        } else if let Some(rest) = line.strip_prefix("MaxHandleName ") { // MaxHandleName行を検出
            if let Ok(val) = rest.trim().parse::<usize>() { // 数値変換に成功したら
                max_handle_name = val; // ハンドルネーム最大長を設定
            }
        } else if let Some(rest) = line.strip_prefix("MaxMessageLength ") { // MaxMessageLength行を検出
            if let Ok(val) = rest.trim().parse::<usize>() { // 数値変換に成功したら
                max_message_length = val; // メッセージ最大長を設定
            }
        }
    }
    // Listen行がなければデフォルトで127.0.0.1:8667を使用
    let address = address.unwrap_or_else(|| "127.0.0.1:8667".to_string()); // デフォルトアドレス
    Config {
        address, // アドレス
        max_handle_name, // ハンドルネーム最大長
        max_message_length, // メッセージ最大長
    }
}

use std::sync::RwLock; // RwLockをインポート

lazy_static::lazy_static! { // lazy_staticでグローバルな設定を定義
    pub static ref CONFIG: RwLock<Config> = RwLock::new(load_config()); // グローバル設定（再読み込み対応）
}
