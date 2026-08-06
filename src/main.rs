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
    actions: Vec<Action>,
}
#[derive(Deserialize, Debug, Clone)]
struct Core {
    avatar_url: String,
    gemini_api_model: String,
}

#[derive(Deserialize, Debug, Clone)]
struct DiscordMessageConfig {
    username: String,
    content_key: String,
}

#[derive(Deserialize, Debug, Clone)]
struct Action {
    key: String,
    modifiers: Vec<String>,
    prompt: String,
    response_schema: Value,
    discord_messages: Vec<DiscordMessageConfig>,
}

async fn castg(
    api_key: &str,
    webhook_url: &str,
    avatar_url: &str,
    api_model: &str,
    prompt: &str,
    response_schema: &Value,
    discord_messages: &[DiscordMessageConfig],
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
        match serde_json::from_str::<serde_json::Value>(text) {
            Ok(parsed) => {
                for msg_config in discord_messages {
                    let mut username = msg_config.username.clone();
                    if username.starts_with('$') {
                        let key = &username[1..];
                        if let Some(val) = parsed.get(key) {
                            if let Some(s) = val.as_str() {
                                username = s.to_string();
                            }
                        }
                    }

                    let mut content = String::new();
                    if let Some(val) = parsed.get(&msg_config.content_key) {
                        if let Some(arr) = val.as_array() {
                            let strings: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
                            content = strings.join("\n");
                        } else if let Some(s) = val.as_str() {
                            content = s.to_string();
                        } else {
                            content = val.to_string();
                        }
                    }

                    client
                        .post(webhook_url)
                        .json(&json!({
                            "username": username,
                            "content": content,
                            "avatar_url": avatar_url
                        }))
                        .send()
                        .await?;
                }

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
        }
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

    let (tx, mut rx) = mpsc::channel::<usize>(10);

    let config_clone = config.clone();
    thread::spawn(move || {
        let mut hkm = HotkeyManager::new();

        for (index, action) in config_clone.actions.iter().enumerate() {
            let trigger_key = VKey::from_keyname(&action.key).unwrap();
            let mut modifiers = Vec::new();
            for mod_str in &action.modifiers {
                let mod_key = VKey::from_keyname(mod_str).unwrap();
                modifiers.push(mod_key);
            }

            let tx_clone = tx.clone();
            hkm.register_hotkey(trigger_key, &modifiers, move || if tx_clone.blocking_send(index).is_err() {})
                .unwrap();
        }

        println!("ホットキー押下待ち");

        hkm.event_loop();
    });

    while let Some(index) = rx.recv().await {
        println!("翻訳処理開始");

        if let Some(action) = config.actions.get(index) {
            let key = api_key.clone();
            let webhook = webhook_url.clone();
            let avatar_url = config.core.avatar_url.clone();
            let model = config.core.gemini_api_model.clone();
            let prompt = action.prompt.clone();
            let response_schema = action.response_schema.clone();
            let discord_messages = action.discord_messages.clone();

            tokio::spawn(async move {
                if let Err(_e) = castg(&key, &webhook, &avatar_url, &model, &prompt, &response_schema, &discord_messages).await {}
            });
        }
    }

    Ok(())
}
