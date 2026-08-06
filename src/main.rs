use base64::prelude::*;
use image::DynamicImage;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::thread;
use tokio::sync::mpsc;
use webpx::{Encoder, Unstoppable};
use win_hotkeys::{HotkeyManager, VKey};
use xcap::Monitor;

#[derive(Deserialize, Debug, Clone)]
struct Config {
    core: Core,
    hotkeys: Vec<Hotkey>,
}
#[derive(Deserialize, Debug, Clone)]
struct Core {
    avatar_url: String,
    gemini_api_model: String,
}

#[derive(Deserialize, Debug, Clone)]
struct Hotkey {
    command: u8,
    key: String,
    modifiers: Vec<String>,
    prompt: String,
    response_schema: Value,
}

#[derive(Deserialize, Debug)]
struct Quest {
    speaker: String,
    body_text: Vec<String>,
    player_options: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct Chat {
    lines: Vec<String>,
}

async fn castg(
    api_key: &str,
    webhook_url: &str,
    command: &u8,
    avatar_url: &str,
    api_model: &str,
    prompt: &str,
    response_schema: &Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let base64_image = {
        let monitors = Monitor::all().unwrap();
        let image = monitors[0].capture_image().unwrap();

        let rgb_image = DynamicImage::ImageRgba8(image)
            .resize(1024, 1024, image::imageops::FilterType::Triangle)
            .to_rgb8();

        let webp = Encoder::new_rgb(rgb_image.as_raw(), rgb_image.width(), rgb_image.height())
            .quality(40.0)
            .encode(Unstoppable)
            .unwrap();

        // fs::write("optimized.webp", &webp).unwrap();

        let webp_bytes: &[u8] = &*webp;
        BASE64_STANDARD.encode(webp_bytes)
    };

    let client = Client::new();

    client
        .post(webhook_url)
        .json(&json!({
            "username": "Game Lens",
            "content": "翻訳処理中…",
            "avatar_url": avatar_url,
        }))
        .send()
        .await?;

    let payload = json!({
        "generationConfig": {
            "responseMimeType": "application/json",
            "responseSchema": response_schema,
        },
        "contents":[{
            "parts":[
                {"text": prompt},
                {
                    "inlineData": {
                        "mimeType": "image/webp",
                        "data": base64_image
                    },
                    "media_resolution": {"level": "MEDIA_RESOLUTION_MEDIUM"},
                }
            ]
        }]
    });

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        api_model, api_key
    );
    let response: serde_json::Value = client.post(&url).json(&payload).send().await?.json().await?;
    if let Some(text) = response["candidates"][0]["content"]["parts"][0]["text"].as_str() {
        match command {
            1 => match serde_json::from_str::<Quest>(text) {
                Ok(analysis) => {
                    client
                        .post(webhook_url)
                        .json(&json!({
                            "username": &analysis.speaker,
                            "content": &analysis.body_text.join("\n"),
                            "avatar_url": avatar_url
                        }))
                        .send()
                        .await?;

                    client
                        .post(webhook_url)
                        .json(&json!({
                            "username": "選択肢",
                            "content": &analysis.player_options.join("\n"),
                            "avatar_url": avatar_url
                        }))
                        .send()
                        .await?;
                    println!(
                        "トークン使用量: 入力: {:?}, 出力: {:?}, 思考: {:?}, 合計: {:?}",
                        response["usageMetadata"]["promptTokenCount"].to_string(),
                        response["usageMetadata"]["candidatesTokenCount"].to_string(),
                        response["usageMetadata"]["thoughtsTokenCount"].to_string(),
                        response["usageMetadata"]["totalTokenCount"].to_string(),
                    );
                }
                Err(e) => {
                    println!("JSONのパースに失敗した: {}", e);
                }
            },
            2 => match serde_json::from_str::<Chat>(text) {
                Ok(analysis) => {
                    client
                        .post(webhook_url)
                        .json(&json!({
                            "username": "チャット欄",
                            "content": &analysis.lines.join("\n"),
                            "avatar_url": avatar_url
                        }))
                        .send()
                        .await?;
                    println!(
                        "トークン使用量: 入力: {:?}, 出力: {:?}, 思考: {:?}, 合計: {:?}",
                        response["usageMetadata"]["promptTokenCount"].to_string(),
                        response["usageMetadata"]["candidatesTokenCount"].to_string(),
                        response["usageMetadata"]["thoughtsTokenCount"].to_string(),
                        response["usageMetadata"]["totalTokenCount"].to_string(),
                    );
                }
                Err(e) => {
                    println!("JSONのパースに失敗した: {}", e);
                }
            },
            _ => {}
        };
    } else {
        println!("{:#?}", response);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("GEMINI_API_KEY").expect("環境変数 GEMINI_API_KEYが空っぽだぞ");
    let webhook_url = env::var("DISCORD_WEBHOOK_URL").expect("環境変数 DISCORD_WEBHOOK_URLが空っぽだぞ");

    let config_str = fs::read_to_string("config.toml").unwrap();
    let config: Config = toml::from_str(&config_str).unwrap();

    // println!("{:#?}", config);

    let (tx, mut rx) = mpsc::channel::<u8>(10);

    let config_clone = config.clone();
    thread::spawn(move || {
        let mut hkm = HotkeyManager::new();

        for hotkey in &config_clone.hotkeys {
            let trigger_key = VKey::from_keyname(&hotkey.key).unwrap();
            let mut modifiers = Vec::new();
            for mod_str in &hotkey.modifiers {
                let mod_key = VKey::from_keyname(mod_str).unwrap();
                modifiers.push(mod_key);
            }

            let tx_clone = tx.clone();
            let command = hotkey.command.clone();
            hkm.register_hotkey(trigger_key, &modifiers, move || if tx_clone.blocking_send(command).is_err() {})
                .unwrap();
        }

        println!("ホットキー押下待ち");

        hkm.event_loop();
    });

    while let Some(command) = rx.recv().await {
        println!("翻訳処理開始");

        if let Some(hotkey) = config.hotkeys.iter().find(|h| h.command == command) {
            let key = api_key.clone();
            let webhook = webhook_url.clone();
            let avatar_url = config.core.avatar_url.clone();
            let model = config.core.gemini_api_model.clone();
            let prompt = hotkey.prompt.clone();
            let response_schema = hotkey.response_schema.clone();

            tokio::spawn(async move {
                if let Err(_e) = castg(&key, &webhook, &command, &avatar_url, &model, &prompt, &response_schema).await {}
            });
        }
    }

    Ok(())
}
