> **日本語** | [English](./README.md)

# SpriteStudio Player 移植ガイド

`libssruntime`（Brain）を使って、任意のゲームエンジンやプラットフォームに SpriteStudio の Player を実装する方法を説明します。Brain はアニメーションの状態を計算し、Player は ssab の読み込み・描画・イベントの通知・音の再生を行います（[概要](./10_introduction/10_overview.ja.md)）。

- **最初に読むもの**: [制限事項](./10_introduction/20_limitations.ja.md) と [Player API 規約](./20_design/30_api_conventions.ja.md)。どちらも Player の設計を左右します。
- **ページの書き方**: 各ページの冒頭に、そのページで Player が何をするかを書いています。「〜してください」「〜しないでください」は守るべきこと、「推奨します」は選べることです。用語は [用語集](./10_introduction/30_glossary.ja.md) の意味で使います。
- **コード例**: C-API を呼ぶ C++ で書いています。実在しない関数（エンジンの描画処理など）は、例のそばにその旨を書いています。

---

## 1. 実装の進め方

SDK の `tests/overall/overall.sspj` には、機能ごとのアニメーションが入っています。次の順に実装し、各段階で **SpriteStudio のエディタと同じ見た目になるか** を比べてください。同じになれば、その段階は完了です。

| 段階 | 確かめるアニメーション | 確かめること | 読むページ |
| :--- | :--- | :--- | :--- |
| 0. 準備 | — | ビルドが通り、ssab からアニメーション名を読める | [事前確認](./20_design/10_preparation.ja.md)、[設計方針](./20_design/20_architecture.ja.md) |
| 1. 基本の描画 | `Basic.ssae` | 親の移動・回転・拡縮に子が付いていく。親の不透明度を下げると子も薄くなる | [ロードと初期化](./30_core/10_initialization.ja.md)、[再生制御](./30_core/20_playback_control.ja.md)、[描画ループ](./30_core/40_render_loop.ja.md) |
| 2. 合成と色 | `BlendMode.ssae`、`PartsColor.ssae` | 加算や乗算が背景に正しく重なる。パーツ全体や頂点ごとの着色が合う | [パーツカラー](./40_parts/10_parts_color.ja.md) |
| 3. 構造と変形 | `Mesh.ssae`、`Instance.ssae`、`Mask.ssae` | メッシュの変形とボーンの追従が滑らか。子アニメーションが再生される。マスクの範囲だけが表示される | [メッシュ](./40_parts/20_mesh.ja.md)、[インスタンスパーツ](./40_parts/50_instance.ja.md)、[マスク](./40_parts/40_mask.ja.md) |
| 4. ゲームとの連携 | `UserData.ssae`、`Signal.ssae` | 決まったフレームでイベントが届く | [イベント](./30_core/30_events.ja.md) |
| 5. エフェクト | `Effect.ssae` | パーティクルが飛び散り、時間とともに消える | [エフェクトパーツ](./40_parts/60_effect.ja.md) |
| 6. 必要に応じて | `NineSlice.ssae`、`Shape.ssae`、`Text.ssae`、`Sound.ssae`、`TextureChange.ssae`、`InstancePropagation.ssae` | 拡大しても 9 スライスの角が保たれる。図形と文字が出る。決まったフレームで音が鳴る。テクスチャ変更で 2 つのパーツが別のテクスチャで描かれる。子アニメーションの中のマスクが親に効く | [9 スライス・シェイプ・テキスト](./40_parts/30_special_parts.ja.md)、[オーディオ](./50_features/50_audio.ja.md)、[外部リソースとテクスチャ](./50_features/30_external_resources.ja.md)、[マスク §2-7](./40_parts/40_mask.ja.md#2-7-インスタンスの中で書かれたクリッピングの持ち越し) |

---

## 2. 目次

### 10. はじめに

| ページ | 内容 |
| :--- | :--- |
| [概要](./10_introduction/10_overview.ja.md) | Brain と Player の分担、読者、コード例の読み方 |
| [制限事項](./10_introduction/20_limitations.ja.md) | Player の設計に影響する技術的な制約。**実装の前に読む** |
| [用語集](./10_introduction/30_glossary.ja.md) | このガイドで使う用語 |

### 20. 設計

| ページ | 内容 |
| :--- | :--- |
| [事前確認とプロジェクトの準備](./20_design/10_preparation.ja.md) | 対象のエンジンで Brain を使えるかの確認と、空のプロジェクトの用意 |
| [Player の設計方針](./20_design/20_architecture.ja.md) | ライブラリの組み込み方、ssab のメモリ、クラス構成、描画方式、座標系 |
| [Player API 規約](./20_design/30_api_conventions.ja.md) | すべての Player で揃える名前・引数の順・イベント・既定値・失敗の伝え方。**公開 API を書く前に読む** |
| [no_std 環境への統合](./20_design/40_no_std_integration.ja.md) | 自分でビルドする場合と、エンジンのアロケータを使う場合 |

### 30. 基本の再生

| ページ | 内容 |
| :--- | :--- |
| [ロードと初期化](./30_core/10_initialization.ja.md) | リソースと Context、ssab の検証、テクスチャの読み込み、ログとエラー |
| [再生制御](./30_core/20_playback_control.ja.md) | 1 フレームの流れ、再生の状態、速度、サブフレーム、ループと終了の検知 |
| [イベント](./30_core/30_events.ja.md) | ユーザーデータ・シグナル・オーディオのイベントの読み方 |
| [描画ループ](./30_core/40_render_loop.ja.md) | SoA バッファ、DrawBatch による描画、RenderMode、性能 |

### 40. パーツ

| ページ | 内容 |
| :--- | :--- |
| [パーツカラー](./40_parts/10_parts_color.ja.md) | 頂点の色とテクスチャの合成、シェーダの例 |
| [メッシュ](./40_parts/20_mesh.ja.md) | 変形済みの頂点の読み方 |
| [9 スライス・シェイプ・テキスト](./40_parts/30_special_parts.ja.md) | 専用のバッファの読み方と、テキストのレイアウトの仕様 |
| [マスク](./40_parts/40_mask.ja.md) | マスクとクリッピングの判定の規則と、ステンシル・被覆ビットマップでの実装 |
| [インスタンスパーツ](./40_parts/50_instance.ja.md) | 子アニメーションの用意・進め方・描き方、独立再生 |
| [エフェクトパーツ](./40_parts/60_effect.ja.md) | パーティクルの進め方と描き方 |

### 50. 機能

| ページ | 内容 |
| :--- | :--- |
| [アニメーションミキシング](./50_features/10_animation_mixing.ja.md) | トラックに積んで重ねる・切り替える・混ぜる |
| [パーツオーバーライド](./50_features/20_part_overrides.ja.md) | パーツのカラー・セル・表示の実行時の上書き |
| [外部リソースとテクスチャ](./50_features/30_external_resources.ja.md) | テクスチャの差し替えと、別の ssab の参照 |
| [シーケンス](./50_features/40_sequence.ja.md) | 複数のアニメーションを順に再生する |
| [オーディオ](./50_features/50_audio.ja.md) | 音を鳴らす条件と、音の寿命 |
| [エディタ統合](./50_features/60_editor_integration.ja.md) | エンジンのエディタで sspj を変換する |

### 60. トラブルシューティング

| ページ | 内容 |
| :--- | :--- |
| [トラブルシューティング](./60_troubleshooting/10_troubleshooting.ja.md) | 症状から原因を探す表と、リンクのエラー |

---
[SpriteStudio-SDK](https://github.com/cri-middleware/SpriteStudio-SDK/blob/HEAD/README.ja.md) / [libssruntime](https://github.com/cri-middleware/SpriteStudio-SDK/blob/HEAD/libs/ssruntime/README.ja.md)
