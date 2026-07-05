> **日本語** | [English](./70_external_resources.md)

# 外部リソースとテクスチャの管理

大規模なプロジェクトや動的な演出で必要となる、外部リソースの差し替え・参照の実装ガイドです。

## 外部テクスチャのオーバーライド (Texture Override)
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

## 外部 SSAB 参照 (External Instance)
インスタンスパーツが「別の SSAB ファイル」にあるアニメーションを参照する機能です（[インスタンスパーツ](./10_instance.ja.md) と併せて参照）。
- **実装方法**: 
    1. 親の SSAB の `external_instances()` セクションを走査し、参照されている外部 SSAB 名をリストアップします。
    2. それらのファイルをロードし、個別の `Resource` オブジェクトとして管理します。
    3. インスタンスパーツのセットアップ時、`ref_anime_hash` をキーに全ロード済み SSAB を検索して対象のアニメーションを見つけ出し、子 Context を初期化します。

---
[目次に戻る](../README.ja.md)
