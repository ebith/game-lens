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
- Gemini Flash Lite系の無料枠が15RPM/500RPDなのでこのツールで使う程度だと無料でいける
- 単発のスクリーンショット目的だとWGC([wgc](https://github.com/Atliac/wgc))よりGDI([XCap](https://github.com/nashaofu/xcap))の方が気持ち速くかつ圧倒的に安定している
