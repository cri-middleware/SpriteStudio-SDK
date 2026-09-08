> **日本語** | [English](./30_render_loop.md)

# レンダリングループとバッチ描画 (Rendering & Batching)

`libssruntime` (Brain) が算出した計算結果を効率的に取得し、ゲームエンジンで描画するための「レンダリング契約」について記載します。

## 1. SoA バッファによる高速なデータ取得

現代的な Player 実装では、パーツごとに計算結果を取得するのではなく、**SoA (Structure of Arrays) 形式で一括取得** する方式が標準です。

`ss_runtime_get_world_matrices(ctx, &ptr, &len)` などの API を使用することで、現在のフレームの全パーツの計算済みデータをポインタ経由で直接参照できます。

| 取得できるデータ | API 名 | 用途 |
| :--- | :--- | :--- |
| **ワールド行列** | `ss_runtime_get_world_matrices` | 各パーツの最終的な位置・回転・拡大縮小（4x4 行列） |
| **ローカル頂点** | `ss_runtime_get_local_vertices` | Pivot 反転等が適用済みの頂点座標（5頂点 x 2軸） |
| **テクスチャハッシュ** | `ss_runtime_get_cell_texture_hashes` | パーツが参照している **セル側の** テクスチャの識別子（バインド用ではありません。§2 参照） |
| **UV / セルメタ** | `ss_runtime_get_local_uvs`, `cell_meta` | アニメーション済みの UV 座標やセルサイズ |

### ワールド行列は鏡像になりうる — 安易に分解しないこと

ワールド行列は一般のアフィン基底であり、「位置・回転・拡大縮小」の 3 要素に還元できるものではありません。行列式が負になるのは、姿勢に**奇数回**の鏡映がかかっているときです。要因は独立に 2 つあり、**パーツ自身または祖先に付いた負のスケール**と、**Player 自身がルート変換に畳み込む反転**（左右反転機能など）です。

**この 2 つは合成されるため、偶数回で打ち消し合います。** Player 側で反転しても行列式が負になるとは限りません。反転は符号を**反転させる**だけです。実データでは元から鏡映が付いたパーツは珍しくなく（`Ringo` テストアセットでは 69 パーツ中 8 パーツ）、そうしたパーツは Player が反転した瞬間に**正へ戻ります**。反転設定の有無から符号を推測せず、実際に手元にある行列から読み取ってください。

鏡像は回転ではありません。姿勢をクォータニオンやオイラー角で表現するエンジンは鏡像を表現できないため、その種の「行列から回転を取り出す」ヘルパは行列式が負のとき **未定義** です。返るのは「精度の落ちた値」ではなく無意味な値です。行列をそのままノードの変換へ渡してよいのは、エンジンの変換型が完全なアフィン（符号付きスケール）を保持できる場合に限られます。なお、この行列に**せん断（skew）は含まれません**——SpriteStudio のせん断は、ローカル頂点段で適用されるパーツ単位の頂点変形であり、子に伝搬せず、Player が公開するパーツ姿勢にも現れません。

これが問題になるのは、Player がパーツの姿勢をゲームコードへ公開したときです（パーツに武器や当たり判定を取り付ける用途など）。その場合は次を守ってください。

* **平行移動は行列式によらず常に正確**です。
* 姿勢を取り出すときは、まず基底を proper に直し（いずれかの列を反転）、鏡像は別途——通常はその軸の負のスケールとして——持ち回ります。
* **どの列を反転するかは規約**であり、取りこぼしがどこに現れるかを決めます。符号を Y に載せる規約なら、左右方向の鏡像では抽出された回転に 180 度が余分に残り、上下方向では残りません。回転だけを受け取ってスケールを捨てる利用側は、この残差をそのまま引き継ぎ、かつ取り付けたオブジェクト自体は鏡像化されません。

最後の点は分解の仕方に由来する性質であって、この層で修正すべき欠陥ではありません。採用した規約とその帰結を、Player 側のドキュメントに明記してください。

### UV アニメーション・反転 (Flip) は取得データに反映済み

UV アニメーション（テクスチャ切り出し範囲の移動・回転・拡大縮小）や、パーツ／セルの反転（`flip_h` / `flip_v` / `img_flip_h` / `img_flip_v`）は、**すべて Brain 側で計算のうえ `local_vertices` / `local_uvs` に反映済み**です。Player は上記 SoA バッファをそのまま頂点座標・テクスチャ座標として渡すだけでよく、座標や UV を反転させる追加ロジックは必要ありません。

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

### バインドするテクスチャは `DrawBatch.texture_hash` だけで決まります

`texture_hash` は次のどちらかで、Player はその**両方を引けるテクスチャマップ**を用意しておく必要があります。

- **セルマップ名のハッシュ**（通常）— `ssab->cellmaps()[].name_hash`
- **テクスチャファイル名のハッシュ** — 作画でテクスチャ変更 (`TCHG`) が指定されたパーツ。`ssab->external_textures()[].name_hash` から名前を逆引きします。

上書きの有無はバッチキーに折り込まれているため、**このループを変える必要はありません**。一方で以下の 2 点に注意してください。

- `ss_runtime_get_cell_texture_hashes`（§1）が返すのは **セルが属するセルマップ**のハッシュで、テクスチャ変更を反映していません。バインドの判断には使わず、必ず `DrawBatch.texture_hash` を使ってください。
- テクスチャ変更やセルのオーバーライドによって **バッチの分割点・バッチ数はフレームごとに変わります**。「バッチ数は一定」という前提の最適化は避けてください。

詳細は [外部リソースとテクスチャの管理](../40_components/70_external_resources.ja.md) を参照してください。

> [!IMPORTANT]
> **この `draw_order` ループは描画（ジオメトリ発行）専用です。** Instance / Effect パーツの
> **`step`（子アニメ・パーティクルの時計の前進）を、この `draw_order` ループから駆動しないでください。**
> `Hide` 中のパーツは Brain が `draw_order` / `DrawBatch` から除外するため
> （[Mask の実装](../40_components/40_mask.ja.md) の補足参照）、「`draw_order` に載ったものだけ `step`」
> にすると、独立再生の子／エフェクトが非表示中に `step` されず時計が凍ります。**`step` は全 Instance /
> Effect パーツをパックのメタデータから列挙して毎フレーム漏れなく回し**、その結果を描画するときにだけ
> この `draw_order`（と `step` の戻り値 `visible`）を使ってください。詳細は
> [インスタンスパーツ](../40_components/10_instance.ja.md) /
> [エフェクトパーツ](../40_components/20_effect.ja.md) を参照。

### RenderMode についての注意点

`libssruntime` には、エンジンの描画思想に合わせた `RenderMode`（レンダリングモード）の概念があります。
初期化時に `ss_runtime_set_render_mode` で指定するモードによって、取得できる `FrameData` (FlatBuffers) の内容が変化します。

- **`RenderMode::Direct` (推奨)**: Godot の `RenderingServer` のように、C-API (SoAバッファ) 経由でデータを一括取得して直接描画するアプローチに最適化されたモードです。このモードでは FlatBuffers への冗長なデータ（`vertices`, `cells`, `deforms`）のシリアライズがスキップされ、CPU負荷とアロケーションが大幅に削減されます。上記のサンプルコードはこちらのモードを前提としています。
- **`RenderMode::SceneGraph`**: 各パーツごとにゲームエンジンのノード（Node2D や GameObject など）を生成し、階層構造（シーングラフ）を組んで描画をエンジン任せにするアプローチ用です。このモードでは FlatBuffers 内の `vertices` などの詳細データが全て出力されるため、それらを読み出してノードのプロパティを更新する用途に使います。

（※ Player 実装で `FrameData` から頂点やセルの情報を取得しようとして `null` が返ってくる場合は、`ss_runtime_set_render_mode` で `RenderMode_SceneGraph` を設定しているか確認してください。）

### ⚠️ FlatBuffers の per-element アクセサは「重い」：固定長 struct ベクタは生読みする

SoA getter (§1) でカバーされない属性 — 例えば **`parts`（パーツ状態フラグ / alpha 等）** や **`parts_color`（頂点カラー）** — は `FrameData` 内の FlatBuffers ベクタとして配送されます。これらを `frame_data->parts_color()->Get(i)->lt()->rgba()->r()` のような **per-element アクセサで1要素ずつ読むと、想像以上に高コスト**です（アクセサオブジェクト構築 + フィールド毎の境界チェック + バッファ間接参照。特に入れ子構造の `parts_color` が重い）。**C# / JS-TS / Python のような高水準ホストでは、このアクセサコストが多体シーンの支配的ボトルネックになり得ます。**

> [!WARNING]
> **実測 (Unity / C#, 100体):** `parts` + `parts_color` を per-element アクセサで毎フレーム読む処理が **約 7ms/frame**（frame-data 取り込みコストの ~72%）を占めていた。下記の「生読み」に変えたところ **約 0.27ms** に短縮（~26倍）。

**対策 — 固定長 struct ベクタは生ポインタから直接読む:**
`parts` / `parts_color` / `draw_batches` は FlatBuffers の **`struct`（固定レイアウト・連続配置）** なので、ベクタ先頭のオフセットを取り、**レイアウトが一致する生の構造体配列として直接読めます**（per-element アクセサを完全に回避）。

- **C#**: 生成コードの struct とバイト互換な `[StructLayout(LayoutKind.Explicit)]` ミラーを定義し、`(MirrorStruct*)(fb_ptr + vectorOffset)` にキャストして読む。
- **JS/TS (WASM)**: ベクタ先頭オフセットに `DataView` / 型付き配列ビューを張り、フィールドをオフセット直読みする。
- **Python (ctypes 等)**: ベクタ先頭のバイト列に対し `struct.iter_unpack`（固定長 struct ベクタ）/ `struct.unpack_from`（数値ベクタ）で一括デコードし、per-element アクセサを回避する。純 Python の生成アクセサ（属性アクセス + 境界チェックが積み重なる）はとりわけ高コストなため、`parts` / `parts_color` / `draw_batches` の生読み化は多体シーンで特に効果が大きい。
- **C++**: FlatBuffers struct はパック済み・連続配置なので、ベクタ先頭を配列として扱える（アクセサのオーバーヘッドは元々小さいが、ホットパスでは直読みが有利）。

> [!CAUTION]
> **よくある罠 — 「ゼロコピー ByteBuffer」では解決しない:** FlatBuffers の `ByteBuffer` をネイティブメモリに向ける（C# の `ENABLE_SPAN_T` 等）「ゼロコピー」化は **効きません（むしろ遅くなる）**。コストの正体は memcpy ではなく **per-element アクセサそのもの**だからです。消すべきはアクセサであって、コピーではありません。

**メンテ上の注意:** 生読みミラーは FlatBuffers の struct レイアウト（アライメント / パディング / エンディアン = リトルエンディアン）と完全一致が必須です。`.fbs` 再生成による無言の drift を防ぐため、**起動時に「生成アクセサ経由の読み」と「ミラー経由の読み」を突き合わせて全フィールド一致を assert** する自己検証を入れることを推奨します（sizeof だけでは同サイズの並べ替えを見逃すため、roundtrip 比較が確実）。

## 3. 描画パフォーマンスの最適化テクニック

近年のゲームエンジンやグラフィックライブラリでは、パーツごとに表示ノードやスプライトオブジェクトを大量に生成するのではなく、エンジンの低レイヤーな描画 API（バッチレンダリング機能など）を直接叩くことでパフォーマンスが劇的に向上します。

1. **ポインタの一括取得と保持 (DrawFrame パターン)**:
    毎フレームの描画ループの先頭で `ss_runtime_get_world_matrices` などのポインタを一括取得し、一時的な構造体やコンテキストに保持しておくことで、ループ内での不要な関数呼び出しやアロケーションを削減できます。

    **各言語環境でのポインタのラップ（ゼロコピー参照）の具体例**:
    C# や JS (WASM) などの高水準言語から FFI 経由で呼び出す場合、返されたポインタからマネージド配列を新しく確保してデータをコピーするのではなく、メモリを直接覗き込む（Borrow する）言語機能を使うことで、毎フレームのメモリコピーを完全に回避できます。

    * **C# の場合**: `unsafe` ブロック内の生ポインタや、`ReadOnlySpan<float>` 等を使用してラップします。
    * **JS/TS (WASM) の場合**: `wasmMemory.buffer` を使って `Float32Array` のビュー（のぞき窓）を張ります。
    
    ```typescript
    // 例: JS/TS 環境でアロケーションとコピーを防ぐパターン
    const ptr = context.get_world_matrices_buf();
    const count = context.get_parts_count();
    
    // 配列のコピーは行わず、WASM のメモリ空間を直接参照 (Borrow) する
    const worldMatrices = new Float32Array(wasmMemory.buffer, ptr, count * 16);
    ```

    > [!WARNING]
    > **マルチスレッドレンダラでのライフサイクルの罠 (Use-After-Free 回避)**
    > Unity の Job System (Burst) や Godot のマルチスレッド RenderingServer、Unreal Engine の RenderThread など、現代のエンジンでは「メインスレッドでのアニメーション更新」と「ワーカースレッドでの描画・ジオメトリ構築」が非同期に実行されることが一般的です。
    > SoA バッファ（C-API から取得した生ポインタ）をゼロコピーで描画スレッドに渡す設計にした場合、**「描画スレッドがポインタを読み取っている最中に、メインスレッドでプレイヤーの破棄処理が走り `ss_runtime_destroy` が呼ばれる」** という状態が発生すると、解放済みメモリへのアクセス（Segfault / アクセス違反）でアプリが確定クラッシュします。
    > これを回避するため、プレイヤー（コンポーネント）の破棄やデストラクタ (`Destroy` / `Dispose`) のタイミングでは、**「必ず実行中の描画ジョブやスレッド処理の完了を待機（Unity の `JobHandle.Complete()` 等での同期）してから、`ss_runtime_destroy` でネイティブメモリを解放する」** という厳格なライフサイクル管理を実装してください。

2. **描画リソースのプーリング**:
    バッチごとに必要となる低レイヤーの描画リソース（頂点バッファ、インデックスバッファ、描画コマンド用の中間オブジェクトなど）は毎フレーム生成・破棄せず、配列等にプールして再利用（使い回し）してください。
3. **ゼロアロケーション設計 (GC スパイクの回避)**:
    毎フレームの描画ループ内で新しくマネージド配列を確保 (`new`) したり、高レイヤーの UI コンポーネント (Unity の uGUI Canvas Rebuild など) に処理を任せると、深刻なパフォーマンス低下と GC スパイクを招きます。事前に確保したネイティブメモリ (`NativeArray` 等) を再利用し、頂点バッファを直接更新 (`SetVertexBufferData` 等) するゼロアロケーション設計を強く推奨します。
4. **頂点データの集約**:
    各パーツの計算済み頂点を、対象エンジンが直接受け付けるバッファ形式に詰め込み、描画コマンドを用いて一括で GPU へ送信します。
5. **描画順序の制御**:
    Z-index やエンジン独自のドロー順序制御機能を利用して、バッチ間の描画順（`draw_order`）を正しく反映させます。

## 4. マスクの実装

Player はマスクバッチをフックにして、エンジンの Stencil Buffer や専用のマスキング機能を開始・更新します。ここでは FrameData から何が読めるかだけを整理します。意味論・判定ルール・実装方式は [Mask の実装](../40_components/40_mask.ja.md) が正本です。

**`DrawBatchKind::Mask` が発行されるパーツ**（いずれも `count = 1`・`vertex_count` / `index_count` は `0`）:

- **マスクパーツ (`PartTypeMask`)**
- **`draw_as_mask` でマスク化された描画パーツ (Shape / Nines)** — マスクバッチと自分の色バッチが **同じ rank に両方** 出ます。pure マスクは色を描かないので、Player は色バッチの方をスキップしてください（[Mask の実装 §3-3](../40_components/40_mask.ja.md#3-3-共通の注意点)）

> [!IMPORTANT]
> **クリッピング (`mask_write`) には `DrawBatchKind::Mask` は発行されません。** クリッピングは通常の描画バッチとしてのみ出るため、このバッチ種別だけをフックにした Player は **クリッピングを丸ごと取りこぼします**。writer の確定は PartData を読んで Player 側で行ってください（[Mask の実装 §1-2](../40_components/40_mask.ja.md#1-2-パーツの役割と-mask_influence)）。

**`mask_indices`**: FrameData が持つ pure マスクのパーツ index 列です。**writer リストとしてそのまま使わないでください** — クリッピングを含まず、`Hide == true` のマスクパーツも落とさないため、`draw_order` と一致しません。

---
[目次に戻る](../README.ja.md)
