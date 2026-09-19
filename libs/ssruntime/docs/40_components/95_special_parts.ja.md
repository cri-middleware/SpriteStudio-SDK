> **日本語** | [English](./95_special_parts.md)

# 特殊なパーツタイプへの対応

9 スライス・シェイプ・テキスト・サウンドといった特殊なパーツ種別の実装ヒントです。

| パーツ種別 | 実装のヒント |
| :--- | :--- |
| **9 スライス** | `DrawBatchKind::Nines` のバッチを、後述の専用 SoA バッファから描画します。このパーツの `local_vertices` には外接するただの矩形しか入っていません。 |
| **シェイプ** | `ss_runtime_get_shape_vertices` 等の専用 SoA バッファを使用して、多角形の頂点データを取得します。 |
| **テキスト** | グリフは Player の担当です。ランタイムが用意するのはパーツの位置だけ（1 パーツにつきジオメトリ無しの `DrawBatchKind::Text` バッチ）で、他のパーツと同じ優先度順で合成できます。ssab の読み方とグリフのレイアウト仕様は [テキスト](#テキスト) を参照してください。 |
| **サウンド** | 更新ループ内で `passed_event` を監視し、`audios` イベントが含まれていれば再生します（詳細は [オーディオ（サウンド）再生の統合](./90_audio.ja.md) を参照）。 |

## 9 スライス

マージン・塗りモード・繰り返し数を含むグリッド生成はランタイム側で完結しているため、Player は頂点を転送するだけです。`kind` が `Nines` のバッチが指すパーツのジオメトリは、次のバッファに入っています。

| バッファ | 内容 |
| :--- | :--- |
| `ss_runtime_get_nines_vertices` | パーツローカル座標の `[x, y, ...]` インターリーブ。原点（pivot）・イメージ反転・Y-Up/Y-Down 設定は適用済みなので、`local_vertices` と同じくワールド行列を掛けるだけです。 |
| `ss_runtime_get_nines_uvs` | セルマップのテクスチャピクセル空間の `[u, v, ...]` インターリーブ。`local_uvs` と同じく、正規化するにはテクスチャサイズで割ります。 |
| `ss_runtime_get_nines_box_coords` | `[fx, fy, ...]` インターリーブ。パーツの左端・上端が `0`、右端・下端が `1` です。`shape_vertex_box_coords` と同じ要領で、`part_color` の 4 隅をバイリニア補間するのに使います。 |
| `ss_runtime_get_nines_indices` | 三角形インデックス。値は**パーツごとの頂点範囲内のローカル値**で、`LT, RT, LB / LB, RT, RB` の順に並びます。 |
| `ss_runtime_get_nines_vertex_offsets` | 長さ `num_parts + 1` の CSR 前置和。上記 3 つの頂点バッファをパーツ単位に切り出します。インターリーブされた float を指すには 2 倍してください。 |
| `ss_runtime_get_nines_index_offsets` | 長さ `num_parts + 1` の CSR 前置和。インデックスバッファをパーツ単位に切り出します。 |

つまりパーツ `p` は頂点 `[vertex_offsets[p], vertex_offsets[p + 1])`、インデックス `[index_offsets[p], index_offsets[p + 1])` を占め、それ以外の種別のパーツは空範囲になります。`DrawBatch.vertex_count` / `index_count` にはバッチに含まれるパーツの合計が入るのでバッファは事前確保できますが、16 bit のため 65535 で飽和します。タイルが大量に並ぶパーツでは CSR オフセットから実数を取ってください。

頂点数はオーサリングの塗りモードで変わります。**引き伸ばし**（`nines_fill_mode == 0`）は 4x4 のグリッドを共有して 16 頂点・54 インデックス固定、**タイル**（`== 1`）は繰り返し 1 枚ごとに矩形を出すため、パーツのサイズに応じて増減します。`PartTypeNines` に格納されたマージンは元セルのピクセル単位で、重なる場合はランタイム側で丸め込むため、Player 側での検証は不要です。

> **9 スライスパーツに適用されないもの:** UV アニメ属性（`UVTX` / `UVTY` / `UVRZ` / `UVSX` / `UVSY`）と、セルマップ内で回転配置されたセル。どちらもオーサリングツール側の挙動に合わせており、グリッドはセル矩形のみから生成されます。

## テキスト

テキストの描画は **Player の責務**です（[制限事項](../10_introduction/20_limitations.ja.md)）。ランタイムはグリフを組みも描きもせず、パーツに描画順の席を与えたうえで、オーサリングされた設定を ssab に載せて渡します。以降は、そこから**実際に文字を並べるまで**の仕様です。

### ランタイムが渡すもの

テキストパーツは **1 パーツにつき 1 つの `DrawBatch`** を占めます。

| `DrawBatch` のフィールド | 値 | 意味 |
| :--- | :--- | :--- |
| `kind` | `DrawBatchKind::Text` | テキストパーツであることの目印 |
| `count` | 常に `1` | 他のテキストパーツとマージされません |
| `vertex_count` / `index_count` | 常に `0` | ジオメトリは入っていません |
| `texture_hash` | 常に `0` | バインドすべきテクスチャは**バッチからは決まりません**（フォントは ssab から引きます） |
| `blend_type` | 通常どおり | グリフの合成に使ってください |

テキストパーツがマージされないのは、**ランタイムがそのパーツの描くフォントをキーにできない**ためです。2 つのテキストパーツを 1 バッチにまとめると、その中でフォントが変わったことを Player 側から見分けられなくなります。バッチが 1 パーツ固定であることに依存して構いません。

#### バッチから ssab のパーツを引く

```cpp
for (auto batch : *frame_data->draw_batches()) {
    if (batch->kind() != ss::runtime::DrawBatchKind::Text) continue;

    // count は 1 固定だが、ループの形は他の種別と揃えておく
    for (int k = 0; k < batch->count(); k++) {
        int slot       = frame_data->draw_order()->Get(batch->start_rank() + k);
        int part_index = frame_data->parts()->Get(slot)->part_index();

        auto part = ssab->parts()->Get(part_index);
        auto text = part->part_type_as_PartTypeText();   // 後述の設定はここ

        const float* m = &world_matrices[slot * 16];     // このパーツのワールド行列
        DrawGlyphs(text, m, frame_data->parts()->Get(slot));
    }
}
```

#### 前提として押さえておくこと

- **テキストパーツはセルを参照しません。** `PartState.cell` は `-1` で、`local_vertices` は 0 のままです。四角形すら来ないので、**ジオメトリは Player が全部作ります**。
- 位置・回転・スケール・不透明度・優先度は通常どおり `PartState` とワールド行列に入っています。グリフはパーツローカル座標で組み立て、他のパーツと同じようにワールド行列を掛けてください。
- **文字列はアニメーションしません。** ssab に入っているのはパーツ定義の静的な 1 本（`PartTypeText.text`）だけで、文字列を差し替えるキーフレーム属性はありません。
- **マスクとして作画されたテキスト**（`draw_as_mask`）は、そもそも draw plan から除外されます。カバレッジを書くグリフをランタイムが持たないためです（[Mask の実装](./40_mask.ja.md)）。

### ssab に入っているもの

#### `PartTypeText` — パーツの設定

| フィールド | 型 | 内容 |
| :--- | :--- | :--- |
| `text` | `string` | 表示する文字列（UTF-8）。改行は `\n` |
| `text_bitmap` | `bool` | `true` = ビットマップフォント（`text_char_map` を使う） / `false` = アウトラインフォント（`text_family` を使う） |
| `text_family` | `string` | アウトラインフォントのファミリ名（例 `Arial`）。**フォントファイルは ssab に入りません** |
| `text_char_map` | `string` | ビットマップフォントの charmap 名。`FontBitmap.name` と**同じ文字列**で、プロジェクト相対パス＋拡張子（例 `font/RoundedMPlus.fnt`）です |
| `text_size` | `int` | 1 行の高さ（px）。グリフの拡大率はここから決まります（後述の「拡大率」） |
| `text_space` | `float` | 字間の追加量（px）。**`text_size` でスケールされません** |
| `text_smooth` | `bool` | フォントページのテクスチャフィルタ。`true` = linear / `false` = nearest |
| `text_width` | `int` | レイアウト枠の幅（px）。`0` = 内容にあわせて自動 |
| `text_height` | `int` | レイアウト枠の高さ（px）。`0` = 内容にあわせて自動 |
| `e_anchor` | `TextAnchor` | 枠の中での行の寄せ方 |

`e_anchor` は**上位ニブルが水平、下位ニブルが垂直**です。

| | `0` | `1` | `2` |
| :--- | :--- | :--- | :--- |
| **上位ニブル**（`e_anchor >> 4`） | 左 | 中央 | 右 |
| **下位ニブル**（`e_anchor & 0x0F`） | 上 | 中央 | 下 |

つまり `LT = 0x00`, `CC = 0x11`, `RB = 0x22` です。`NO = 0xFF` は未指定で、左上（`LT`）として扱ってください。

#### `FontBitmap` / `FontGlyphBitmap` — ビットマップフォントのグリフ表

`ssab->font_bitmaps()` に入るのは、**`text_bitmap == true` のテキストパーツが実際に参照している charmap だけ**です。アウトラインフォントのパーツしか無いプロジェクトでは、このテーブルは空になります。

`PartTypeText.text_char_map` の文字列をそのまま `fnv1a` したものが `FontBitmap.name_hash` なので、パーツ → フォントはハッシュ 1 回で引けます。

| `FontBitmap` | 内容 |
| :--- | :--- |
| `name` / `name_hash` | charmap 名（＝ `text_char_map`）とそのハッシュ |
| `name_face` | 元フォントのフェイス名。表示用の情報で、描画には使いません |
| `base` | ベースライン位置（px、行の上端から） |
| `height_line` | charmap の 1 行の高さ（px）。**拡大率の分母**です |
| `size_texture_original` | ページ画像の元サイズ（px）。**`atlas` の正規化に使う値はこれ**で、実際にロードしたテクスチャのサイズではありません |
| `name_texture[]` | ページ画像のファイル名。**page id 順**に並んでいるので `name_texture[glyph.page]` で引けます |
| `name_texture_hash[]` | 上記のハッシュ（埋め込みモードでのアセットキー） |
| `table_data[]` | グリフ表。`id` 昇順にソート済みなので二分探索できます |

| `FontGlyphBitmap` | 内容 |
| :--- | :--- |
| `id` | 文字コード（UTF-16 コードユニット） |
| `atlas` | ページ画像内のグリフの領域（`x`, `y`, `w`, `h`） |
| `offset` | ペン位置からのグリフの描き出しオフセット（px、Y は下方向が正） |
| `width_advance` | 送り幅（px） |
| `text` | `id` を文字にしたもの。デバッグ表示用 |
| `page` | `name_texture[]` のインデックス |

ページ画像名は charmap ファイルからの相対をコンバータが解決済みで、**他の外部リソースと同じ「プロジェクト相対のファイル参照文字列」**として入っています。解決の規約は [データのロードと初期化](../30_core/10_initialization.ja.md) §3.0 を参照してください。

> [!IMPORTANT]
> **読んではいけない / 期待してはいけないフィールド**
>
> - `FontBitmap` の `size` / `padding_l` / `padding_r` / `padding_u` / `padding_d` / `scale` / `spacing`、および `FontGlyphBitmap.channel` は**固定値（`0` または `1.0`）**で、オーサリング内容を反映しません。レイアウトに使わないでください。字間は `PartTypeText.text_space`、行の高さは `height_line` から取ります。
> - **カーニングは ssab に入りません。** 送り幅は `width_advance` だけで決まります。
> - `FontGlyphBitmap.id` は 16 bit なので、**BMP 外（U+FFFF 超）の文字は扱えません**。絵文字や一部の漢字を使う企画では、ビットマップフォントではなくアウトラインフォントを前提にしてください。

### レイアウト仕様（ビットマップフォント）

座標系は **X が右、Y が下**（グリフの `offset` の向き）で計算し、最後にパーツのローカル座標へ移します。計算は**浮動小数点**で行ってください。

#### 拡大率

```
scale = text_size / height_line
```

`text_size` は「1 行の高さを何 px にしたいか」であり、グリフの実寸ではありません。charmap の `height_line` に対する比として、すべてのグリフに掛かります。`height_line == 0` の charmap は 0 除算になるので、ロード時に弾いてください。

#### オフセットの基準

```
base_offset = ( min(glyph.offset.x), min(glyph.offset.y) )   // charmap 内の全グリフの成分ごとの最小値
```

各グリフの `offset` からこの最小値を引いて配置します。charmap 全体で見て一番左上に出るグリフが原点に接するように、レイアウト全体を詰める操作です。**個々の文字列ではなく charmap 単位で決まる定数**なので、フォントのロード時に 1 度だけ求めてキャッシュしてください。

#### 文字を送る

ペン位置 `pen = (0, 0)`、行番号 `line = 0` から始めて、文字列を先頭から 1 文字ずつ:

```
if 文字が '\n':
    pen = (0, pen.y + text_size)      // 行送りは text_size そのもの
    line += 1
    continue

if charmap に glyph が無い:
    何もしない                         // 送りもしない。豆腐も出さない
    continue

pos  = pen + (glyph.offset - base_offset) * scale
size = (glyph.atlas.width, glyph.atlas.height) * scale

// 折り返し（text_width > 0 のときだけ）
if text_width > 0 and pos.x + size.x > text_width:
    pen = (0, pen.y + text_size)
    line += 1
    pos  = pen + (glyph.offset - base_offset) * scale

このグリフを (pos, size) に記録し、行番号 line を覚えておく

pen.x += glyph.width_advance * scale + text_space
```

押さえておく点:

- **行送りは `text_size`** です（`height_line * scale` と同値）。
- **`text_space` はスケールされません。** 送り幅だけが `scale` 倍され、字間は px そのままで足されます。
- **折り返しは文字単位**で、単語境界を見ません。`text_height` は折り返しにも縦のクリップにも**関与しません**（後述の「枠のサイズ」にだけ効きます）。
- **charmap に無い文字は完全に無視されます。** フォールバックも豆腐も無く、ペンも進みません。`\r` にもグリフが無いのが普通なので、CRLF の文字列は結果的に `\r` が落ちて `\n` だけが効きます。

#### 外接矩形と行ごとの範囲

配置し終わったら、全グリフの矩形の和 `bound`（`left / top / right / bottom`）と、**行ごとの右端 `R[line]`** を取ります。

#### 枠のサイズ

```
box.w = (text_width  > 0) ? text_width  : bound.left * 2 + bound.width
box.h = (text_height > 0) ? text_height : bound.top  * 2 + bound.height
```

自動サイズのときに前置オフセットを 2 倍して足しているのは、**内容の外接矩形が枠のちょうど中央に来るようにする**ためです（`left + width + left = box.w`）。

#### アンカーで寄せる

```
// 水平: 行ごと
shift_x[line] = (box.w - R[line]) * (左 ? 0 : 中央 ? 0.5 : 1)

// 垂直: 全行共通
shift_y        = (box.h - bound.bottom) * (上 ? 0 : 中央 ? 0.5 : 1)
```

水平は**行の右端 `R[line]` だけ**を見ます（行は必ず `x = 0` から始まるので、左端は 0 とみなせるためです）。垂直はブロック全体の下端で 1 度だけ決め、全行に同じ量を足します。

#### パーツのローカル座標へ

枠 `box` は**パーツの原点を中心**に置かれます。したがって最終的なグリフ矩形は:

```
local.x =  (pos.x + shift_x[line]) - box.w / 2
local.y = -((pos.y + shift_y)      - box.h / 2)   // Y-up のエンジンでは符号を反転
```

UV は `atlas` を `size_texture_original` で割って正規化します（実際にロードしたテクスチャのサイズでは**ありません**）。テクスチャは `name_texture[glyph.page]` を、フィルタは `text_smooth` に従って linear / nearest でバインドします。

### アウトラインフォント（`text_bitmap == false`）

ssab に入るのは `text_family`（ファミリ名）と、`text_size` / `text_space` / `text_width` / `text_height` / `e_anchor` といったレイアウト設定だけです。**フォントファイルそのものは ssab に含まれません。** ファミリ名からフォントを解決するのはホスト側の役目で、同じフォントが入っていない環境では見た目が変わります。

グリフ表が無いので前節の手順は適用できません。ホストエンジンのテキストスタックに文字列とサイズを渡し、`e_anchor` と `text_width` / `text_height` を、そのスタックのアライメント・折り返し指定に読み替えてください。

---
[目次に戻る](../README.ja.md)
