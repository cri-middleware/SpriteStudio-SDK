> **日本語** | [English](./10_troubleshooting.md)

# トラブルシューティング

Player を作っているときによく起きる問題と、その原因です。

## 1. 症状から探す

| 症状 | 原因 | 説明のあるページ |
| :--- | :--- | :--- |
| アニメーションが動かない | `ss_runtime_play` を呼んでいない。セットアップしただけでは `update` で進まない | [再生制御 §2](../30_core/20_playback_control.ja.md#2-セットアップと再生) |
| アニメーションが先頭から進まない / 有限のループが終わらない | 同じアニメーションを毎フレームセットアップしている | [Player API 規約 §8-2](../20_design/30_api_conventions.ja.md#8-2-選択済みのアニメーションを指定されたら何もしない) |
| 一時停止を解くと先頭に戻る | `resume` の代わりに `play` を呼んでいる | [再生制御 §4-1](../30_core/20_playback_control.ja.md#4-1-各操作の意味) |
| 終了の通知が毎フレーム来る / 来ない | `is_finished` の立ち上がりを取っていない | [再生制御 §7](../30_core/20_playback_control.ja.md#7-ループと終了の検知) |
| 1 回の更新で 1 フレームに満たない時間しか進まない環境で、アニメーションが止まる | 状態として持つフレーム番号を丸めて書き戻している | [再生制御 §6-2](../30_core/20_playback_control.ja.md#6-2-小数を捨てない) |
| ループの境目で全パーツが一瞬消える | `ss_runtime_get_frame_data` に範囲外のフレーム番号を渡している | [再生制御 §6-1](../30_core/20_playback_control.ja.md#6-1-受け付けるフレーム番号) |
| フレーム 0 のイベントや音が、ときどき出ない | イベントの処理を、描画するかどうかの条件で止めている | [イベント §3-1](../30_core/30_events.ja.md#3-1-開始フレームのイベント) |
| テクスチャ変更のあるパーツが描かれない | `external_textures()` を読み込んでいない | [ロードと初期化 §3](../30_core/10_initialization.ja.md#3-テクスチャの読み込み) |
| 表示が上下に反転している / 二重に反転している | 座標系の設定と、Player 側の反転が重なっている | [Player の設計方針 §5](../20_design/20_architecture.ja.md#5-座標系) |
| 非表示から戻ったエフェクトや子アニメーションが遅れる / 止まっている | 子の時計を `draw_order` から進めている | [インスタンスパーツ §2](../40_parts/50_instance.ja.md#2-スロットで子を進める) |
| 再生し直すと、独立再生のエフェクトや子が始まらない | `play` のときにスロットを初期化していない | [インスタンスパーツ §6](../40_parts/50_instance.ja.md#6-再生し直すとき) |
| 子アニメーションやエフェクトが、親のフェードに合わせて消えない | インスタンスパーツ・エフェクトパーツの不透明度を掛けていない | [インスタンスパーツ §5](../40_parts/50_instance.ja.md#5-子を描く) / [エフェクトパーツ §3](../40_parts/60_effect.ja.md#3-effectdrawplan-を描く) |
| クリッピングが効かない | `Mask` の DrawBatch だけを見て writer を決めている | [マスク §3](../40_parts/40_mask.ja.md#3-framedata-から読むもの) |
| マスクの重なった部分が、描く順番で変わる | 加算と反転に同じビットを使っている | [マスク §4-4](../40_parts/40_mask.ja.md#4-4-書き込み方ごとにビットのプレーンを分ける必須) |
| 同じフレームの別のアニメーションのマスクがおかしい | クリッピングを打ち消していない | [マスク §4-2](../40_parts/40_mask.ja.md#4-2-ステンシルを元に戻す) |
| 着色が二重に暗くなる | パーツカラーのアルファに、不透明度をもう一度掛けている | [パーツカラー §3](../40_parts/10_parts_color.ja.md#3-パーツカラーを読む) |
| オーバーライドが効かないように見える | フレーム番号でキャッシュしている | [パーツオーバーライド §5-1](../50_features/20_part_overrides.ja.md#5-1-フレーム番号でキャッシュしていると効かない) |
| 破棄するとクラッシュする | 描画のスレッドが SoA バッファを読んでいる間に Context を破棄している | [描画ループ §4-1](../30_core/40_render_loop.ja.md#4-1-マルチスレッドの描画と破棄) |
| 名前で指定したアニメーションやパーツが見つからない | 文字列のハッシュを自分で計算していて、Brain と値が違う | §2 |

## 2. 名前のハッシュが合わない

**症状**: エラーは出ないのに別のアニメーションが再生される、インスタンスパーツの参照先が見つからず何も描かれない。

**原因**: アニメーション名やパーツ名のハッシュ（`uint32`）を、移植先の言語で FNV-1a などを**自分で実装して**計算し、Brain の `ss_runtime_hash_string` と 1 bit でも違う値になっている。

**対処**: 文字列のハッシュは自分で実装せず、**必ず `ss_runtime_hash_string(const char* s)` を呼んで**作ってください。

## 3. Windows でのリンクエラー（MSVC）

### 3-1. LNK2019 / LNK2001: 未解決の外部シンボル

**症状**: `ssconverter.lib` や `ssruntime.lib` を Windows の C++ プロジェクトで使うと、リンクで `LNK2019` / `LNK2001` が出る。

**原因**: Rust の標準ライブラリが使う Windows API のシンボルが、最終的なバイナリのリンクで解決されていない。

**対処**: リンクの設定に、次の Windows 標準ライブラリを加えてください。

- **Userenv.lib**
- **Bcrypt.lib**
- **Ntdll.lib**
- **Ws2_32.lib**

- **Visual Studio**: プロジェクトのプロパティ → リンカー → 入力 → 追加の依存ファイル に上の 4 つを加える
- **SCons**: `env.Append(LIBS=["Userenv", "Bcrypt", "Ntdll", "Ws2_32"])`
- **CMake**: `target_link_libraries(<target> PRIVATE Userenv Bcrypt Ntdll Ws2_32)`

### 3-2. LNK2005 / LNK1169: シンボルの重複（`ssruntime.lib` と `ssconverter.lib` を両方リンクする）

**症状**: 2 つの静的ライブラリを同じバイナリ（Godot のモジュールや GDExtension の Windows 版など）にリンクすると、`link.exe` が `LNK2005` / `LNK1169` で失敗する。

```
ssruntime.windows.editor.x86_64.lib(...) : error LNK2005: ___rust_alloc は既に ssconverter.windows.editor.x86_64.lib(...) で定義されています。
godot.windows.editor.x86_64.exe : fatal error LNK1169: 1 つ以上の複数定義のシンボルが見つかりました。
```

**原因**: Rust の静的ライブラリは、それぞれが Rust のランタイム（アロケータやパニックの関数）を含むので、2 つの `.lib` に `__rust_alloc` / `rust_begin_unwind` / `rust_eh_personality` などが重複します。MSVC のリンカは重複をエラーにします。macOS は [§4-1](#4-1-シンボルの重複libssruntimea-と-libssconvertera-を両方リンクする) を見てください。

**対処**: 最終的なバイナリのリンクのオプションに `/FORCE:MULTIPLE` を加えます。

```python
# SCons（Godot のモジュール / GDExtension）の例
if env["platform"] == "windows":
    env.Append(LINKFLAGS=["/FORCE:MULTIPLE", "/IGNORE:4006"])
```

- `/FORCE:MULTIPLE`: `LNK2005` を警告（`LNK4006`）にしてリンクを続ける
- `/IGNORE:4006`: その警告も出さない（任意）

#### `/WX` を使っている場合（`LNK4088`）

godot-cpp などが `/WX`（警告をエラーにする）を有効にしていると、`/FORCE:MULTIPLE` で必ず出る `LNK4088`（「/FORCE オプションによってイメージが生成されています」）が `LNK1218` になってビルドが失敗します。

```
warning LNK4088: /FORCE オプションによってイメージが生成されています...
LINK : error LNK1218: 警告がエラーとして扱われました。
```

`LNK4088` は、MSVC が `/IGNORE` で消せない警告の 1 つで、**`/IGNORE:4088` は効きません。** かわりに `/WX:NO` をリンカに渡して、リンクのときだけ警告をエラーにしないようにしてください。

```python
if env["platform"] == "windows":
    env.Append(LINKFLAGS=["/FORCE:MULTIPLE", "/IGNORE:4006", "/WX:NO"])
```

`/WX:NO` は**リンカだけ**に効くので、コンパイラ（`CCFLAGS` / `CXXFLAGS`）の `/WX` はそのままです。ソースコードの警告は引き続きエラーになります。

> [!IMPORTANT]
> `/WX:NO` を入れると、**リンクのすべての警告が通ってしまいます。** リンクのエラー（`LNK2005` など）をすべて片付けてから、最後にこのオプションを加えてください。

#### 動作への影響

`panic = "abort"` でビルドしているので `rust_begin_unwind` / `rust_eh_personality` は実際には使われず、アロケータもどちらも MSVCRT の `malloc` / `free` を呼ぶので同じ動きです。どちらの定義が使われても、動作は変わりません。

## 4. macOS でのリンクのエラー（ld）

### 4-1. シンボルの重複（`libssruntime.a` と `libssconverter.a` を両方リンクする）

**症状**: 2 つの静的ライブラリを同じバイナリにリンクすると、`ld` が失敗する。

```
duplicate symbol '_rust_eh_personality' in:
    .../libssruntime.a[2](...)
    .../libssconverter.a[2](...)
ld: 1 duplicate symbols
```

**原因**: §3-2 と同じです。

**対処**: 最終的なバイナリのリンクのオプションに `-Wl,-dead_strip` を加えるか、どちらか一方を動的ライブラリとしてリンクします。`-dead_strip` を付けると、`ld` は重複を警告にしてリンクを完了します。動作への影響は §3-2 のとおりありません。

---
[目次に戻る](../README.ja.md)
