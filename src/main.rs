use base64::prelude::*;
use image::DynamicImage;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{Value, json};
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
    thinking_level: String,
    loading_message: String,
    system_instruction: String,
}

#[derive(Deserialize, Debug, Clone)]
struct DiscordMessageConfig {
    username: String,
    content_key: String,
    #[serde(default)]
    as_markdown_list: bool,
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
    client: &Client,
    api_key: &str,
    webhook_url: &str,
    avatar_url: &str,
    api_model: &str,
    thinking_level: &str,
    loading_message: &str,
    system_instruction: &str,
    prompt: &str,
    response_schema: &Value,
    discord_messages: &[DiscordMessageConfig],
) -> Result<(), Box<dyn std::error::Error>> {
    let base64_image = {
        let monitors = Monitor::all()?;
        if monitors.is_empty() {
            return Err("モニターが見つからなかった".into());
        }
        let image = monitors[0].capture_image()?;

        let rgb_image = DynamicImage::ImageRgba8(image)
            .resize(1024, 1024, image::imageops::FilterType::Triangle)
            .to_rgb8();

        let webp = Encoder::new_rgb(rgb_image.as_raw(), rgb_image.width(), rgb_image.height())
            .quality(40.0)
            .encode(Unstoppable)
            .map_err(|e| format!("WebPエンコードエラー: {:?}", e))?;

        // fs::write("optimized.webp", &webp).unwrap();

        let webp_bytes: &[u8] = &*webp;
        BASE64_STANDARD.encode(webp_bytes)
    };

    client
        .post(webhook_url)
        .json(&json!({
            "username": "Game Lens",
            "content": loading_message,
            "avatar_url": avatar_url,
        }))
        .send()
        .await?;

    let payload = json!({
        "systemInstruction": {
            "parts": [
                { "text": system_instruction }
            ]
        },
        "generationConfig": {
            "responseMimeType": "application/json",
            "responseSchema": response_schema,
            "thinkingConfig": {
                "thinkingLevel": thinking_level
            }
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
                            let strings: Vec<String> = arr.iter().filter_map(|v| {
                                v.as_str().map(|s| {
                                    if msg_config.as_markdown_list {
                                        format!("- {}", s)
                                    } else {
                                        s.to_string()
                                    }
                                })
                            }).collect();
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

    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 { &args[1] } else { "config.toml" };

    let config_str = fs::read_to_string(config_path).unwrap_or_else(|e| {
        eprintln!("{} の読み込みに失敗した: {}", config_path, e);
        std::process::exit(1);
    });
    let config: Config = toml::from_str(&config_str).unwrap_or_else(|e| {
        eprintln!("{} のパースに失敗した: {}", config_path, e);
        std::process::exit(1);
    });

    let client = Client::new();

    let (tx, mut rx) = mpsc::channel::<usize>(10);

    let config_clone = config.clone();
    thread::spawn(move || {
        let mut hkm = HotkeyManager::new();

        for (index, action) in config_clone.actions.iter().enumerate() {
            let trigger_key = match VKey::from_keyname(&action.key) {
                Ok(k) => k,
                Err(_) => {
                    eprintln!("無効なキー名 ({}): {}", index, action.key);
                    continue;
                }
            };

            let mut modifiers = Vec::new();
            let mut has_invalid_mod = false;
            for mod_str in &action.modifiers {
                match VKey::from_keyname(mod_str) {
                    Ok(k) => modifiers.push(k),
                    Err(_) => {
                        eprintln!("無効な修飾キー名: {}", mod_str);
                        has_invalid_mod = true;
                    }
                }
            }
            if has_invalid_mod {
                continue;
            }

            let tx_clone = tx.clone();
            if let Err(e) = hkm.register_hotkey(trigger_key, &modifiers, move || if tx_clone.blocking_send(index).is_err() {}) {
                eprintln!("ホットキーの登録に失敗した ({:?} + {:?}): {}", trigger_key, modifiers, e);
            }
        }

        println!("ホットキー押下待ち");

        hkm.event_loop();
    });

    while let Some(index) = rx.recv().await {
        println!("翻訳処理開始");

        if let Some(action) = config.actions.get(index) {
            let client_clone = client.clone();
            let key = api_key.clone();
            let webhook = webhook_url.clone();
            let avatar_url = config.core.avatar_url.clone();
            let model = config.core.gemini_api_model.clone();
            let thinking_level = config.core.thinking_level.clone();
            let loading_msg = config.core.loading_message.clone();
            let sys_inst = config.core.system_instruction.clone();
            let prompt = action.prompt.clone();
            let response_schema = action.response_schema.clone();
            let discord_messages = action.discord_messages.clone();

            tokio::spawn(async move {
                if let Err(e) = castg(
                    &client_clone,
                    &key,
                    &webhook,
                    &avatar_url,
                    &model,
                    &thinking_level,
                    &loading_msg,
                    &sys_inst,
                    &prompt,
                    &response_schema,
                    &discord_messages,
                )
                .await
                {
                    eprintln!("実行エラー: {}", e);
                }
            });
        }
    }

    Ok(())
}
