> **日本語** | [English](./40_render_loop.md)

# 描画ループ

`ss_runtime_get_frame_data` で計算させたフレームを描画する手順です。Brain は、描画する順番と、まとめて描けるパーツの組（DrawBatch）を決めて渡します。Player は FlatBuffers の FrameData で描画の計画を読み、頂点などの数値は SoA バッファからポインタで直接読みます。

FrameData の各フィールドの定義は [FrameData スキーマ](https://github.com/cri-middleware/SpriteStudio-SDK/blob/HEAD/libs/ssruntime/SCHEMA.ja.md) にあります。

## 1. SoA バッファ

`ss_runtime_get_frame_data` を呼ぶと、そのフレームの全パーツ分の値が SoA バッファに書かれます（`SceneGraph` モードで書かれないバッファは §3）。Player はポインタと長さを受け取り、パーツのインデックスで直接読みます。バッファは、次に `ss_runtime_get_frame_data` を呼ぶまで有効です。

| バッファ | 関数 | 1 パーツあたり | 内容 |
| :--- | :--- | :--- | :--- |
| ワールド行列 | `ss_runtime_get_world_matrices` | 16 float | 親子の変換を合成した 4x4 行列 |
| ローカル頂点 | `ss_runtime_get_local_vertices` | 10 float | 左上・右上・左下・右下・中心の 5 頂点の `(x, y)`。原点・反転・頂点変形・座標系の設定を適用済み。ワールド行列を掛けるだけで使える |
| ローカル UV | `ss_runtime_get_local_uvs` | 10 float | 上の 5 頂点に対応する UV。**セルマップのテクスチャのピクセル座標**なので、テクスチャの大きさで割って正規化する |
| セル情報 | `ss_runtime_get_cell_meta` | 6 float | `[原点 x, 原点 y, 幅, 高さ, 左, 上]`。左・上はセルマップのテクスチャ上の位置 |

セルを持たないパーツの値は 0 です。メッシュ・シェイプ・9 スライスは、専用のバッファで頂点を渡します（[メッシュ](../40_parts/20_mesh.ja.md)、[9 スライス・シェイプ・テキスト](../40_parts/30_special_parts.ja.md)）。

UV アニメーション（切り出し範囲の移動・回転・拡縮）と、パーツ・セルの反転（`flip_h` / `flip_v` / `img_flip_h` / `img_flip_v`）は、ローカル頂点とローカル UV に反映済みです。Player が座標や UV を反転する必要はありません。

### 1-1. ワールド行列は鏡像になりうる

ワールド行列は一般のアフィン変換で、「位置・回転・拡縮」の 3 つに分解できるとは限りません。行列式が負になるのは、姿勢に**奇数回**の鏡映がかかっているときです。鏡映の要因は 2 つあり、**パーツ自身か祖先の負のスケール**と、**Player が根元の変換に加える反転**（左右反転の機能など）です。

**この 2 つは掛け合わさり、偶数回なら打ち消し合います。** Player が反転しても、行列式が負になるとは限りません。反転は符号を**入れ替える**だけです。実際のデータでは、もともと鏡映のかかったパーツは珍しくありません（テスト用の `Ringo` では 69 パーツのうち 8 パーツ）。そうしたパーツは、Player が反転すると**正に戻ります**。反転の設定から符号を推測せず、手元の行列から読んでください。

鏡像は回転ではありません。姿勢をクォータニオンやオイラー角で表すエンジンは鏡像を表せないので、そうした「行列から回転を取り出す」関数は、行列式が負のとき**意味のない値**を返します。行列をそのままエンジンのノードの変換に渡してよいのは、エンジンの変換が符号付きの拡縮を含むアフィン変換を保持できる場合だけです。なお、この行列に**せん断（skew）は含まれません**。SpriteStudio のせん断はパーツごとの頂点の変形としてローカル頂点に適用され、子には伝わりません。

これが問題になるのは、Player がパーツの姿勢をゲームのコードへ公開するとき（パーツに武器や当たり判定を付ける用途など）です。その場合は次のようにしてください。

* **平行移動は、行列式にかかわらず常に正確です。**
* 姿勢を取り出すときは、まず基底の向きを正に直し（どれか 1 列の符号を反転し）、鏡像は別に（通常はその軸の負のスケールとして）持ちます。
* **どの列を反転するかは取り決めで、誤差がどこに出るかを決めます。** 符号を Y に載せる取り決めでは、左右方向の鏡像で取り出した回転に 180 度が余分に残り、上下方向では残りません。回転だけを受け取って拡縮を捨てる利用者は、この差をそのまま受け取り、取り付けたオブジェクト自体は鏡像になりません。

最後の点は分解の仕方から来る性質で、直すべき不具合ではありません。採用した取り決めとその結果を、Player のドキュメントに書いてください。

## 2. DrawBatch による描画

Brain は、描画する順に並べたパーツ（`draw_order`）と、それを同じテクスチャ・同じ合成モード・同じ種類で続けて描ける区間に切った DrawBatch の列（`draw_batches`）を出力します。Player は DrawBatch ごとに描画の状態を 1 回だけ切り替え、1 回のドローコールで描けます。続く DrawBatch を 1 回のドローコールにまとめることもできます（§4-3）。

```cpp
// 1. FrameData を計算させる
const uint8_t* fd_ptr = nullptr; uintptr_t fd_len = 0;
if (!ss_runtime_get_frame_data(ctx, frame_no, &fd_ptr, &fd_len)) return;   // 再生制御 §6-1
auto frame_data = ss::runtime::GetFrameData(fd_ptr);

// 2. SoA バッファを受け取る
const float *world_matrices, *local_vertices;
uintptr_t m_len, v_len;
ss_runtime_get_world_matrices(ctx, &world_matrices, &m_len);
ss_runtime_get_local_vertices(ctx, &local_vertices, &v_len);

// 3. DrawBatch ごとに描く
auto draw_order = frame_data->draw_order();
for (auto batch : *frame_data->draw_batches()) {
    apply_material(batch->texture_hash(), batch->blend_type());

    for (int k = 0; k < batch->count(); k++) {
        int p = draw_order->Get(batch->start_rank() + k);
        emit_geometry(&world_matrices[p * 16], &local_vertices[p * 10]);
    }
    flush_draw_call();
}
```

`apply_material` / `emit_geometry` / `flush_draw_call` は、エンジンの描画処理を表す実在しない関数です。この例は `Normal` の DrawBatch だけを扱っています。種類ごとの頂点の取り方は §2-2 にあります。

`draw_order` の値は `FrameData.parts` のインデックスで、SoA バッファもこのインデックスで引きます。ssab のパーツ定義を引くときは `FrameData.parts[p].part_index` を使ってください。

### 2-1. バインドするテクスチャ

**バインドするテクスチャは `DrawBatch.texture_hash` だけで決まります。** 値はセルマップ名のハッシュか、作画でテクスチャ変更（`TCHG`）が指定されたパーツではテクスチャのファイル名のハッシュです。両方を引けるマップの作り方は [ロードと初期化 §3](./10_initialization.ja.md#3-テクスチャの読み込み) にあります。

- `FrameData.cells` のセルマップ名のハッシュは、**セルが属するセルマップ**のもので、テクスチャ変更を反映していません。バインドには使わないでください。
- テクスチャ変更やセルのオーバーライドによって、**DrawBatch の区切り方と数はフレームごとに変わります。** 「DrawBatch の数は一定」という前提の最適化はしないでください。

### 2-2. DrawBatch の種類

`DrawBatch.kind` で、その区間のパーツの頂点をどこから取るかが決まります。

| `kind` | 頂点の取り方 | 説明のあるページ |
| :--- | :--- | :--- |
| `Normal` | ローカル頂点（§1）とワールド行列 | このページ、[パーツカラー](../40_parts/10_parts_color.ja.md) |
| `Mesh` | メッシュ用の SoA バッファ | [メッシュ](../40_parts/20_mesh.ja.md) |
| `Shape` | シェイプ用の SoA バッファ | [9 スライス・シェイプ・テキスト §2](../40_parts/30_special_parts.ja.md#2-シェイプ) |
| `Nines` | 9 スライス用の SoA バッファ | [9 スライス・シェイプ・テキスト §1](../40_parts/30_special_parts.ja.md#1-9-スライス) |
| `Text` | 頂点は無い。Player が ssab からグリフを組み立てる | [9 スライス・シェイプ・テキスト §3](../40_parts/30_special_parts.ja.md#3-テキスト) |
| `Mask` | ステンシル（または被覆）にだけ書く。色は描かない | [マスク](../40_parts/40_mask.ja.md) |
| `Instance` | その位置に、子アニメーションを描く | [インスタンスパーツ](../40_parts/50_instance.ja.md) |
| `Effect` | その位置に、エフェクトのパーティクルを描く | [エフェクトパーツ](../40_parts/60_effect.ja.md) |

`DrawBatch.flags` のビット `0x01` は、その区間のパーツの `mask_influence` が `1` であることを表します。他のビットは予約されているので、無視してください。

### 2-3. 子の時計を `draw_order` で進めない

`draw_order` は**描画のためだけ**の列です。非表示のパーツは `draw_order` と DrawBatch に入りません（例外は `draw_as_mask` でマスクにした描画パーツのマスクの DrawBatch だけです。[マスク §3](../40_parts/40_mask.ja.md#3-framedata-から読むもの)）。インスタンスパーツとエフェクトパーツの子の時計（スロットの `step`）を `draw_order` を走査しながら進めると、非表示の間は子が進まず、独立再生の子の時計が止まります。

**子の時計は、ssab からすべてのインスタンスパーツとエフェクトパーツを列挙して、毎フレーム漏れなく進めてください。** `draw_order` と `step` の戻り値の `visible` は、描くかどうかと描く順番を決めるためだけに使います（[インスタンスパーツ §2](../40_parts/50_instance.ja.md#2-スロットで子を進める)）。

## 3. RenderMode

`ss_runtime_set_render_mode` で、FrameData に何を載せるかを選びます。

| モード | 値 | 内容 |
| :--- | :--- | :--- |
| **`Direct`**（既定・推奨） | `0` | 頂点などの数値は SoA バッファだけで渡す。FrameData に `cells` / `vertices` / `deforms` を載せないので、計算とメモリ確保が少ない。このページの例はこのモードを前提にしている |
| **`SceneGraph`** | `1` | パーツごとにエンジンのノードを作る Player 向け。FrameData に `cells` / `vertices` / `deforms` をすべて載せる。ワールド行列を計算しないので、`ss_runtime_get_world_matrices` の値は使えない。メッシュの頂点も計算せず、メッシュの SoA バッファは空になる |

`parts_color` はどちらのモードでも FrameData に載ります。FrameData の `cells` や `vertices` が `null` になる場合は、`Direct` モードで読んでいないかを確かめてください。

## 4. 性能

エンジンの表示ノードやスプライトをパーツごとに作るより、エンジンの低レベルの描画 API を直接使うほうが、ずっと速く描けます。

1. **ポインタは 1 フレームに 1 回受け取る。** 描画ループの前に SoA バッファのポインタを受け取って持っておき、ループの中で関数を呼び直さないようにします。
2. **コピーせずに読む。** C# や JavaScript のような高水準の言語でも、返されたポインタから配列を作ってコピーせず、その言語の機能でメモリを直接読みます。C# では `unsafe` のポインタや `ReadOnlySpan<float>`、WebAssembly では `memory.buffer` の上に `Float32Array` のビューを作ります。

    ```typescript
    const worldMatrices = new Float32Array(
        memory.buffer, ctx.get_world_matrices_buf(), ctx.get_world_matrices_len());
    ```

    WebAssembly のメモリが拡張されると、ビューは無効になります。ビューは使う直前に作り直してください。

3. **描画のリソースを使い回す。** 頂点バッファ・インデックスバッファ・ドローコールの中間オブジェクトは、毎フレーム作って捨てず、プールして再利用します。
4. **毎フレームのメモリ確保をしない。** 描画ループの中でマネージドな配列を `new` したり、重い UI の仕組み（Unity の uGUI の Canvas の再構築など）に任せたりすると、性能が落ち、GC による停止が起きます。確保済みのネイティブのメモリ（`NativeArray` など）を再利用し、頂点バッファを直接更新してください（`SetVertexBufferData` など）。

Brain も、アニメーションを 2 回再生し終えたあとは同じ決まりを守ります。最初の再生ではバッファが伸びます。`ss_runtime_setup_animation` がパーツで決まる分を確保し、残り（FrameData・DrawBatch・メッシュの変形・エフェクトのパーティクル）は、再生中のいちばん大きなフレームに合わせて伸びます。FrameData のバッファは、その次の呼び出しでそのフレームに合う大きさに作り直されます。この作り直しは、2 回目の再生の頭に来ることがあります。それ以降は、`ss_runtime_update`・`ss_runtime_get_frame_data`・`ss_effect_slot_step`・`ss_instance_slot_step` はメモリを確保しません。そのインスタンスで初めて起きることがあると、ここからやり直しになります。[ミキシング](../50_features/10_animation_mixing.ja.md)、[パーツオーバーライド](../50_features/20_part_overrides.ja.md)、それまでより多くのフレームを一度に進める `ss_runtime_update`、それまでより多くのパーティクルが同時に存在するエフェクトです。再生中に確保させたくない場面では、読み込み中などにそのアニメーションを 2 回再生しておいてください。

### 4-1. マルチスレッドの描画と破棄

Unity の Job System、Godot のマルチスレッドの RenderingServer、Unreal Engine の RenderThread のように、アニメーションの更新と、頂点の組み立てや描画を別のスレッドで行うエンジンでは、SoA バッファのポインタをそのまま描画のスレッドに渡すことがあります。

**描画のスレッドがポインタを読んでいる間に `ss_runtime_destroy` を呼ぶと、解放済みのメモリを読んでクラッシュします。** Player を破棄するとき（`Destroy` / `Dispose` / デストラクタ）は、実行中の描画ジョブやスレッドの完了を待ってから（Unity の `JobHandle.Complete()` など）、`ss_runtime_destroy` を呼んでください。

### 4-2. FrameData のベクタを直接読む

SoA バッファに無い値、たとえば **`parts`**（パーツの状態）と **`parts_color`**（頂点の色）は、FrameData の FlatBuffers のベクタで渡されます。これを `frame_data->parts_color()->Get(i)->lt().rgba().r()` のように要素ごとのアクセサで読むと、思った以上に重くなります。アクセサのオブジェクトを作り、フィールドごとに境界を確かめ、バッファを間接参照するためです。**C#・JavaScript・Python のような高水準の言語では、ここがパーツの多い場面で一番重い処理になりえます。**

> [!WARNING]
> **実測（Unity / C#、100 体）:** `parts` と `parts_color` を要素ごとのアクセサで毎フレーム読む処理が、**約 7ms / フレーム**（FrameData を取り込む処理の約 72%）を占めていました。下の方法に変えると **約 0.27ms** になりました（約 26 倍）。

`parts` / `parts_color` / `draw_batches` は FlatBuffers の **`struct`**（固定の配置で連続して並ぶ）なので、ベクタの先頭の位置を取り、**同じ配置の構造体の配列として直接読めます。**

- **C#**: 生成されたコードの struct とバイト単位で同じ配置の `[StructLayout(LayoutKind.Explicit)]` の構造体を定義し、`(MirrorStruct*)(fb_ptr + vectorOffset)` にキャストして読む。
- **JavaScript / TypeScript**: ベクタの先頭に `DataView` か型付き配列のビューを作り、フィールドを位置から直接読む。
- **Python（ctypes など）**: ベクタの先頭からのバイト列を `struct.iter_unpack`（構造体のベクタ）や `struct.unpack_from`（数値のベクタ）でまとめて読む。Python で生成したアクセサは特に重いので、効果が大きい。
- **C++**: FlatBuffers の struct は詰めて連続に並ぶので、ベクタの先頭を配列として扱える。アクセサのコストはもともと小さいが、頻繁に通る経路では直接読むほうが速い。

> [!CAUTION]
> **FlatBuffers の `ByteBuffer` をネイティブのメモリに向ける「ゼロコピー」では解決しません。** 重いのはコピーではなく、要素ごとのアクセサそのものだからです。コピーを省いても、値を 1 つ読むたびのアクセサの手間は残ります（Unity の Player では、アニメーションが少ないときにかえって遅くなることがありました）。

直接読むための構造体は、FlatBuffers の struct の配置（境界・詰め物・リトルエンディアン）と完全に一致していなければなりません。`.fbs` から生成し直したときに配置がずれても気づけるよう、**起動時に、生成したアクセサで読んだ値と直接読んだ値を全フィールドについて比べる**検証を入れることを推奨します。`sizeof` を比べるだけでは、同じ大きさのままフィールドの順が入れ替わった場合を見逃します。

### 4-3. 続く DrawBatch をまとめて描く

DrawBatch は、テクスチャと合成モードが同じでも、種類（`kind`）が変わるところで切れます。`kind` が決めるのは頂点の取り方だけなので（§2-2）、`Normal` と `Mesh` が入り混じるキャラクターでは、1 回で描ける頂点がいくつもの DrawBatch に分かれて届きます。

**続く DrawBatch どうしでテクスチャと合成モードが同じなら、頂点を 1 つのバッファにつなげて、1 回のドローコールで描くことを推奨します。**

> [!NOTE]
> **実測（Web の Player、WebGL）:** SDK のテストと SpriteStudio のサンプルの 191 アニメーションで、ドローコールの合計が約 20% 減りました。SDK のテストの Ringo は、8 個の DrawBatch が 1 回のドローコールになりました。

- まとめるのは**続く DrawBatch だけ**にしてください。描く順を入れ替えると、重なったパーツの合成の結果が変わります。
- 描画の状態を変えるものが間に来たら、そこまでを描いてください。インスタンスパーツの子アニメーションや、シェーダのアトリビュートを持つパーツなどです。マスクの判定を受けるパーツは、ステンシル（または被覆）の判定の設定が同じ間だけつなげられます。
- エフェクトのパーティクルは、続く `Effect` の DrawBatch の間でまとめられます（[エフェクトパーツ §4](../40_parts/60_effect.ja.md#4-コマンドをまとめて描く)）。
- インスタンスの中のクリッピングを持ち越す Player は、[マスク §4-2](../40_parts/40_mask.ja.md#4-2-ステンシルを元に戻す) の、閉じる描画の隙間をまたいで結合しない注意も守ってください。

---
[目次に戻る](../README.ja.md)
