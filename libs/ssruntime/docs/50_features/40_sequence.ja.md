> **日本語** | [English](./40_sequence.md)

# シーケンス

シーケンスは、複数のアニメーションを順に再生する設定です。ssconverter が ssqe から ssqb を作ります。Brain にはアニメーションを順に切り替える機能が無いので、シーケンスの進行は Player が管理します。

## 1. ssqb に入っているもの

ssqb のスキーマは `libs/ssab/fbs/ssqb.fbs` にあります（ファイル識別子は `"SSQB"`）。

| テーブル | フィールド | 内容 |
| :--- | :--- | :--- |
| `SsSequenceBinary` | `sequence_list` | シーケンスの一覧 |
| `Sequence` | `name` / `name_hash` | シーケンスの名前とハッシュ |
| | `type` | シーケンスの最後での挙動（`Last` / `Keep` / `Top`） |
| | `list` | 順に再生するアイテム |
| `SequenceItem` | `ref_anime_pack` / `ref_anime_pack_hash` | アニメーションが入っている ssab の名前とハッシュ |
| | `ref_anime_name` / `ref_anime_name_hash` | アニメーションの名前とハッシュ |
| | `repeat_count` | そのアイテムを繰り返す回数 |

アイテムが参照する ssab は、インスタンスパーツの外部参照と同じ方法で探します（[外部リソースとテクスチャ §4](./30_external_resources.ja.md#4-外部-ssab-の参照)）。

## 2. 再生の進め方

1. アイテムのアニメーションをセットアップし、`repeat_count` をループ回数に設定して再生する。
2. 毎フレーム `ss_runtime_is_finished` の立ち上がりで、アイテムの終わりを検出する（[再生制御 §7](../30_core/20_playback_control.ja.md#7-ループと終了の検知)）。
3. 次のアイテムをセットアップして再生する。別の ssab のアイテムなら、先にその ssab のリソースをバインドする。
4. 最後のアイテムが終わったら、`type` に従って止めるか続ける。

---
[目次に戻る](../README.ja.md)
