# 実装フェーズ 5：パーツカラーと特殊アトリビュート (Parts Color & Attributes)

パーツの見た目をカスタマイズするためのカラー合成や、UV アニメーションなどの特殊なアトリビュートの実装ガイドです。

## 1. パーツカラー (Parts Color)

頂点ごとにカラーを合成し、着色やグラデーションを実現します。

### カラーの取得
`ss_runtime_get_parts_color` 等の API は用意されていません。`FrameData` の `parts_color` テーブルから直接取得するか、最適化された Player では Brain が `local_vertices` と共に提供する色情報（将来拡張）を使用します。

```cpp
// パーツカラーの適用例
int color_idx = part_state->part_color();
if (color_idx >= 0) {
    auto pc = frame_data->parts_color()->Get(color_idx);
    
    // a チャンネルには階層 alpha が事前合成済み。
    // Player 側で state->alpha() を再乗算する必要はありません。
    Color clt = ConvertColor(pc->lt()->rgba()); // 左上
    Color crt = ConvertColor(pc->rt()->rgba()); // 右上
    Color clb = ConvertColor(pc->lb()->rgba()); // 左下
    Color crb = ConvertColor(pc->rb()->rgba()); // 右下
    
    // カラー合成モード (Mix, Mul, Add, Sub)
    ApplyColorBlend(pc->blend_type());
}
```

## 2. 合成モード (Blend Mode)

パーツ全体の描画に対するブレンド設定です。

- **Mix (通常)**: `SRC_ALPHA / ONE_MINUS_SRC_ALPHA`
- **Add (加算)**: `SRC_ALPHA / ONE` (または `ONE / ONE`)
- **Sub (減算)**: `SRC_ALPHA / ONE` (ブレンド演算を SUB に設定)
- **Mul (乗算)**: `DST_COLOR / ONE_MINUS_SRC_ALPHA` (または `ZERO / SRC_COLOR`)

## 3. UV アニメーション

テクスチャの切り出し範囲（UV）を移動・回転・拡大縮小させる機能です。
Brain 側で計算済みの `local_uvs` (SoA) を使用することで、Player は単にテクスチャ座標として渡すだけで済みます。

## 4. 左右・上下反転 (Flip)

パーツの反転 (`flip_h`, `flip_v`) やセルの反転 (`img_flip_h`, `img_flip_v`) は、すべて **`local_vertices` および `local_uvs` に反映済み** です。Player 側で座標を反転させる追加ロジックは不要です。

---
[目次に戻る](./README.ja.md)
