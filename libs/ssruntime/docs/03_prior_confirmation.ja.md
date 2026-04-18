# 事前確認
libssruntime は Rust で実装しており、libssruntime から受け取るデータフォーマットを FlatBuffers を採用しており下記の確認をお願いします。

# Rust のビルドサポート
Player を実装する対象のゲームエンジンがサポートする CPU アーキテクチャ、プラットフォーム を把握し、Rust がビルドに対応しているかを確認してください。

https://doc.Rust-lang.org/beta/Rustc/platform-support.html

libssruntime のリリース成果物に対象の CPU アーキテクチャ / プラットフォーム のライブラリバイナリが含まれていない場合は自前でのビルドをお願いします。

# プログラミング言語のネイティブラリブラリのバインディング
Player を実装する対象のゲームエンジンとプログラミング言語について確認してください。

- ゲームエンジンがネイティブライブラリ or wasm を取り込めるかを確認してください。
- プログラミング言語から C/C++ ライブラリ(or wasm ライブラリ)の関数をバインディングして実行できるかを確認してください。

# FlatBuffers のサポート
Player を実装する対象ゲームエンジンのプログラミング言語を把握し、FlatBuffers の flatc コマンドがその言語に対応しているかを確認してください。

https://FlatBuffers.dev/support/

---
[目次に戻る](./README.ja.md)
