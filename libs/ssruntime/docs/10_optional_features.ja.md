# 実装フェーズ 6：外部リソースとオプション機能

大規模なプロジェクトや、特定の演出で必要となる発展的な機能の実装ガイドです。

## 1. 外部リソースとテクスチャの管理

### 外部テクスチャのオーバーライド (Texture Override)
特定のパーツやセルマップのテクスチャを、実行時に動的に差し替える機能です。

- **実装方法**: Player 側で `cellmap_name_hash` (またはセルマップ名文字列) をキーとしたテクスチャオブジェクトのマップを保持します。
- **解決手順**:
    1. 初期化時に通常通り SSAB の定義に従ってテクスチャをロードし、マップに格納します。
    2. プログラムから特定のキーに対して別のテクスチャオブジェクトを上書き（オーバーライド）します。
    3. **オーバーライドの解除（復元）**: オーバーライドに `nullptr` (または `null`) が指定された場合は、SSAB に定義されているオリジナルのテクスチャパスから画像を再ロードし、マップの状態を初期値に復元します。
    4. 描画時、`DrawBatch` の `texture_hash()` をキーにマップから対象のテクスチャをルックアップして描画処理に渡します。これにより、元のデータを変更せずに衣装替えや動的な表情変更が可能になります。

```cpp
// テクスチャのオーバーライド（差し替え）の実装例 (C++)
void Player::set_cellmap_texture(const std::string& cellmap_name, Texture* override_texture) {
    uint32_t name_hash = fnv1a_hash(cellmap_name); // cellmap 名のハッシュ値を取得
    
    if (override_texture != nullptr) {
        // 1. 指定されたテクスチャでマップを上書き
        textures_map[name_hash] = override_texture;
    } else {
        // 2. 解除時の処理: SSAB 内の定義からオリジナル画像を再ロードして復元
        auto cellmaps = ssab_binary->cellmaps();
        for (uint32_t i = 0; i < cellmaps->size(); i++) {
            auto cellmap = cellmaps->Get(i);
            if (cellmap->name_hash() == name_hash) {
                // 相対パス解決を行い、オリジナルのテクスチャを再ロード
                std::string original_path = resolve_path(cellmap->image_path()->c_str());
                textures_map[name_hash] = Engine::load_texture(original_path);
                break;
            }
        }
    }
}
```

### 外部 SSAB 参照 (External Instance)
インスタンスパーツが「別の SSAB ファイル」にあるアニメーションを参照する機能です。
- **実装方法**: 
    1. 親の SSAB の `external_instances()` セクションを走査し、参照されている外部 SSAB 名をリストアップします。
    2. それらのファイルをロードし、個別の `Resource` オブジェクトとして管理します。
    3. インスタンスパーツのセットアップ時、`ref_anime_hash` をキーに全ロード済み SSAB を検索して対象のアニメーションを見つけ出し、子 Context を初期化します。

## 2. シーケンス (Sequence) の実装

`.ssqb` ファイルに定義された「アニメーションの連結再生リスト」を処理する機能です。`libssruntime` 自体は単一のアニメーション再生に特化しているため、シーケンスの管理は Player 側のロジックで行います。

- **再生フロー**:
    1. `ss_runtime_is_end_frame_reached` で現在のアニメーションの終了を検知します。
    2. シーケンスリストの次の項目を確認します。
    3. `ss_runtime_setup_animation` で次のアニメーションをセットアップし、再生を継続します。
- **ループ設定**: シーケンス全体のループや、特定の項目のみのループなど、Player 側でステートマシンを構築して制御します。

## 3. 特殊なパーツタイプへの対応

| パーツ種別 | 実装のヒント |
| :--- | :--- |
| **9 スライス** | SoA の `local_vertices` には既に 9 スライス計算済みの座標が入っています。通常のパーツと同様に描画するだけで対応可能です。 |
| **シェイプ** | `ss_runtime_get_shape_vertices` 等の専用 SoA バッファを使用して、多角形の頂点データを取得します。 |
| **サウンド** | 更新ループ内で `passed_event` を監視し、`audios` イベントが含まれていれば、指定されたサウンド ID をエンジンのオーディオプレイヤーで再生します。 |

## 4. エディタ拡張とワークフローの統合

開発効率向上のため、ゲームエンジンのエディタ拡張として `ssconverter` を統合することを強く推奨します。

- **Godot (C++) の例**:
    - `EditorImportPlugin`: `.sspj` を検知して自動的に `ssconverter` を呼び出し、内部的な `.ssab` リソースに変換します。
    - `EditorInspectorPlugin`: インスペクタ上でアニメーションのプレビューや、テクスチャオーバーライドの確認ができるようにします。
- **ワークフロー**: デザイナーが SpriteStudio で保存した瞬間に、ゲームエディタ側で最新のアニメーションが即座に反映される環境（ホットリロード）を構築することが理想です。

---
[目次に戻る](./README.ja.md)
