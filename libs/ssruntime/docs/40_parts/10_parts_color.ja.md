> **日本語** | [English](./10_parts_color.md)

# パーツカラー

パーツカラーは、パーツの頂点に色を持たせ、テクスチャの色と合成して着色やグラデーションを作る機能です。Brain は色と合成の種類を FrameData で渡し、合成の計算は Player がシェーダで行います。

## 1. 2 つのブレンドを区別する

「ブレンド」と呼ばれるものが 2 つあります。混同すると実装を誤るので、先に区別してください。

| | **パーツカラーの合成**（このページ） | **描画先との合成** |
|---|---|---|
| 何を混ぜるか | テクスチャの色とパーツカラー | 描いた結果と、描画先（画面）の色 |
| どこで計算するか | **シェーダの中**（§6） | **GPU のブレンドの段階** |
| 値の出どころ | FrameData の `parts_color` の **`blend_type`** | **`DrawBatch.blend_type`**（ssab の `PartData.blend_type`） |
| 有効な種類 | **Mix / Mul / Add / Sub の 4 種類** | 12 種類すべて |

このページで扱うのは、パーツカラーの合成です。

## 2. 合成の種類

| 値 | 種類 | 計算 |
| :--- | :--- | :--- |
| `0` | Mix（通常） | テクスチャの色とパーツカラーを `rate` で線形補間する |
| `1` | Mul（乗算） | テクスチャの色にパーツカラーを掛ける。強さは `rate` |
| `2` | Add（加算） | テクスチャの色にパーツカラーの `rate` 倍を足す |
| `3` | Sub（減算） | テクスチャの色からパーツカラーの `rate` 倍を引く |

`blend_type` の型は 12 種類を定義する `BlendType` ですが、パーツカラーの合成で有効なのは上の 4 つだけです。`4` 以上の値が来た場合は `Mix`（`0`）として扱ってください。

## 3. パーツカラーを読む

パーツカラーを返す C-API はありません。FrameData の `parts_color` から読みます。

- パーツの `PartState.part_color` が `parts_color` の添字です。`-1` ならパーツカラーはありません（§5）。
- 1 つのパーツカラーは、合成の種類 `blend_type` と、4 隅（`lt` / `rt` / `lb` / `rb`）の色を持ちます。各隅は色 `rgba` と合成の強さ `rate` を持ちます。
- **`rgba.a` には、パーツの不透明度（親から継承したものを含む）が掛けてあります。** Player は `PartState.alpha` を掛け直さないでください。
- `rgba` は `U8Color` で、メモリ上の並びは `a, r, g, b`（各 1 byte）です。構造体を直接読むとき（[描画ループ §4-2](../30_core/40_render_loop.ja.md#4-2-framedata-のベクタを直接読む)）は、この順で読んでください。

## 4. パーツの種類ごとの規則

パーツの種類によって、パーツカラーの使い方が違います。この規則は Player が適用します（ssab のデータを劣化させず、シェーダの種類を増やさないため）。

| パーツの種類 | 合成の種類 | 色 | `rate` |
|---|---|---|---|
| 通常 | `blend_type` のとおり | 4 隅それぞれ | 4 隅それぞれ |
| メッシュ | `blend_type` のとおり | 全頂点に同じ値（`lt` の値） | 全頂点に同じ値 |
| シェイプ | 合成しない（Mix・`rate = 1.0` 相当） | 4 隅それぞれ | 常に `1.0` |

### 4-1. 通常のパーツ

```cpp
int color_idx = part_state->part_color();
if (color_idx >= 0) {
    auto pc = frame_data->parts_color()->Get(color_idx);
    set_corner_color(LT, pc->lt()->rgba(), pc->lt()->rate());
    set_corner_color(RT, pc->rt()->rgba(), pc->rt()->rate());
    set_corner_color(LB, pc->lb()->rgba(), pc->lb()->rate());
    set_corner_color(RB, pc->rb()->rgba(), pc->rb()->rate());
    set_blend(pc->blend_type() <= 3 ? pc->blend_type() : 0);
}
```

`set_corner_color` / `set_blend` は、描画用の頂点に値を書く処理を表す、実在しない関数です。

**中心の頂点**: 頂点の変形やパーツカラーのある通常のパーツを、4 隅と中心の 5 頂点（三角形のファン）で描く場合は、中心の頂点の色と `rate` に 4 隅の平均を与えてください。位置と UV は、ローカル頂点とローカル UV の 5 番目の値を使います。4 隅だけで描くと、変形や着色の見た目が変わります。

### 4-2. メッシュ

メッシュのパーツカラーは、全頂点に同じ値です。`lt` の色と `rate` をすべての頂点に与えます。全頂点に同じ値をコピーするのは冗長なので、メッシュ用のシェーダで 1 つの値（uniform）として渡してもかまいません。

### 4-3. シェイプ

シェイプはパーツカラーを合成せず、頂点の色に `rgba` をそのまま使います。§6 のシェーダを使う場合は、`rate = 1.0`・Mix として白いテクスチャと合成させれば、結果が `rgba` になります。`blend_type == Mix` のときの `rgba.a` の調整などは、コンバータが済ませています。Player がアルファを補正する必要はありません。

シェイプの内側の頂点の色は、4 隅の色をシェイプの外接矩形の中の位置で補間して求めます（[9 スライス・シェイプ・テキスト §2](./30_special_parts.ja.md#2-シェイプ)）。

## 5. パーツカラーが無いパーツ

パーツカラーの無いパーツ（`part_color == -1`）も、同じシェーダで描けます。**色 = 白（アルファはパーツの不透明度）、`rate = 0`** を与えてください。`rate = 0` では、§6 の式が合成の種類によらずテクスチャの色をそのまま返すので、パーツカラーの有無で描き方を分けずに済みます。

パーツカラーの有無はフレームごとに変わることがあります（カラーオーバーライドを設定したときなど）。描き方を分けていなければ、何もする必要はありません。

## 6. シェーダの例

パーツカラーの合成は、シェーダで計算するのが合理的です。ここでは、4 種類の合成を**分岐なしで**計算する方法を示します。

頂点シェーダで `blend_type` から 4 つの係数（`varg`）を作り、フラグメントシェーダでは次の 1 つの式で計算します。

```
合成後の色 = テクスチャの色 * varg.x + mix(白, テクスチャの色, varg.z) * パーツカラー * varg.y
```

例は **Godot のシェーダ言語**で書いています。他のエンジンでは次のように読み替えてください。

| Godot | 一般的な対応 |
|---|---|
| `CUSTOM0` | 頂点ごとの任意の 4 成分の属性。ここでは `(rate, blend_type, pma_flag, 未使用)` を入れる（Unity では追加の頂点ストリームや `TEXCOORDn`、D3D では任意のセマンティクス） |
| `COLOR`（頂点シェーダ） | 頂点の色（= パーツカラーの `rgba`） |
| `TEXTURE` / `UV` | サンプラーと UV |
| `vertex()` / `fragment()` | 頂点シェーダ / フラグメントシェーダの入り口 |
| `varying` | 頂点シェーダからフラグメントシェーダへの補間値（GLSL の `out` / `in`、HLSL の補間子） |

### 6-1. 係数の表

```glsl
const vec4 _PartColorCoef[4] = vec4[4](
    vec4(-1.0,  1.0, 0.0, 0.0),  // [0] Mix
    vec4(-1.0,  1.0, 1.0, 0.0),  // [1] Mul（.z = 1 で、白の代わりにテクスチャの色を掛ける）
    vec4( 0.0,  1.0, 0.0, 0.0),  // [2] Add
    vec4( 0.0, -1.0, 0.0, 0.0)   // [3] Sub
);

varying vec4 partcolor_varg;
varying vec4 partcolor_color;
```

係数の `x` / `y` / `z` は、それぞれ「テクスチャの色の重みの調整」「パーツカラーの重み」「Mul の印」です。`w` は使いません。

### 6-2. 頂点シェーダ

```glsl
void vertex() {
    int b = int(CUSTOM0.y + 0.5);
    int idx = (b >= 0 && b <= 3) ? b : 0;   // 0..3 の外は Mix（§2）
    vec4 c = _PartColorCoef[idx];
    float rate = CUSTOM0.x;
    partcolor_varg = vec4(
        1.0 + c.x * rate,                            // x: テクスチャの色の重み
        c.y * rate,                                  // y: パーツカラーの重み
        c.z,                                         // z: Mul の印
        CUSTOM0.z                                    // w: PMA の印をそのまま渡す
    );
    partcolor_color = COLOR;                         // パーツカラーの rgba
}
```

### 6-3. フラグメントシェーダ

```glsl
vec3 ss_partcolor_blend(vec3 pixel_rgb, vec3 color_rgb, vec4 varg) {
    return pixel_rgb * varg.x + mix(vec3(1.0), pixel_rgb, varg.z) * color_rgb * varg.y;
}

void fragment() {
    vec4 p = ss_input_texture(TEXTURE, UV);
    vec3 rgb = ss_partcolor_blend(p.rgb, partcolor_color.rgb, partcolor_varg);
    float a  = p.a * partcolor_color.a;              // パーツカラーのアルファには不透明度が掛けてある
    COLOR    = ss_output_color(vec4(rgb, a), partcolor_varg.w);
}
```

`ss_input_texture()` と `ss_output_color()` は、テクスチャを読むときと色を出すときの色の変換（PMA の解除と適用、リニアや HDR への変換など）を 1 か所にまとめるための関数です。PMA を扱わない最小の実装では、それぞれ `texture(tex, uv)` と、受け取った色をそのまま返す関数でかまいません。

---
[目次に戻る](../README.ja.md)
