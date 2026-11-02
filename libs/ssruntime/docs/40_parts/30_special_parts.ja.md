> **日本語** | [English](./30_special_parts.md)

# 9 スライス・シェイプ・テキスト

通常のスプライトとは頂点の取り方が違う 3 つのパーツです。9 スライスとシェイプは、Brain が専用の SoA バッファで頂点を渡します。テキストは、Brain が描画の順番だけを決め、文字の配置と描画は Player が行います。

| パーツ | `DrawBatch.kind` | 頂点 |
| :--- | :--- | :--- |
| 9 スライス | `Nines` | 9 スライス用の SoA バッファ（§1）。`local_vertices` には外接矩形しか入っていない |
| シェイプ | `Shape` | シェイプ用の SoA バッファ（§2）。テクスチャは無い |
| テキスト | `Text` | 無い。Player が ssab から組み立てる（§3） |

## 1. 9 スライス

余白・塗りの方式・繰り返しの数を含むグリッドの生成は Brain が済ませているので、Player は頂点を転送するだけです。

| 関数 | 内容 |
| :--- | :--- |
| `ss_runtime_get_nines_vertices` | パーツのローカル座標の `[x, y, ...]`。原点・画像の反転・座標系の設定を適用済みなので、ローカル頂点と同じくワールド行列を掛けるだけで使える |
| `ss_runtime_get_nines_uvs` | セルマップのテクスチャのピクセル座標の `[u, v, ...]`。ローカル UV と同じく、テクスチャの大きさで割って正規化する |
| `ss_runtime_get_nines_box_coords` | `[fx, fy, ...]`。パーツの左端・上端が `0`、右端・下端が `1`。パーツカラーの 4 隅を補間するのに使う |
| `ss_runtime_get_nines_indices` | 三角形のインデックス。**パーツの中での 0 始まり**の値で、`LT, RT, LB / LB, RT, RB` の順 |
| `ss_runtime_get_nines_vertex_offsets` | 長さ `パーツ数 + 1` の累積和。上の 3 つの頂点のバッファをパーツごとに切り出す。float の位置にするには 2 倍する |
| `ss_runtime_get_nines_index_offsets` | 長さ `パーツ数 + 1` の累積和。インデックスのバッファをパーツごとに切り出す |

パーツ `p` は、頂点 `[vertex_offsets[p], vertex_offsets[p + 1])` とインデックス `[index_offsets[p], index_offsets[p + 1])` を占めます。9 スライスでないパーツは空の範囲です。

`DrawBatch.vertex_count` / `index_count` には、その DrawBatch のパーツの合計が入るので、バッファを先に確保できます。ただし 16 bit なので 65535 で頭打ちになります。タイルが大量に並ぶパーツでは、累積和から実際の数を取ってください。

頂点の数は、塗りの方式で変わります。**引き伸ばし**（`nines_fill_mode == 0`）は 4x4 のグリッドを共有して 16 頂点・54 インデックスに固定です。**タイル**（`== 1`）は繰り返し 1 枚ごとに矩形を出すので、パーツの大きさで増減します。余白が重なる場合は Brain が収めるので、Player が確かめる必要はありません。

9 スライスのパーツには、UV アニメーション（`UVTX` / `UVTY` / `UVRZ` / `UVSX` / `UVSY`）と、セルマップの中で回転して配置されたセルは適用されません。どちらもオーサリングツールの挙動に合わせたものです。

## 2. シェイプ

シェイプは、三角形・矩形・星などの図形を描くパーツです。テクスチャを持たず、`DrawBatch.texture_hash` は `0` です。

| 関数 | 1 パーツあたり | 内容 |
| :--- | :--- | :--- |
| `ss_runtime_get_shape_vertices` | 24 float（12 頂点 × `(x, y)`） | パーツのローカル座標の頂点。原点と座標系の設定を適用済みなので、ワールド行列を掛けるだけで使える |
| `ss_runtime_get_shape_vertex_counts` | 1 個 | そのパーツの有効な頂点の数。残りの枠は 0 |
| `ss_runtime_get_shape_vertex_box_coords` | 24 float | 各頂点の、外接矩形の中での位置 `(fx, fy)`。左上が `(0, 0)`、右下が `(1, 1)` |

- 頂点と外接矩形内の位置のバッファは `part_index * 24` で引きます。パックにシェイプのパーツが 1 つも無いときは、どちらも空です。頂点の数はそのときもパーツごとに 1 個あり、すべて `0` です。
- 頂点は**三角形のファン**の順に並んでいます。先頭の頂点が中心で、最後の頂点は 2 番目の頂点と同じ位置に戻って図形を閉じます。
- パーツカラーは、4 隅の色を `(fx, fy)` で双線形補間して各頂点の色にします（[パーツカラー §4-3](./10_parts_color.ja.md#4-3-シェイプ)）。
- マスクとして作画されたシェイプには、マスクの閾値が効かず、図形全体がマスクになります（[マスク §2-5](./40_mask.ja.md#2-5-カットアウトの閾値)）。

## 3. テキスト

テキストの描画は Player が行います（[制限事項 §1](../10_introduction/20_limitations.ja.md#1-テキストとサウンドは-player-が実装する)）。Brain は文字を配置も描画もせず、パーツに描画の順番を与え、作画された設定を ssab に載せて渡します。ここでは、そこから文字を並べるまでの仕様を説明します。

### 3-1. Brain が渡すもの

テキストパーツは、**1 パーツにつき 1 つの DrawBatch** になります。

| DrawBatch のフィールド | 値 | 意味 |
| :--- | :--- | :--- |
| `kind` | `Text` | テキストパーツであることを表す |
| `count` | 常に `1` | 他のテキストパーツとまとめられない |
| `vertex_count` / `index_count` | 常に `0` | 頂点は入っていない |
| `texture_hash` | 常に `0` | テクスチャは DrawBatch からは決まらない（フォントは ssab から引く） |
| `blend_type` | 通常どおり | 文字を描くときの描画先との合成に使う |

テキストパーツがまとめられないのは、**どのフォントで描くかを Brain が区別できない**からです。2 つのテキストパーツを 1 つの DrawBatch にまとめると、途中でフォントが変わったことを Player が見分けられなくなります。DrawBatch が 1 パーツであることを前提にしてかまいません。

```cpp
for (auto batch : *frame_data->draw_batches()) {
    if (batch->kind() != ss::runtime::DrawBatchKind::Text) continue;

    int p          = frame_data->draw_order()->Get(batch->start_rank());
    int part_index = frame_data->parts()->Get(p)->part_index();
    auto text      = ssab->parts()->Get(part_index)->part_type_as_PartTypeText();   // §3-2 の設定

    draw_glyphs(text, &world_matrices[p * 16], frame_data->parts()->Get(p));
}
```

`draw_glyphs` は、§3-3 のレイアウトで文字を並べて描く、実在しない関数です。

- **テキストパーツはセルを参照しません。** `PartState.cell` は `-1` で、ローカル頂点は 0 です。矩形すら来ないので、**頂点はすべて Player が作ります。**
- 位置・回転・拡縮・不透明度・描画優先度は、通常どおり `PartState` とワールド行列に入っています。文字はパーツのローカル座標で組み立て、ワールド行列を掛けてください。
- **文字列はアニメーションしません。** ssab に入っているのはパーツの定義にある 1 つの文字列（`PartTypeText.text`）だけで、文字列を変えるキーフレームはありません。
- **マスクとして作画されたテキスト**（`draw_as_mask`）は、描画計画から除外されます。

### 3-2. ssab に入っているもの

#### `PartTypeText`: パーツの設定

| フィールド | 型 | 内容 |
| :--- | :--- | :--- |
| `text` | `string` | 表示する文字列（UTF-8）。改行は `\n` |
| `text_bitmap` | `bool` | `true` = ビットマップフォント（`text_char_map` を使う） / `false` = アウトラインフォント（`text_family` を使う） |
| `text_family` | `string` | アウトラインフォントのファミリ名（例: `Arial`）。**フォントのファイルは ssab に入らない** |
| `text_char_map` | `string` | ビットマップフォントの charmap 名。`FontBitmap.name` と**同じ文字列**で、プロジェクトからの相対パスと拡張子を含む（例: `font/RoundedMPlus.fnt`） |
| `text_size` | `int` | 1 行の高さ（px）。文字の拡大率はここから決まる（§3-3） |
| `text_space` | `float` | 文字の間に足す量（px）。**`text_size` で拡大されない** |
| `text_smooth` | `bool` | フォントのテクスチャのフィルタ。`true` = linear / `false` = nearest |
| `text_width` | `int` | レイアウトの枠の幅（px）。`0` なら内容に合わせる |
| `text_height` | `int` | レイアウトの枠の高さ（px）。`0` なら内容に合わせる |
| `e_anchor` | `TextAnchor` | 枠の中での寄せ方 |

`e_anchor` は、**上位 4 bit が水平、下位 4 bit が垂直**です。

| | `0` | `1` | `2` |
| :--- | :--- | :--- | :--- |
| **上位 4 bit**（`e_anchor >> 4`） | 左 | 中央 | 右 |
| **下位 4 bit**（`e_anchor & 0x0F`） | 上 | 中央 | 下 |

つまり `LT = 0x00`、`CC = 0x11`、`RB = 0x22` です。`NO = 0xFF` は指定なしで、左上（`LT`）として扱ってください。

#### `FontBitmap` / `FontGlyphBitmap`: ビットマップフォントのグリフの表

`ssab->font_bitmaps()` に入るのは、**`text_bitmap == true` のテキストパーツが実際に参照している charmap だけ**です。アウトラインフォントのパーツしか無いプロジェクトでは、`font_bitmaps()` 自体がありません（C++ では `nullptr`）。

`PartTypeText.text_char_map` の文字列をそのまま `fnv1a` でハッシュした値が `FontBitmap.name_hash` なので、パーツからフォントはハッシュ 1 回で引けます。

| `FontBitmap` | 内容 |
| :--- | :--- |
| `name` / `name_hash` | charmap 名（= `text_char_map`）とそのハッシュ |
| `name_face` | 元のフォントのフェイス名。表示用で、描画には使わない |
| `base` | ベースラインの位置（px、行の上端から） |
| `height_line` | charmap の 1 行の高さ（px）。**拡大率の分母** |
| `size_texture_original` | ページ画像の元の大きさ（px）。**`atlas` の正規化にはこの値を使う**（読み込んだテクスチャの大きさではない） |
| `name_texture[]` | ページ画像のファイル名。**ページ番号の順**に並んでいるので、`name_texture[glyph.page]` で引ける |
| `name_texture_hash[]` | 上のハッシュ（埋め込みモードでのアセットのキー） |
| `table_data[]` | グリフの表。`id` の昇順に並んでいるので、二分探索できる |

| `FontGlyphBitmap` | 内容 |
| :--- | :--- |
| `id` | 文字コード（UTF-16 のコード単位） |
| `atlas` | ページ画像の中のグリフの領域（`x`, `y`, `w`, `h`） |
| `offset` | ペンの位置からグリフを描き始める位置のずれ（px、Y は下向きが正） |
| `width_advance` | 送り幅（px） |
| `text` | `id` を文字にしたもの。デバッグ表示用 |
| `page` | `name_texture[]` の添字 |

`offset` と `size_texture_original` は ssab の `Vec2` で、成分の名前は `x` / `y` ではなく `v1` / `v2` です（`v1` が X または幅、`v2` が Y または高さ）。以降の式の `glyph.offset.x` / `glyph.offset.y` は `offset.v1` / `offset.v2` のことです。

ページ画像の名前は、charmap のファイルからの相対パスをコンバータが解決済みで、他の外部リソースと同じ**プロジェクトからの相対パス**として入っています（[ロードと初期化 §3-1](../30_core/10_initialization.ja.md#3-1-ssab-に入っているファイル参照の文字列)）。

> [!IMPORTANT]
> **使ってはいけないフィールド**
>
> - `FontBitmap` の `size` / `padding_l` / `padding_r` / `padding_u` / `padding_d` / `scale` / `spacing` と `FontGlyphBitmap.channel` は**固定値（`0` か `1.0`）**で、作画の内容を反映していません。レイアウトに使わないでください。文字の間は `PartTypeText.text_space`、行の高さは `height_line` から取ります。
> - **カーニングは ssab に入りません。** 送り幅は `width_advance` だけで決まります。
> - `FontGlyphBitmap.id` は 16 bit なので、**U+FFFF を超える文字は扱えません。** 絵文字や一部の漢字を使う場合は、アウトラインフォントを使ってください。

### 3-3. レイアウト（ビットマップフォント）

**X は右、Y は下**（グリフの `offset` の向き）で計算し、最後にパーツのローカル座標へ移します。計算は**浮動小数点**で行ってください。

#### 拡大率

```
scale = text_size / height_line
```

`text_size` は「1 行の高さを何 px にするか」で、グリフの実際の大きさではありません。charmap の `height_line` との比が、すべてのグリフに掛かります。`height_line == 0` の charmap は 0 で割ることになるので、読み込むときに除外してください。

#### ずれの基準

```
base_offset = ( min(glyph.offset.x), min(glyph.offset.y) )   // charmap の全グリフについて、成分ごとの最小値
```

各グリフの `offset` からこの最小値を引いて配置します。charmap の中で一番左上に出るグリフが原点に接するように、全体を詰める操作です。**文字列ではなく charmap ごとに決まる定数**なので、フォントを読み込むときに 1 回だけ求めてキャッシュしてください。

#### 文字を送る

ペンの位置 `pen = (0, 0)`、行番号 `line = 0` から始め、文字列の先頭から 1 文字ずつ次のように処理します。

```
if 文字が '\n':
    pen = (0, pen.y + text_size)      // 行送りは text_size
    line += 1
    continue

if charmap にグリフが無い:
    何もしない                         // 送りもしない。代わりの字形も出さない
    continue

pos  = pen + (glyph.offset - base_offset) * scale
size = (glyph.atlas.w, glyph.atlas.h) * scale

// 折り返し（text_width > 0 のときだけ）
if text_width > 0 and pos.x + size.x > text_width:
    pen = (0, pen.y + text_size)
    line += 1
    pos  = pen + (glyph.offset - base_offset) * scale

このグリフを (pos, size) に置き、行番号 line を記録する

pen.x += glyph.width_advance * scale + text_space
```

- **行送りは `text_size`** です（`height_line * scale` と同じ値）。
- **`text_space` は拡大されません。** 拡大するのは送り幅だけで、文字の間は px のまま足します。
- **折り返しは文字単位**で、単語の区切りを見ません。`text_height` は折り返しにも縦の切り取りにも**関係しません**（枠の大きさにだけ効きます）。
- **charmap に無い文字は完全に無視します。** 代わりの字形も出さず、ペンも進めません。`\r` にもふつうグリフが無いので、CRLF の文字列では `\r` が落ちて `\n` だけが効きます。

#### 外接矩形と行ごとの右端

すべてのグリフを置いたら、全グリフの矩形を囲む `bound`（`left / top / right / bottom`。幅は `width = right - left`、高さは `height = bottom - top`）と、**行ごとの右端 `R[line]`** を求めます。

#### 枠の大きさ

```
box.w = (text_width  > 0) ? text_width  : bound.left * 2 + bound.width
box.h = (text_height > 0) ? text_height : bound.top  * 2 + bound.height
```

自動の大きさで左上のずれを 2 倍して足すのは、**内容の外接矩形が枠のちょうど中央に来るようにする**ためです（`left + width + left = box.w`）。

#### 寄せる

```
// 水平: 行ごと
shift_x[line] = (box.w - R[line]) * (左 ? 0 : 中央 ? 0.5 : 1)

// 垂直: 全行で共通
shift_y        = (box.h - bound.bottom) * (上 ? 0 : 中央 ? 0.5 : 1)
```

水平方向は**行の右端 `R[line]` だけ**を見ます（行は必ず `x = 0` から始まるので、左端は 0 とみなせます）。垂直方向は全体の下端で 1 回だけ決め、全行に同じ量を足します。

#### パーツのローカル座標へ

枠 `box` は**パーツの原点を中心**に置かれます。最終的なグリフの矩形は次のとおりです。

```
local.x =  (pos.x + shift_x[line]) - box.w / 2
local.y = -((pos.y + shift_y)      - box.h / 2)   // Y-Up（既定）の式。座標系を Y-Down に設定した場合は先頭の - を付けない
```

UV は、`atlas` を `size_texture_original` で割って正規化します（読み込んだテクスチャの大きさでは**ありません**）。テクスチャは `name_texture[glyph.page]` を、フィルタは `text_smooth` に従って linear / nearest でバインドします。

### 3-4. アウトラインフォント（`text_bitmap == false`）

ssab に入るのは、`text_family`（ファミリ名）と、`text_size` / `text_space` / `text_width` / `text_height` / `e_anchor` のレイアウトの設定だけです。**フォントのファイルは ssab に入りません。** ファミリ名からフォントを探すのはホストの役目で、同じフォントの無い環境では見た目が変わります。

グリフの表が無いので、§3-3 の手順は使えません。ホストのエンジンのテキスト機能に文字列と大きさを渡し、`e_anchor` と `text_width` / `text_height` を、その機能の寄せ方と折り返しの指定に読み替えてください。

---
[目次に戻る](../README.ja.md)
