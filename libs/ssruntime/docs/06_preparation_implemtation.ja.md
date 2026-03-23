# Player 実装前段階

# Player 用プロジェクトの用意
- Player の空プロジェクトを作成し、ビルドをできるようにしてください。
- FlatBuffers の flatc コマンドを利用して ssab.fbs と framedata.fsb から利用するプログラミング言語のパースコードを作成してください。
- 出力されたパースコードと FlatBuffers のライブラリを Player のプロジェクトへ取り込んでビルドができるようにしてください。
- libssruntime をプロジェクトへ取り込んで成果物にリンクし、ビルドができるようにしてください。

# ssab ファイルの用意
ssconverter-cli を用いて tests/overall/overall.sspj から ssab ファイルを作成してください。
ssab ファイルの成果物を Player を実装するプロジェクトへアセットとして登録してください。

# libssruntime の組み込み手順（詳細）
※ ここには、選択したライブラリタイプ（静的/動的）に応じたプロジェクト設定や、Rust のプラットフォームサポートに関する具体的な構築手順を記載します。

- **静的ライブラリの場合**: リンカへの追加、依存ライブラリの解決。
- **動的ライブラリの場合**: 実行環境でのパスの通し方、FFI のバインディング生成。
- **WebAssembly の場合**: emscripten や wasm-bindgen を用いた統合。
