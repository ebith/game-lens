use base64::prelude::*;
use image::DynamicImage;
use log::info;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::env;
use std::fs;
use std::thread;
use tokio::sync::mpsc;
use webpx::{Encoder, Unstoppable};
use win_hotkeys::{HotkeyManager, VKey};
use xcap::Monitor;

#[derive(Deserialize, Debug)]
struct Config {
    core: Core,
}
#[derive(Deserialize, Debug)]
struct Core {
    key: String,
    modifiers: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct DDO {
    speaker: String,
    body_text: Vec<String>,
    player_options: Vec<String>,
}

async fn send_to_discord(webhook_url: &str, ddo: &DDO) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    client.post(webhook_url).json(&json!({
        "username": ddo.speaker,
        "content": ddo.body_text.join("\n"),
        "avatar_url": "https://images-ext-1.discordapp.net/external/SfHNcPgzdvapQgglxxfodbDxqC3Z7bTNGdrfsb87FBE/%3Fv%3D1482477394/https/www.ddo.com/images/global/header/ddo-logo-small.png?format=webp&quality=lossless",
    })).send().await?;

    client.post(webhook_url).json(&json!({
        "username": "選択肢",
        "content": ddo.player_options.join("\n"),
        "avatar_url": "https://images-ext-1.discordapp.net/external/SfHNcPgzdvapQgglxxfodbDxqC3Z7bTNGdrfsb87FBE/%3Fv%3D1482477394/https/www.ddo.com/images/global/header/ddo-logo-small.png?format=webp&quality=lossless",
    })).send().await?;

    Ok(())
}

async fn castg(api_key: &str, webhook_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let base64_image = {
        let monitors = Monitor::all().unwrap();
        let image = monitors[0].capture_image().unwrap();
        info!("キャプチャ成功");

        let rgb_image = DynamicImage::ImageRgba8(image)
            .resize(1024, 1024, image::imageops::FilterType::Triangle)
            .to_rgb8();
        info!("リサイズした");

        let webp = Encoder::new_rgb(rgb_image.as_raw(), rgb_image.width(), rgb_image.height())
            .quality(40.0)
            .encode(Unstoppable)
            .unwrap();
        info!("webpに変換した");

        // fs::write("optimized.webp", &webp).unwrap();

        let webp_bytes: &[u8] = &*webp;
        BASE64_STANDARD.encode(webp_bytes)
    };
    info!("base64に変換した");

    let client = Client::new();

    client.post(webhook_url).json(&json!({
        "username": "Game Lens",
        "content": "翻訳処理中…",
        "avatar_url": "https://images-ext-1.discordapp.net/external/SfHNcPgzdvapQgglxxfodbDxqC3Z7bTNGdrfsb87FBE/%3Fv%3D1482477394/https/www.ddo.com/images/global/header/ddo-logo-small.png?format=webp&quality=lossless",
    })).send().await?;

    info!("Gemini APIへ送信した");

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );
    let payload = json!({
        "generationConfig": {
            "responseMimeType": "application/json",
            "responseSchema": {
                "type": "OBJECT",
                "properties": {
                    "speaker": {"type": "STRING", "description": "話者の名前"},
                    "body_text":{"type": "ARRAY", "items": {"type": "STRING"}, "description": "会話本文"},
                    "player_options":{"type": "ARRAY", "items": {"type": "STRING"}, "description": "返事の選択肢"},
                },
                "required":["speaker", "body_text", "player_options"]
            },
            "thinkingConfig": {"thinkingBudget": 600},
        },
        "contents":[{
            "parts":[
                {"text": "画像中の黒い背景のダイアログの会話文を日本語に翻訳してください。"},
                {
                    "inlineData": {
                        "mimeType": "image/webp",
                        "data": base64_image
                    }
                }
            ]
        }]
    });
    let response: serde_json::Value = client.post(&url).json(&payload).send().await?.json().await?;
    if let Some(text) = response["candidates"][0]["content"]["parts"][0]["text"].as_str() {
        match serde_json::from_str::<DDO>(text) {
            Ok(analysis) => {
                send_to_discord(&webhook_url, &analysis).await?;
                info!("Discordに送った");
                info!(
                    "トークン使用量: 入力: {:?}, 出力: {:?}, 思考: {:?}, 合計: {:?}",
                    response["usageMetadata"]["promptTokenCount"].to_string(),
                    response["usageMetadata"]["candidatesTokenCount"].to_string(),
                    response["usageMetadata"]["thoughtsTokenCount"].to_string(),
                    response["usageMetadata"]["totalTokenCount"].to_string(),
                );
            }
            Err(e) => {
                info!("JSONのパースに失敗した: {}", e);
            }
        }
    } else {
        info!("{:#?}", response);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder().format_timestamp_micros().init();
    let api_key = env::var("GEMINI_API_KEY").expect("環境変数 GEMINI_API_KEYが空っぽだぞ");
    let webhook_url = env::var("DISCORD_WEBHOOK_URL").expect("環境変数 DISCORD_WEBHOOK_URLが空っぽだぞ");
    // let title = env::var("GAME_LENS_TARGET").expect("環境変数 GAME_LENS_TARGETが空っぽだぞ");

    let config_str = fs::read_to_string("config.toml").unwrap();
    let config: Config = toml::from_str(&config_str).unwrap();

    // info!("{:#?}", config);

    let (tx, mut rx) = mpsc::channel::<()>(10);

    thread::spawn(move || {
        let trigger_key = VKey::from_keyname(&config.core.key).unwrap();
        let mut modifiers = Vec::new();
        for mod_str in &config.core.modifiers {
            let mod_key = VKey::from_keyname(mod_str).unwrap();
            modifiers.push(mod_key);
        }

        let mut hkm = HotkeyManager::new();

        hkm.register_hotkey(trigger_key, &modifiers, move || if tx.blocking_send(()).is_err() {})
            .unwrap();

        info!("ホットキー押下待ち");

        hkm.event_loop();
    });

    while let Some(_) = rx.recv().await {
        let key = api_key.clone();
        let webhook = webhook_url.clone();

        info!("ホットキーが押された");
        tokio::spawn(async move { if let Err(_e) = castg(&key, &webhook).await {} });
    }

    Ok(())
}
