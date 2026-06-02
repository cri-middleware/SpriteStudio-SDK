# Mask の実装

SpriteStudio 7 の「マスク」「クリッピングマスク」機能の実装ガイドです。両者はステンシルという同じ仕組みで実現されますが、**機能としては別物** です。本書は機能の定義を一次情報とし、実装パターンはそれに従属するものとして扱います。

---

## 1. 概念とパーツの役割

### 用語

混乱しやすいフィールド名・用語を先に整理します。詳細は各節を参照してください。

| 用語 / フィールド | 読み出し元 | 意味 |
|---|---|---|
| **マスク（パーツ）** `PartTypeMask` | PartData（静的） | マスク専用のパーツ種別。常に pure マスク（色を描かない） |
| **マスク（機能）** | — | 自分より描画優先度が **低い** パーツを、形状の **外側** に残す |
| **クリッピング（機能）** | — | 自分より描画優先度が **高い** パーツを、形状の **内側** に収める |
| **writer** | — | ステンシルに書き込むパーツ（pure マスク または クリッピング） |
| **対象 (target)** | — | マスク／クリッピングの影響を受ける描画パーツ |
| **pure マスク** | — | 色を描かず穴になる writer（マスクパーツ＋マスク化された描画パーツ） |
| `Priority` | **FrameData（毎フレーム）** | 描画優先度。同着はパーツ ID で決まる（§2-1） |
| `mask` (float, `0..255`) | **FrameData（毎フレーム）** | カットアウト閾値を兼ねる。`0` で writer 無効（§2-2 / §2-5） |
| `mask_influence` | **PartData（静的）** | **役割で意味が変わる**（§1-2）。OP または マスク対象フラグ |
| `mask_write` | **PartData（静的）** | 描画パーツを **クリッピング** writer にするフラグ |
| `draw_as_mask` | **PartData（静的）** | 描画パーツ (Shape/Text/Nines) を **pure マスク** 化する統合フラグ。オーサリングでは種別ごとの設定だが ssab では1つに集約 |
| `visible_inside_mask` | **PartData（静的）** | 極性。`1` = 内側に描画 / `0` = 外側に描画（§2-4） |
| `Hide` | **FrameData（毎フレーム）** | 非表示。※ Player では通常チェック不要（§2-2 補足） |

> **【重要】読み出し元の区別**: マスク関連フィールドは 2 つの異なるテーブルに分かれています。実装時はどちらから読むかを必ず意識してください。
> - **PartData（静的・パーツ単位）**: `mask_influence` / `mask_write` / `visible_inside_mask` / `draw_as_mask` / パーツ種別。**1 回読めば以後フレームで不変**。
> - **FrameData（毎フレーム・アニメート）**: `mask` (float) / `Priority` (float) / `Hide` / `part_color` / `alpha`。**毎フレーム読み直す**。
>
> 以降の擬似コードでは出所を `part.data.*`（PartData）と `part.frame.*`（FrameData）で区別します。

### 1-1. マスクとクリッピングは別機能

用途は似ており実装も同じ仕組みですが、機能としては別物です。

| | **マスク** | **クリッピング** |
|---|---|---|
| 機能するパーツ種別 | **「マスク」パーツ** | 描画パーツ（通常 / メッシュ / シェイプ等） |
| 自分の色描画 | **しない**（pure。穴になる） | **する**（自分を描画しつつステンシルにも書く） |
| 適用先 | 自分より **描画優先度が低い** 描画パーツ | 自分より **描画優先度が高い** 描画パーツ |
| 典型的な見え方 | 形状の **外側** を残す | 形状の **内側** に収める |

「マスク」は専用パーツ種別が単独で持つ機能、「クリッピング」は描画パーツが付帯的に持つ機能、という非対称な関係です。

### 1-2. パーツの役割と `mask_influence`

各パーツは「writer か」「対象か」で役割が決まります。「マスク機能」を持つのはマスクパーツのみですが、一部の描画パーツ種別は共通フラグ **`draw_as_mask`** によって **マスクパーツの役割に変わる**（pure マスク化）ことができます。オーサリングでは種別ごとの設定ですが、ssab では `PartData.draw_as_mask` 1つに集約されるため、下表の Shape/Text/Nines は同一条件になります。

| パーツ種別 | 既定の役割 | pure マスク化条件 | クリッピング (`mask_write`) を持てるか |
|---|---|---|---|
| **マスク** (`PartTypeMask`) | 常に pure マスク | （常に） | — |
| **通常** (Normal) | 描画 | （ pure マスクになれない） | ○ |
| **メッシュ** (Mesh) | 描画 | （ pure マスクになれない） | ○ |
| **シェイプ** (Shape) | 描画 | `draw_as_mask == 1` | ○ (`draw_as_mask == 0` 時) |
| **テキスト** (Text) | 描画 | `draw_as_mask == 1` | ○ |
| **ナインスライス** (Nines) | 描画 | `draw_as_mask == 1` | ○ |

> **シェイプの特例**: `draw_as_mask == 1` でマスク化されたシェイプは Cell（テクスチャ）を持たないため、後述のカットアウト閾値（§2-5）は効きません。**形状全体がマスク**になります。

プレイヤー側の writer 判定ロジックは概念的に次のようになります：

```
pure_mask   = PartTypeMask || draw_as_mask
is_writer   = pure_mask || mask_write          // ステンシルに書く全パーツ
is_clipping = mask_write && !pure_mask         // 色も描く（高優先度に適用）
```

#### `mask_influence` の二重定義（重要）

`mask_influence` は同一フィールドですが、パーツの **役割で意味が変わります**。`mask_influence` は §2 のいくつかのルールで登場するため、先にここで意味を確定させておきます。プレイヤーは役割ごとに使い分けてください。

| 役割 | `mask_influence` の意味 |
|---|---|
| **マスクパーツ（writer 専任）** | **OP**。`0` = 増分 / `1` = 反転（§2-3） |
| **描画パーツ（対象側、writer ではない）** | **マスク対象フラグ**。`1` = 適用される / `0` = オプトアウト（§2-2） |
| **クリッピング（writer 兼 描画パーツ）** | 上記の **両方を同時に兼ねる**。`mask_influence` の **同じ値** が「OP」と「対象フラグ」の両方として解釈される |

クリッピングは「描画パーツでありかつ writer でもある」存在なので、`mask_influence` の値が両方の役割で同時に使われます。たとえば `mask_influence == 1` のクリッピングは：

- **対象フラグ = 1**：自身が他のマスクの影響を受ける（マスクに対する opt-in）
- **OP = 反転**：自身が writer として書き込む際は反転 OP で書く

の二つが同時に成立します。これは仕様上 **同じフィールドを意図的に流用** している設計です。Player 側で値を分けたり、片方の意味を捨てたりしないでください。

---

## 2. 判定ルール

機能の違い（マスク／クリッピング）はあれど、適用ロジックは以下のいくつかの単純なルールに分解して、統一的に扱うことができます。各ルールで使う `mask_influence` の意味は役割で変わる点に注意してください（§1-2）。

### 2-1. スコープ（どの描画パーツにマスクが及ぶか）

描画優先度を基準に決まります：

- **マスク** → 自分より **描画優先度が低い** 描画パーツに適用（描画優先度が自身を超えるとマスクの効果が自動消滅する）
- **クリッピング** → 自分より **描画優先度が高い** 描画パーツに適用（描画最高優先度まで持続するためマスクの効果は自動消滅しない）

描画優先度の決定（SpriteStudio 共通ルール）：

- 原則 **FrameData の Priority** で決定
- 同着の場合、**パーツ ID（格納インデックス）が高い方が上**（= 描画優先度が高い）

実装上は `draw_order`（Priority 昇順・同着は ID 昇順で並ぶパーツ index 列）の位置 `rank` とした時に:

- マスク: `rank < M`（writer M より前のスコープ）
- クリッピング: `rank > C`（writer C より後のスコープ）

### 2-2. マスクの効果のオプトアウト

- マスクする側（ writer ）が機能するのは、次の **両方** の条件を満たす必要があります。
    - **`mask != 0`**（`mask == 0` → 何も書かない＝動作せず）
    - **`Hide == false`**（`Hide == true` → 何も書かない）

- 描画パーツがマスクされるのは、次を満たす必要があります。
    - 描画パーツが **`mask_influence == 1` のときのみ** マスク／クリッピングの影響を受けます。`mask_influence == 0` の描画パーツはマスクの影響を受けません（オプトアウト）。

> 補足: `Hide == true` の writer は ssruntime が draw_order / DrawBatch に含めないため、プレイヤー側で `Hide` を意識する必要はありません。**`mask` 値だけプレイヤーでチェック** してください。

### 2-3. OP（ステンシルへの書込演算）

- **マスクパーツ**: `mask_influence` が OP を兼ねる。`0` = **増分**（`stencil + 1`）/ `1` = **反転**（`~stencil`）
- **クリッピング**: 同じく `mask_influence` が OP を兼ねる（マスクパーツの OP 解釈と同一）。クリッピングは描画パーツでもあるため、**同じ `mask_influence` 値が「対象フラグ」（§2-2）と「OP」の両方として同時に使われる**（詳細は §1-2）

### 2-4. 極性（マスクの内側か外側のどちらに描画するか）

極性は **機能では決まらず、被マスク対象パーツの `visible_inside_mask`** で一律に決まります。

- `visible_inside_mask == 1` → マスクの **内側** に描画
- `visible_inside_mask == 0` → マスクの **外側** に描画
- 判定式: `masked == visible_inside`（true で描画 / false で `discard`）

> 補足: §1-1 の表に書いた「マスク = 外側 / クリッピング = 内側」は **典型的なオーサリング既定値** であって、極性そのものは対象側のプロパティです。**対象パーツがマスクとクリッピングの両方のスコープに同時に入る場合も、1 本のステンシル + 対象 1 個の `visible_inside_mask` で判定** します（機能ごとで極性を分けていません）。

### 2-5. カットアウト閾値（被覆に書く画素の範囲）

マスク・クリッピング **共通** で、FrameData の `mask` 値（`0..255`）から決まります。シェーダで処理する想定です：

- 閾値 = `(255 - mask) / 255`
- `mask == 255` → 閾値 0 → **完全透明（α=0）以外のすべての画素** を被覆に書く
- `mask == 0` → 閾値 1.0 → α > 1.0 を要求（あり得ない）→ **何も書かない**（§2-2 と整合）

**テクスチャを持たないシェイプ** は閾値の対象外（幾何全体が形になる）。

---

## 3. 実装方式

### 3-1. GPU ステンシルバッファが使える場合（推奨）

SpriteStudio オリジナルのマスク機構はステンシルバッファに沿った設計です。エンジンがステンシルを公開している場合、それを使うのが最も素直です：

> [!NOTE]
> Unity の Shader Graph などのモダンなノードベースシェーダは、通常ステンシルバッファの細かな出力をサポートしていません。エンジンの制約によっては、マスク処理用のパスだけは手書きのシェーダ（HLSL等）や専用の RenderPass を用いて住み分ける必要がある点に留意してください。

1. `draw_order` を順に走査
2. Writer が来たら：カットアウト閾値で `discard` した残り画素を、OP（増分 / 反転）でステンシル更新
3. 対象描画パーツが来たら：ステンシル値と `visible_inside_mask` で `discard` 判定

### 3-2. ステンシルが公開されていないエンジンの場合

例として Godot 4.6 の 2D canvas はステンシルバッファを公開していません。このようなエンジンでは、以下のハイブリッド方式でステンシルを再現できます：

1. **被覆ビットマップの生成**: フレーム毎に、その時の全マスク writer をオフスクリーンの被覆ビットマップへラスタライズ。各 writer に 1 ビットを割り当て、被覆する画素にそのビットを立てる。
2. **描画時の replay**: 被マスク対象は描画シェーダ内で被覆ビットマップをサンプルし、アクティブな writer を draw_order 順に replay してステンシル状態を再構成、`discard` で内外をクリップする。

要点は **「描画時刻に依存する状態を持たず、被覆（writer の形）と replay（対象側の再構成）に分離する」** こと。被覆 1 枚で 24 writer まで扱えるエンコード（RGBA8 の RGB チャンネルに 8 bit × 3 = 24 bit）が実用範囲です。

> **被覆ビットマップのサイズについて**：writer 全体の bbox に合わせて動的にスケールし、**長辺を上限値（例：1024 px）に揃える** 実装が実用的です。Full HD クラスの画面でピクセル等倍精度を確保しつつ、1 枚あたり最大 4 MiB（RGBA8）に収まります。アニメーションを多数同時表示する場合は、画面解像度連動や小プール共有でさらに削減できます（最適化案）。

#### 3-2-1. 参考実装（被覆ビットマップ方式の擬似コード）

以下、エンジン非依存の擬似コードで全体の流れを示します。

**データ構造（最大 24 writer、RGBA8 の RGB チャンネルに 8 bit × 3）**

```
const int MAX_WRITERS = 24;

struct MaskWriter {
    int  part_index;     // パーツインデックス
    int  bit;            // 0..23、被覆内のビット位置
    int  draw_rank;      // draw_order 上の rank
    bool op_invert;      // OP: false=増分, true=反転
    bool is_clipping;    // クリッピング（色も描く）
};
```

**Writer 集合の確定（毎フレーム）**

```
function build_mask_writers(frame) -> [MaskWriter]:
    writers = []
    for rank, part_index in enumerate(frame.draw_order):
        part = frame.parts[part_index]
        // part.data.*  = PartData（静的）/ part.frame.* = FrameData（毎フレーム）
        pure_mask  = part.data.type == Mask || part.data.draw_as_mask
        is_writer  = pure_mask || part.data.mask_write
        if not is_writer:               continue
        if part.frame.mask == 0:        continue       // §2-2 opt-out（毎フレーム値）
        if len(writers) >= MAX_WRITERS: break

        writers.append(MaskWriter {
            part_index  = part_index,
            bit         = len(writers),                 // 0,1,2,...
            draw_rank   = rank,
            op_invert   = part.data.mask_influence,     // §2-3 OP（pure マスク・クリッピング共通）
            is_clipping = part.data.mask_write && !pure_mask,
        })
    return writers
```

**被覆ビットマップ生成（毎フレーム）**

オフスクリーンの RGBA8 viewport に additive blend で各 writer の幾何を描画。disjoint な bit が OR されるよう、各 writer は自分の bit だけが立つ色で描画します。

```
function render_coverage_bitmap(writers, frame) -> Bitmap:
    bitmap = create_offscreen_RGBA8_with_additive_blend()
    for w in writers:
        channel   = w.bit / 8                          // 0=R, 1=G, 2=B
        bit_val   = (1 << (w.bit % 8)) / 255.0
        bit_color = vec4_zero_except(channel, bit_val)
        threshold = (255 - frame.parts[w.part_index].frame.mask) / 255.0  // mask は float

        draw_geometry(
            geometry = geometry_of(w.part_index),       // Normal=sprite / Mesh / Shape
            shader   = MASK_WRITE_SHADER,
            uniforms = { mask_bit_color: bit_color, mask_threshold: threshold }
        )
    return bitmap
```

**ビット書込みシェーダ（GLSL）**

```glsl
uniform vec4  mask_bit_color;   // 該当 RGB チャンネルに pre-encode されたビット
uniform float mask_threshold;   // (255 - mask) / 255

void fragment() {
    // §2-5 カットアウト: α が閾値を超える画素のみ被覆へ
    if (texture(TEXTURE, UV).a <= mask_threshold) discard;
    // additive blend が src.rgb * src.a で動くので、alpha=1 が必須
    COLOR = vec4(mask_bit_color.rgb, 1.0);
}
```

**描画時の replay（被マスク対象シェーダ、GLSL）**

```glsl
uniform sampler2D coverage;
uniform int   mask_count;
uniform vec4  mask_meta[MAX_WRITERS]; // (draw_rank, bit, op_invert, is_clipping) ※ build_mask_writers の MaskWriter と対応
uniform float this_rank;
uniform float visible_inside;         // §2-4 極性（1=内側 / 0=外側）

bool mask_passes(vec2 cov_uv) {
    if (mask_count == 0) return true;
    // bbox 外は writer 無しと同じ扱い（端の CLAMP による誤マスク回避）
    if (cov_uv.x < 0.0 || cov_uv.x > 1.0 || cov_uv.y < 0.0 || cov_uv.y > 1.0)
        return visible_inside <= 0.5;

    vec4 cov = texture(coverage, cov_uv);   // sampler は nearest 必須
    int stencil = 0;
    for (int i = 0; i < mask_count; i++) {
        vec4 m = mask_meta[i];
        bool is_clipping = m.w > 0.5;
        // §2-1 スコープ判定（クリッピング=自分より後 / マスク=自分より前）
        bool active = is_clipping ? (this_rank > m.x) : (this_rank < m.x);
        if (!active) continue;
        // 被覆からこの writer のビットを抽出
        int bit  = int(m.y + 0.5);
        int byte = int(cov[bit / 8] * 255.0 + 0.5);
        if (((byte >> (bit % 8)) & 1) == 0) continue;
        // §2-3 OP
        if (m.z > 0.5) stencil = (~stencil) & 0xFF;     // 反転
        else           stencil = (stencil + 1) & 0xFF;  // 増分
    }
    bool masked = (stencil != 0);
    return masked == (visible_inside > 0.5);            // §2-4 判定式
}

void fragment() {
    // ... 通常の色合成 ...
    if (!mask_passes(coverage_uv)) discard;
    COLOR = ...;
}
```

**実装上の注意**

- **被覆 sampler は nearest 必須**（LINEAR だと隣接ビットがブレンドして誤復号）
- **bbox 外は `visible_inside` だけで分岐**（writer 無し相当として扱う）
- `mask_meta[i].x`（= writer 自身の `draw_rank`）を、対象の `this_rank` と前後比較する（§2-1 スコープ判定）

#### 3-2-2. 被覆 UV の導出とインスタンス（サブアニメ）への継承

**被覆 UV の導出**: replay シェーダの `coverage_uv` は、頂点ステージで **被マスク対象の player-local 座標を被覆 UV `[0,1]` へ写すアフィン変換** `M`（§3-2 の bbox 正規化＝ basis 2x2 ＋ offset）で求めます。被覆ビットマップは「マスク writer を所有するアニメ」の bbox に張られるため、`M` はそのアニメの player-local 空間が基準です。

**インスタンス（サブアニメ）への継承**: SpriteStudio のマスクは、被マスク対象が **インスタンスパーツ（サブアニメ参照）** の場合、**そのサブアニメ全体**へ及びます。被覆とメタは親アニメが所有するので、子へ次のように引き渡して replay します。

1. 親が自分の **被覆テクスチャ・`mask_meta`・writer 数** をインスタンス子のプレーヤーへ渡す。
2. UV 変換を合成する: `M_child = M_parent · slot_xf`（`slot_xf` = 子-local → 親-local のインスタンス配置行列）。これで子の各頂点の player-local 座標が **親の被覆 UV** に正しく写る。
3. サブアニメ全体を **インスタンスパーツ自身の `rank` と `visible_inside_mask`** で一括判定する（子の個々のパーツ単位では評価しない）。

要点:

- **継承は単段（1 階層）**。インスタンスは「1 個の被マスク対象」として扱う。
- 継承が働くのは、インスタンスパーツが **マスク対象（`mask_influence == 1` または `mask_write`）かつ親マスクのスコープ内（または `visible_inside`）** のときのみ。
- 子は親の被覆を参照するだけで、自前の被覆は生成しない（子が自分のマスクを持つ場合はそれとは別系統）。

### 3-3. 共通の注意点

- **pure マスクは色を描かない**: マスクパーツ（と pure マスク化された描画パーツ）の色描画バッチはスキップしてください。
- **writer 数の上限**: ステンシル幅 / 被覆エンコードのビット数で writer 上限が決まります。実用上は 1 フレームあたり数個〜数十個に収まります。
- **被覆方式の bbox 外の扱い**: 被覆ビットマップ方式の場合、bbox 外の対象パーツは「writer 無し」と同じ扱い（マスクされない）にすることで、テクスチャ端の CLAMP による誤マスクを避けられます。

---

## 4. 実装チェックリスト

実装漏れの自己点検用。各項目の根拠は対応する節を参照してください。

- [ ] **役割判定**（§1-2）: `pure_mask = PartTypeMask || draw_as_mask` / `is_writer = pure_mask || mask_write` / `is_clipping = mask_write && !pure_mask` の 3 式で分類した。
- [ ] **読み出し元**（用語表）: `mask_influence` / `mask_write` / `visible_inside_mask` / `draw_as_mask` を **PartData（静的）** から、`mask` / `Priority` / `Hide` を **FrameData（毎フレーム）** から読んでいる。
- [ ] **writer 無効化**（§2-2）: `mask == 0` の writer はスキップした（`Hide` は draw_order に含まれないので Player では未チェックでよい）。
- [ ] **オプトアウト**（§2-2）: 対象描画パーツは `mask_influence == 1` のときのみ影響を受ける。
- [ ] **OP**（§2-3）: `mask_influence` を `0`=増分 / `1`=反転 として、pure マスク・クリッピング共通で適用した。
- [ ] **極性**（§2-4）: 機能ではなく **対象パーツの `visible_inside_mask`** で `masked == visible_inside` を判定した。
- [ ] **スコープ**（§2-1）: マスク=自分より rank 小、クリッピング=自分より rank 大の対象に適用した。
- [ ] **カットアウト閾値**（§2-5）: `(255 - mask) / 255`（`mask` は float）で被覆に書く画素を絞った。テクスチャを持たないシェイプは閾値対象外。
- [ ] **pure マスクの色描画スキップ**（§3-3）: マスクパーツ／pure マスク化パーツの色バッチを描いていない。
- [ ] **インスタンス継承**（§3-2-2）: 被マスク対象がインスタンスなら、サブアニメ全体を 1 対象として親の被覆・メタで一括判定した（単段のみ）。

---
[目次に戻る](./README.ja.md)
