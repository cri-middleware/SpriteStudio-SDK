> **日本語** | [English](./80_sequence.md)

# シーケンス (Sequence) の実装

`.ssqb` ファイルに定義された「アニメーションの連結再生リスト」を処理する機能です。`libssruntime` 自体は単一のアニメーション再生に特化しているため、シーケンスの管理は Player 側のロジックで行います。

- **再生フロー**:
    1. `ss_runtime_is_finished` で現在のアニメーションの終了を検知します（[再生制御とイベント処理](../30_core/20_playback_control.ja.md) 参照）。
    2. シーケンスリストの次の項目を確認します。
    3. `ss_runtime_setup_animation` で次のアニメーションをセットアップし、再生を継続します。
- **ループ設定**: シーケンス全体のループや、特定の項目のみのループなど、Player 側でステートマシンを構築して制御します。

> **将来計画 (Phase 3)**: 上記は現時点で推奨される Player 側の実装です。将来的に、シーケンス再生（線形・自動送りのステートマシン）を `libssruntime` 側に内包する機能を予定しています。詳細は [ROADMAP.md](https://github.com/cri-middleware/SpriteStudio-SDK/blob/HEAD/ROADMAP.md) を参照してください。

---
[目次に戻る](../README.ja.md)
