# 実装フェーズ 5：パーツカラー (Parts Color)

パーツの見た目をカスタマイズするためのカラー合成の実装ガイドです。

---

## 1. パーツカラー (Parts Color)

頂点ごとにカラーを合成し、着色やグラデーションを実現します。

### 1-0.「2 つのブレンド」を最初に区別する（重要）

本章で「ブレンド」と呼ぶものは **別軸の 2 つ** があります。混同すると実装を誤るので先に切り分けます。

| | **PartColor ブレンド**（本章の主題） | **フレームバッファ合成** |
|---|---|---|
| 何を混ぜるか | テクセル色 と パーツカラー（頂点色） | 描画結果 と 描画先（画面） |
| どこで計算するか | **シェーダ内**（§1-4 の式） | **GPU のブレンドステージ**（描画材質） |
| 由来フィールド | `parts_color` の **`blend_type`**（`0..3`） | パーツの **`alpha_blend_type`** |
| 有効な種類 | **Mix / Mul / Add / Sub の 4 種のみ** | 12 種すべて |

以降 §1-1〜§1-4 で扱うのは **PartColor ブレンド** です。フレームバッファ合成（`alpha_blend_type`）は別途 `07_render_loop` 系で扱う描画材質側の設定であり、混同しないでください。

### 1-1.PartColor ブレンドモード (Blend Mode)

パーツカラー（頂点色）をテクセル色と **シェーダ内で** 合成する方式です。有効値は次の 4 種です。

- **Mix (通常)**: テクセルとパーツ色を `rate` で線形補間
- **Add (加算)**: テクセルにパーツ色を `rate` 倍して加算
- **Sub (減算)**: テクセルからパーツ色を `rate` 倍して減算
- **Mul (乗算)**: テクセルとパーツ色を乗算（`rate` で強度調整）

具体的な合成式は §1-4 の参考シェーダを参照してください。

注釈:  
`parts_color` の `blend_type` フィールドは `BlendType` 列挙型のため、上記 4 種に加えて合計 12 種（Mulalpha / Screen / Exclusion / Invert / Mul2 / Div2 / Screen2 / Overlay2）の値が定義されていますが、 **PartColor の合成モードとして有効なのは上記の 4 種のみ** です。`4` 以上が来た場合は防御的に `Mix`（`0`）へクランプしてください。

### 1-2.カラーの取得
`ss_runtime_get_parts_color` 等の API は用意されていません。`FrameData` の `parts_color` テーブルから直接取得します。  

また、描画するパーツの種別で、それぞれ特殊なルールが存在しています。  
本ルールの適用は、プレイヤー側で行なう必要があります。  

下表の「PartColor ブレンド」列は §1-0 の **PartColor ブレンド**（テクセル式・4 種・シェーダ側）を指し、`alpha_blend_type`（フレームバッファ合成）とは別軸です。

| パーツ種別 | PartColor ブレンド | 適用色 | rate の扱い |
|---|---|---|---|
| Normal | あり（`blend_type` を使用） | 4隅の頂点で個別 | 4隅の頂点で個別 |
| Mesh | あり（`blend_type` を使用） | 全頂点同値（Whole-only） | 全頂点同値 |
| Shape | Mix に強制（式をバイパス） | 4隅の頂点で個別 | 常に1.0 に強制 |

このルールの適用をプレイヤー側の責務としている理由については、ssabのデータ上でなるだけ劣化させないためと、シェーダが個別化して増えることを抑制するための 2 つの意図があります。  

#### 1-2-1.「通常（Normal）」パーツの例
```cpp
// Normalパーツのパーツカラーの適用例（参考実装）
int color_idx = part_state->part_color();
if (color_idx >= 0) {
    auto pc = frame_data->parts_color()->Get(color_idx);
    
    // a チャンネルには階層 alpha が事前合成済み。
    // プレイヤー 側で state->alpha() を再乗算する必要はありません。
    ConvertColor(pc->lt()->rgba(), pc->lt()->rate()); // 左上
    ConvertColor(pc->rt()->rgba(), pc->rt()->rate()); // 右上
    ConvertColor(pc->lb()->rgba(), pc->lb()->rate()); // 左下
    ConvertColor(pc->rb()->rgba(), pc->rb()->rate()); // 右下
    
    // カラー合成モード (Mix, Mul, Add, Sub)
    ApplyColorBlend(pc->blend_type());
}
```
注釈: ConvertColor関数とApplyColorBlend関数は非実在の疑似関数で、描画用頂点データに値を設定するための処理となります。

> **センター頂点（5 頂点目）について**: プレーヤーは、頂点変形またはパーツカラーが有効な通常スプライトを **4 隅 ＋ 中心の 5 頂点（三角形ファン）** で描画する場合があります。その際は中心頂点の **色・`rate` に 4 隅の平均値**（位置・UV も中心へ補間）を与えてください。4 隅のみを出力すると、変形・着色されたスプライトの見た目が変わります。

#### 1-2-2.「メッシュ（Mesh）」パーツの例
```cpp
// Meshパーツのパーツカラーの適用例（参考実装）
int color_idx = part_state->part_color();
if (color_idx >= 0) {
    auto pc = frame_data->parts_color()->Get(color_idx);
    auto cl = pc->lt()->rgba();      // メッシュは Whole-only なので、lt の値を全頂点に流す
    float rt = pc->lt()->rate();

    // メッシュの全頂点を処理
    for (int i = 0; i < vert_count; i++) {
        // a チャンネルには階層 alpha が事前合成済み。
        // プレイヤー 側で state->alpha() を再乗算する必要はありません。
        ConvertColorMesh(cl, rt);   // 全頂点に同じ色 & rate（左上）

        // カラー合成モード (Mix, Mul, Add, Sub)
        ApplyColorBlendMesh(pc->blend_type());
    }
}
```
注釈: ConvertColorMesh関数とApplyColorBlendMesh関数は非実在の疑似関数で、描画用頂点データに値を設定するための処理となります。  

> **メッシュは Whole-only**: メッシュのパーツカラーは全頂点同値（4 隅個別ではない）です。全頂点へ同じ色・`rate` をコピーする実装で正しく動きますが、これは冗長でもあるため、メッシュ専用シェーダで `uniform` 化して 1 値で渡す最適化も可能です。

#### 1-2-3.「シェイプ（Shape）」パーツの例
 ```cpp
 // Shapeパーツのパーツカラー適用例（参考実装）
 int color_idx = part_state->part_color();
 if (color_idx >= 0) {
     auto pc = frame_data->parts_color()->Get(color_idx);
     
     // a チャンネルには階層 alpha が事前合成済み。
     // プレイヤー 側で state->alpha() を再乗算する必要はありません。
     ConvertColorShape(pc->lt()->rgba(), 1.0f); // 左上（rate は強制で 1.0）
     ConvertColorShape(pc->rt()->rgba(), 1.0f); // 右上（同上）
     ConvertColorShape(pc->lb()->rgba(), 1.0f); // 左下（同上）
     ConvertColorShape(pc->rb()->rgba(), 1.0f); // 右下（同上）
     
     // カラー合成モード (Mixに強制)
     ApplyColorBlendShape(0);	// Mixに固定
 }
 ```
注釈: ConvertColorShape関数とApplyColorBlendShape関数は非実在の疑似関数で、描画用頂点データに値を設定するための処理となります。  

> **シェイプは合成式をバイパス**: シェイプは PartColor の合成式（§1-4）を通さず、頂点色 = `pc.rgba` を直接使います。シェーダ上は `rate = 1.0` ＋ `Mix` 固定で `ss_partcolor_blend(白, pc.rgb, …)` が `pc.rgb` に収束するように渡すのが簡単です。なお `blend_type == Mix` のときの `rgba.a` 正規化など SS6/SS7 の調停は **コンバータ側で適用済み**なので、Player 側で追加の α 補正は不要です。

#### 1-2-4.パーツカラー無し（`color_idx < 0`）の既定値

パーツカラーを持たないパーツも、描画は同じ汎用シェーダ式（§1-4）を通ります。既定値として **頂点色 = 白（α は階層 α）／ `rate = 0`** として扱うことを推奨します。  
`rate = 0` のときの合成式の結果は **テクセル素通り**（`blend_idx` に依らず `pixel_rgb` をそのまま返す）になるので、パーツカラーの有無で描画経路を分けずに統一できます。

### 1-3.参考:パーツカラーに含まれるデータについて

パーツカラーには下記の内容が含まれています。  
- blend_type : パーツカラーの各色をテクセル色と合成する方法
- （各頂点色の）rgba : 合成する色。a はパーツカラーのアルファ素材（パーツの不透明度と乗算される）
- （各頂点色の）rate : テクセル色と合成する時の比率（合成強度）

上記の内、rgba の a については（実装例などにある通り）ssruntimeの処理内で自動的にパーツの不透明度と合算されるため、基本的なプレイヤーの実装では特段留意する必要はありません。

### 1-4.参考:シェーダでのパーツカラーの適用例（テーブル駆動の参考実装）

パーツカラーの合成は、基本的に（頂点及びフラグメントの）シェーダで計算を行うのが合理的です。  
下記に、パーツカラーの色の合成を、合成式 4 種（Mix / Mul / Add / Sub）を **分岐なし** で実装するパターンを参考実装として提示します。  

概略:  
頂点シェーダで `blend_type` から、4 要素の `varg` ベクタ（係数テーブル）を組み立て、フラグメントシェーダでは 1 本の汎用式に渡して評価します。  
フラグメントシェーダでは、合成の種類を気にせず、下記の式でテクセル色との合成を行います。  
「合成後の色 = ( ピクセルのRGB * varg.x ) + mix( 純白のRGB, ピクセルのRGB, varg.z) * パーツカラー色のRGB * varg.y;」

#### 1-4-1.頂点シェーダの実装例（係数テーブルの定数定義）
```glsl
const vec4 _PartColorCoef[4] = vec4[4](
    vec4(-1.0,  1.0, 0.0, 0.0),  // [0] Mix
    vec4(-1.0,  1.0, 1.0, 0.0),  // [1] Mul（.z=1 で「白の代わりにピクセル色を係数化」）
    vec4( 0.0,  1.0, 0.0, 0.0),  // [2] Add
    vec4( 0.0, -1.0, 0.0, 0.0)   // [3] Sub
);

varying vec4 partcolor_varg;
varying vec4 partcolor_color;
```

`(c.x, c.y, c.z)` は順に「ピクセル側の重み調整」「PartColor 側の重み係数」「Mul モードフラグ」を表します（c.wは現状未使用）。

#### 1-4-2.頂点シェーダの実装例（`varg` の組み立て）
`CUSTOM0` には プレイヤー が `(rate, blend_idx, pma_flag, _)` を頂点ごとに積んでいます（`rate` の意味は §1-3 を参照）。

```glsl
void vertex() {
    int idx = clamp(int(CUSTOM0.y + 0.5), 0, 3);  // blend_idx を 0..3 に保護
    vec4 c = _PartColorCoef[idx];
    float rate = CUSTOM0.x;                        // colorA
    partcolor_varg = vec4(
        1.0 + c.x * rate,                          // .x: ピクセル側重み
        c.y * rate,                                // .y: PartColor 側重み
        c.z,                                       // .z: Mul モードフラグ
        CUSTOM0.z                                  // .w: pma フラグを passthrough
    );
    partcolor_color = COLOR;                       // 頂点色（= PartColor の rgba）
}
```

#### 1-4-3.フラグメントシェーダの実装例（汎用式 1 本で評価）
```glsl
vec3 ss_partcolor_blend(vec3 pixel_rgb, vec3 color_rgb, vec4 varg) {
    return pixel_rgb * varg.x + mix(vec3(1.0), pixel_rgb, varg.z) * color_rgb * varg.y;
}

void fragment() {
    vec4 p = ss_input_texture(TEXTURE, UV);
    vec3 rgb = ss_partcolor_blend(p.rgb, partcolor_color.rgb, partcolor_varg);
    float a  = p.a * partcolor_color.a;            // テクスチャ α × 頂点 α（rateAlpha に階層 α が事前合成済み）
    COLOR    = ss_output_color(vec4(rgb, a), partcolor_varg.w);
}
```

> **`ss_input_texture()` / `ss_output_color()` について**: これらは入出力段の **色変換の集約点** として用意したフック関数です。逆 PMA・PMA・Linear/HDR 変換などのテクスチャ入力／最終出力の色処理を 1 箇所にまとめる設計で、PMA を扱わない最小実装ではそれぞれ素の `texture(tex, uv)` / 受け取った色をそのまま返す実装で構いません（`partcolor_varg.w` の PMA フラグは passthrough）。

---

## 2. 実装チェックリスト

- [ ] **2 つのブレンドを混同していない**（§1-0）: PartColor ブレンド（`blend_type`・シェーダ式・4 種）と `alpha_blend_type`（フレームバッファ・12 種）を別軸として扱った。
- [ ] **インデックス取得**（§1-2）: `part_color()`（FrameData、毎フレーム）を読み、`< 0` なら PartColor 無し。
- [ ] **PartColor 無しの既定値**（§1-2-4）: 頂点色 = 白（α は階層 α）・`rate = 0` を与え、描画経路を分けずに統一した（`rate=0` で式がテクセル素通りに収束）。
- [ ] **パーツ種別ごとのルール**（§1-2）: Normal=4 隅個別 / Mesh=全頂点同値（Whole-only）/ Shape=`rate` 強制 1.0・式バイパス。
- [ ] **`blend_type` のクランプ**（§1-1）: `0..3` 以外は `Mix`（`0`）へ。
- [ ] **α を二重適用していない**（§1-3）: `rgba.a` には階層 α が **事前合成済み**。Player 側で `state->alpha()` を再乗算しない。
- [ ] **センター頂点**（§1-2-1）: 4 隅＋中心の 5 頂点で描く場合、中心の色・`rate`・位置・UV に 4 隅の平均（補間）を与えた。

---
[目次に戻る](./README.ja.md)
