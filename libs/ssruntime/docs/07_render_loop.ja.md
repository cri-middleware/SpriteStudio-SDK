# 実装フェーズ 3：レンダリングループとバッチ描画 (Rendering & Batching)

`libssruntime` (Brain) が算出した計算結果を効率的に取得し、ゲームエンジンで描画するための「レンダリング契約」について記載します。

## 1. SoA バッファによる高速なデータ取得

現代的な Player 実装では、パーツごとに計算結果を取得するのではなく、**SoA (Structure of Arrays) 形式で一括取得** する方式が標準です。

`ss_runtime_get_world_matrices(ctx, &ptr, &len)` などの API を使用することで、現在のフレームの全パーツの計算済みデータをポインタ経由で直接参照できます。

| 取得できるデータ | API 名 | 用途 |
| :--- | :--- | :--- |
| **ワールド行列** | `ss_runtime_get_world_matrices` | 各パーツの最終的な位置・回転・拡大縮小（4x4 行列） |
| **ローカル頂点** | `ss_runtime_get_local_vertices` | Pivot 反転等が適用済みの頂点座標（5頂点 x 2軸） |
| **テクスチャハッシュ** | `ss_runtime_get_cell_texture_hashes` | パーツが使用しているテクスチャの識別子 |
| **UV / セルメタ** | `ss_runtime_get_local_uvs`, `cell_meta` | アニメーション済みの UV 座標やセルサイズ |

## 2. バッチ描画契約 (The Batching Contract)

ドローコールを最小化するため、Brain は **`DrawBatch`** という単位で描画計画を提示します。Player は「FlatBuffers で構造を把握し、SoA でデータを引く」というハイブリッドな方式でループを回します。

```cpp
// 1. FrameData (FlatBuffers) を取得
uint8_t* fd_ptr; size_t fd_len;
ss_runtime_get_frame_data(ctx, frame_no, &fd_ptr, &fd_len);
auto frame_data = ss::runtime::GetFrameData(fd_ptr);

// 2. 構造情報の参照
auto batches = frame_data->draw_batches();
auto draw_order = frame_data->draw_order();

// 3. SoA バッファ (ポインタ) の参照
const float *world_matrices, *local_vertices;
size_t m_len, v_len;
ss_runtime_get_world_matrices(ctx, &world_matrices, &m_len);
ss_runtime_get_local_vertices(ctx, &local_vertices, &v_len);

// 4. 描画ループ
for (auto batch : *batches) {
    ApplyMaterial(batch->texture_hash(), batch->blend_type());
    
    for (int k = 0; k < batch->count(); k++) {
        int p_idx = draw_order->Get(batch->start_rank() + k);
        
        // SoA バッファから対象パーツのデータを直接参照
        const float* m = &world_matrices[p_idx * 16];
        const float* v = &local_vertices[p_idx * 10];
        EmitGeometry(m, v);
    }
    FlushDrawCall();
}
```

### RenderMode についての注意点

`libssruntime` には、エンジンの描画思想に合わせた `RenderMode`（レンダリングモード）の概念があります。
初期化時に `ss_runtime_set_render_mode` で指定するモードによって、取得できる `FrameData` (FlatBuffers) の内容が変化します。

- **`RenderMode::Direct` (推奨)**: Godot の `RenderingServer` のように、C-API (SoAバッファ) 経由でデータを一括取得して直接描画するアプローチに最適化されたモードです。このモードでは FlatBuffers への冗長なデータ（`vertices`, `cells`, `parts_color`, `shaders`, `deforms`）のシリアライズがスキップされ、CPU負荷とアロケーションが大幅に削減されます。上記のサンプルコードはこちらのモードを前提としています。
- **`RenderMode::SceneGraph`**: 各パーツごとにゲームエンジンのノード（Node2D や GameObject など）を生成し、階層構造（シーングラフ）を組んで描画をエンジン任せにするアプローチ用です。このモードでは FlatBuffers 内の `vertices` などの詳細データが全て出力されるため、それらを読み出してノードのプロパティを更新する用途に使います。

（※ Player 実装で `FrameData` から頂点やセルの情報を取得しようとして `null` が返ってくる場合は、`ss_runtime_set_render_mode` で `RenderMode_SceneGraph` を設定しているか確認してください。）

## 3. 実装のポイント：Godot / RenderingServer

Godot などのモダンなエンジンでは、大量の `Sprite2D` ノードを作るのではなく、`RenderingServer` を用いたダイレクト描画が最も効率的です。

1.  **CanvasItem のプール**: バッチごとに `RID` (CanvasItem) を使い回します。
2.  **頂点データの集約**: 各パーツの頂点を `PackedVector2Array` 等に詰め込み、`canvas_item_add_triangle_array` で一括描画します。
3.  **Z-index の活用**: `canvas_item_set_z_index` を用いて、バッチ間の描画順を制御します。

## 4. マスクの実装

マスクパーツが出現すると、`DrawBatchKind::Mask` が発行されます。
Player はこれをフックにして、エンジンの Stencil Buffer や専用のマスタリング機能を開始・更新します。

---
[目次に戻る](./README.ja.md)
