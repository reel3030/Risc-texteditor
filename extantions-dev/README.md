# HTML タグ補完拡張

この開発用拡張は HTML の `>` 入力時に閉じタグを自動挿入しないよう設定し、既存の HTML Language Server が提供する補完候補を維持します。`</` 入力時の自動挿入には、同梱された Zed 本体の変更も必要です。

詳細な要件は [仕様書](./SPECIFICATION.md) を参照してください。

## Zed へのインストール

1. Zed の Extensions ページを開きます。
2. `Install Dev Extension` を選択します。
3. このフォルダーを指定します。

この拡張は既存の `html` 拡張を置き換えます。元に戻すには、Extensions ページから dev extension を削除または無効化してください。

## 動作

- `<p>` 入力で `</p>` を自動挿入しません。
- `<h` で表示される `h1` などの候補を含め、HTML Language Server の補完機能を維持します。
- `</` の入力で、対応する最も内側の未閉鎖タグを自動挿入します。
- `</` 入力時の自動挿入は、Zed 本体に含まれる `crates/editor/src/html_tag_auto_close.rs` の変更によって実現します。
