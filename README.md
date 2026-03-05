# Game Lens
ホットキーで画面をキャプチャしてGoogle Gemini APIに投げてOCRと翻訳を行いその結果をDiscordに流すやつ。  
今のところDungeons & Dragons Online用。

## メモ
- Google AI Studioで試した範囲ではGemini 2.0 Flashが安くて良かったが新規ユーザーにはAPIが解放されていない
- gemini-2.5-flashでthinkingBudgetを指定しないとやたら考えてるのか返事が遅くなることがある
- たぶん1回0.2から0.3円ぐらい
- キャプチャの後のリサイズとwebpへの変換で0.5秒ぐらいかかっており地味に長い
- 最新のAPIでキャプチャできるライブラリも見つけた [Atliac/wgc: An ergonomic Rust wrapper for Windows.Graphics.Capture API](https://github.com/atliac/wgc)
- 現状DDO専用だしそれで良いが複数ゲーム対応にしたいときどうすんのが良いのか全然わからんね

