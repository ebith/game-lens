# Game Lens
ホットキーで画面をキャプチャしてGoogle Gemini APIに投げてOCRと翻訳を行いその結果をDiscordに流すやつ。  
Dungeons & Dragons Online用として作り始めたがAntigravityとGeminiのお陰で汎用ツールになった。

こういうゲーム画面で指定のホットキーを押すと、  
<img src="https://github.com/user-attachments/assets/0efd2e0e-43fe-495f-b713-eecb4369589f"  width="33%"/>

Discordにこんな感じに流れてくる。  
<img src="https://github.com/user-attachments/assets/6c9549cc-06ec-4e9e-89d4-662ec02325d4" width="33%"/>

## メモ
- 目の前のNPCとかオブジェクトの解説もさせてみたらとても便利になった
- デフォルトでは`config.toml`を読むが`game-lens.exe ddo.toml`とかやると別ファイルも読めるようになった
- ハードコードしてた部分を`config.toml`から読むようにしたので汎用ツールになった
- Gemini Flash Lite系の無料枠が15RPM/500RPDなのでこのツールで使う程度だと無料でいけそう
- Gemini 3.1 Flash-Lite Previewだと入力545 出力100-220トークンみたいな感じになるので1回0.05-0.08円
- Gemini 2.5 Flashだと入力276 出力200+思考トークンという感じで1回0.3円前後
- 単発のスクリーンショット目的だとGDI([XCap](https://github.com/nashaofu/xcap))の方がWGC([wgc](https://github.com/Atliac/wgc))よりだいぶ早かった
  - wgcのexamplesに従うと画面の縁がチカチカしちゃう問題もある
