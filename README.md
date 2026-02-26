# Game Lens
指定のフォルダを監視してjpgファイルを見つけたらGoogle Gemini AIに投げてOCRと翻訳を行いその結果をDiscordに流すやつ。  
今のところDungeons & Dragons Online用。

## メモ
- 理想はホットキーで画面をキャプチャしてそれをGeminiに投げるという流れだが、手軽にグローバルホットキー用意する手段を見つけられなかった
- Google AI Studioで試した範囲ではGemini 2.0 Flashが安くて良かったが新規ユーザーにはAPIが解放されていない
- gemini-2.5-flashでthinkingBudgetを指定しないとやたら考えてるのか返事が遅くなることがある
- たぶん1回0.2円ぐらい
- `(await sharp(path).webp({ quality: 30 }).toBuffer()).toString('base64')`でファイルサイズは1/4になるが0.1秒以上かかる

