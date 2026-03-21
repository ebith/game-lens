# Game Lens
ホットキーで画面をキャプチャしてGoogle Gemini APIに投げてOCRと翻訳を行いその結果をDiscordに流すやつ。  
今のところDungeons & Dragons Online用。

こういうゲーム画面で指定のホットキーを押すと、  
<img src="https://github.com/user-attachments/assets/0efd2e0e-43fe-495f-b713-eecb4369589f"  width="33%"/>

Discordにこんな感じに流れてくる。  
<img src="https://github.com/user-attachments/assets/6c9549cc-06ec-4e9e-89d4-662ec02325d4" width="33%"/>

## メモ
- ホットキー2つ設定することでクエスト会話ウインドウと左下のチャット欄どっちも翻訳できるようにした
- Gemini 3.1 Flash-Lite Previewだと入力545 出力100-220トークンみたいな感じになるので1回0.05-0.08円
- Gemini 2.5 Flashだと入力276 出力200+思考トークンという感じで1回0.3円前後
- 単発のスクリーンショット目的だとGDI([XCap](https://github.com/nashaofu/xcap))の方がWGC([wgc](https://github.com/Atliac/wgc))よりだいぶ早かった
  - wgcのexamplesに従うと画面の縁がチカチカしちゃう問題もある
- 現状DDO専用だしそれで良いが複数ゲーム対応にしたいときどうすんのが良いのか全然わからんね
  - [pacak/bpaf: Command line parser with applicative interface](https://github.com/pacak/bpaf)とかで対応することになんのかな
