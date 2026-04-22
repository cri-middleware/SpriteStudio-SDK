# Player 移植ガイド (Player Implementation Guide)

このディレクトリには、`libssruntime` を使用して独自の SpriteStudio 7 プレイヤーを実装するための詳細なドキュメントが含まれています。

機能の検証には、テストプロジェクト **`tests/overall/overall.sspj`** の利用を推奨しています。第1章のロードマップに沿って実装を進めることで、効率的に開発を行うことができます。

## 目次

### 第1章：概要と準備 (Overview & Preparation)
実装を始める前に読むべき「概念」と「準備」についてです。
1. [はじめに・用語解説](./01_introduction.ja.md)
2. [アーキテクチャと実装方針](./02_architecture.ja.md)
3. [事前確認とプロジェクトの準備](./03_preparation.ja.md)
4. [実装ロードマップ (overall.sspj による機能検証)](./04_roadmap.ja.md)

### 第2章：基本実装 (Core Implementation)
最小限のアニメーションを画面に出すまでのステップです。
5. [データのロードと初期化](./05_initialization.ja.md)
6. [アニメーションの設定と再生制御](./06_playback_control.ja.md)
7. [描画ループと基本アトリビュートの適用](./07_render_loop.ja.md)

### 第3章：応用実装 (Advanced Features)
よりリッチなアニメーションを再現するための追加実装です。
8. [高度な機能と描画ループの拡張](./08_advanced_rendering.ja.md)
9. [イベント処理 (UserData / Signal / サウンド)](./09_events.ja.md)
10. [Optional な機能の実装](./10_optional_features.ja.md)

### 第4章：リファレンス (Reference)
11. [`no_std` 組み込み環境への統合](./11_no_std_integration.ja.md)
12. [トラブルシューティングと制限事項](./12_troubleshooting.ja.md)

---
[SpriteStudio7-SDK Top](../../README.ja.md) / [libssruntime Top](../README.ja.md)
