> **日本語** | [English](./10_initialization.md)

# ロードと初期化

ssab を Brain に渡して再生できる状態にするまでと、破棄の手順です。ssab のメモリを誰が持つか（borrow / copy）は、先に [Player の設計方針 §2](../20_design/20_architecture.ja.md#2-ssab-のメモリの持ち方) で決めておいてください。

## 1. リソースと Context

Brain は 2 つのオブジェクトを使います。

| オブジェクト | 中身 | 共有 |
| :--- | :--- | :--- |
| **リソース** | ssab のバイト列と、そこから読んだ読み取り専用のメタデータ | 同じ ssab を再生するすべての Context で共有できる |
| **Context** | 1 体分の再生状態と計算バッファ | 共有しない。再生する 1 体ごとに作る |

同じアニメーションを複数のキャラクターで再生する場合は、リソースを 1 つだけ作り、キャラクターごとに Context を作ってバインドしてください。

```cpp
#include "ssruntime.h"

// 1. リソースを作る。borrow は Player のメモリを参照し、copy は Brain の中に複製する
void* resource = ss_resource_create_borrow(ssab_data_ptr, ssab_data_len);
if (resource == nullptr) { /* ssab が空か、版が合わない */ }

// 2. Context を作る
void* context = ss_runtime_create();

// 3. リソースを Context にバインドする
ss_runtime_bind_resource(context, resource);
```

Context を 1 つしか使わない場合は、リソースの作成とバインドを 1 回で行う `ss_runtime_load_ssab_borrow` / `ss_runtime_load_ssab_copy` も使えます。

Rust から直接使う場合は、バイト列・メタデータ・Context の寿命を型で表す 3 段階になります。

```rust
// 1. バイト列を参照する（ssab_data_ptr: *const u8, ssab_data_len: usize）
let ssab_data = unsafe { ssruntime::SsabData::load_borrow(ssab_data_ptr, ssab_data_len).unwrap() };

// 2. メタデータを読む
let resource = ssruntime::AnimationResource::new(ssab_data.as_slice()).unwrap();

// 3. Context を作ってバインドする
let mut context = ssruntime::Context::new();
context.bind_resource(&resource);
```

### 1-1. 寿命と破棄の順

リソースは参照カウントで管理されています。Context はバインドしているリソースへの参照を持つので、`ss_resource_destroy` を先に呼んでも、そのリソースをバインドしている Context がある間は解放されません。**破棄の順は自由です。**

ただし、borrow で作ったリソースは Player のバイト列を直接読みます。**リソース自身と、それを参照するすべての Context・エフェクトスロットを破棄するまで、元のバイト列を解放しないでください。**

| オブジェクト | 所有者 | 破棄 |
| :--- | :--- | :--- |
| ssab のバイト列 | borrow では Player、copy では Brain | borrow では、リソースと、それを参照するすべての Context・エフェクトスロットを破棄した後に Player が解放する |
| リソース | Brain（参照カウント） | `ss_resource_destroy`。バインドしている Context がある間は残る |
| Context | Player | `ss_runtime_destroy` |

```cpp
ss_runtime_destroy(context);
ss_resource_destroy(resource);
```

## 2. ssab を渡す前の検証

Brain は ssab の構造を検証しません（[制限事項 §5](../10_introduction/20_limitations.ja.md#5-brain-は-ssab-を検証しない)）。ネットワークや利用者の入力から ssab を受け取る場合は、Brain に渡す前に次の確認をしてください。

- **ファイル識別子**: ssab の **4〜7 バイト目**には、ファイル識別子として ASCII の `"SSAB"` が入っています。この 4 バイトを比べれば、別の形式のファイルや空のファイルを取り違えて渡すことを防げます。
- **FlatBuffers の境界チェック**: 構造の妥当性まで確かめるには、Player の言語の FlatBuffers バインディングで生成した Verifier（C++ の `flatbuffers::Verifier`、C# の `SsAnimeBinaryVerify` など）に通してから渡します。

版が Brain と合わない ssab は、リソースの作成が失敗します（`ss_resource_create_*` は `nullptr` を返し、`ss_runtime_load_ssab_*` は `false` を返します）。

> [!NOTE]
> ssconverter が出力する ssab には、先頭にサイズが付いていません。FlatBuffers の `size-prefixed` 系の関数（C++ の `GetSizePrefixedSsAnimeBinary` など）ではなく、通常の関数（`GetSsAnimeBinary` など）を使ってください。

## 3. テクスチャの読み込み

描画のときにバインドするテクスチャは、`DrawBatch.texture_hash` で決まります（[描画ループ §2](./40_render_loop.ja.md#2-drawbatch-による描画)）。この値は、次の 2 種類のハッシュのどちらかです。

| `texture_hash` の値 | 対応する ssab のエントリ | 使われるとき |
| :--- | :--- | :--- |
| セルマップ名のハッシュ | `cellmaps()[].name_hash` | 通常 |
| テクスチャのファイル名のハッシュ | `external_textures()[].name_hash` | そのパーツに作画でテクスチャ変更（`TCHG`）が指定されているとき |

**`cellmaps()` と `external_textures()` の両方を読み込み、それぞれの `name_hash` をキーに 1 つのマップへ入れてください。** `external_textures()` を読み込まないと、テクスチャ変更が指定されたパーツが描画されません。テクスチャ変更の仕組みは [外部リソースとテクスチャ §2](../50_features/30_external_resources.ja.md#2-作画で指定されたテクスチャ変更) にあります。

### 3-1. ssab に入っているファイル参照の文字列

ssab に入っているファイル参照（テクスチャ・サウンド・フォントなど）の文字列は、ssconverter が次の規則で作ります。エンジンのアセットと対応付けるときは、この規則を前提にしてください。

- `sspj` / `ss*e`（`ssae` / `ssce` / `ssee` / `ssse`）/ `fnt` に書かれたファイルパスとファイル名は、**そのままの文字列**で入ります。大文字小文字や区切り文字の正規化、絶対パスへの変換は行いません。
- charmap（`.fnt`）のテクスチャのファイル名には、**sspj に書かれた charmap のパスのディレクトリ部分**が前に付きます。
    - 例: sspj に `font/font.fnt` と書かれ、`font.fnt` の中のテクスチャ名が `glyphs.png` なら、ssab の `name_texture` は **`font/glyphs.png`** になります。

### 3-2. 参照モードの ssab

- `cellmaps()[].image_path()` と `external_textures()[].name()` のパスを使い、エンジンの通常のアセット読み込みでテクスチャを読み込みます。
- パスは ssab からの相対パスです。Player で ssab のあるディレクトリと結合してください。ssab を読み込むときにそのディレクトリも受け取れるようにしておくと、解決が簡単になります。

```cpp
// Godot（C++）の例
void load_textures(const ss::format::SsAnimeBinary* binary, String base_dir) {
    for (int i = 0; i < binary->cellmaps()->size(); i++) {
        auto cellmap = binary->cellmaps()->Get(i);
        String image_path = base_dir.path_join(String::utf8(cellmap->image_path()->c_str()));
        _textures[cellmap->name_hash()] = ResourceLoader::get_singleton()->load(image_path);
    }
    // テクスチャ変更（TCHG）がバインドする先。同じマップに、テクスチャ名のハッシュで入れる
    if (binary->external_textures() != nullptr) {
        for (int i = 0; i < binary->external_textures()->size(); i++) {
            auto etexture = binary->external_textures()->Get(i);
            String image_path = base_dir.path_join(String::utf8(etexture->name()->c_str()));
            _textures[etexture->name_hash()] = ResourceLoader::get_singleton()->load(image_path);
        }
    }
}
```

### 3-3. 埋め込みモードの ssab

- `embedded_assets()` を走査し、画像のバイト列をエンジンの「画像データからテクスチャを作る」機能に渡します。外部テクスチャも埋め込まれています。
- `embedded_assets()` の `name_hash` は**ファイル名の**ハッシュです（セルマップでは `image_path`、外部テクスチャでは `name`）。セルマップ名のハッシュとは違うので、`cellmaps()` / `external_textures()` のエントリを経由して対応付けてください。

### 3-4. セルの索引

描画のたびに探さずに済むよう、初期化のときに次の索引を作っておくことを推奨します。

- セルマップごとに、`cells()` の `name_hash` から、そのセルの切り出し領域と原点を引ける表。

## 4. ログとエラー

### 4-1. ログのコールバック

配布している Brain は標準出力を使えません（[no_std 環境への統合](../20_design/40_no_std_integration.ja.md)）。Brain のエラーを受け取るために、初期化の最初にログのコールバックを登録してください。

```cpp
void my_log_callback(int level, const char* message) {
    // エンジンのログ関数（UE_LOG・Debug.Log・print_line など）に渡す
}

ss_runtime_set_log_callback(my_log_callback);
```

`level` は `SS_RUNTIME_LOG_ERROR`（1）/ `SS_RUNTIME_LOG_WARN`（2）/ `SS_RUNTIME_LOG_INFO`（3）のどれかです。いま発行されるのは `ERROR` だけですが、後から使われても `switch` の分岐が古くならないよう、残りの 2 つを予約しています。

失敗した C-API の呼び出しは、どの関数で失敗したかを前に付けた `ERROR` を 1 行出します（`ss_runtime_setup_animation: Animation not found`）。コールバックを登録していなければ文字列を組み立てないので、負荷はかかりません。

### 4-2. 失敗の理由

失敗する C-API の呼び出しは、戻り値（`false`・`-1`・`NaN` など）で失敗を伝えます。理由は、その直後に `ss_runtime_get_last_error(context)` で数値のコードとして取り、`ss_runtime_error_message(code)` で文言にします。コード `0` は成功です。ログは戻り値を確かめない呼び出し側のためのもので、`ss_runtime_get_last_error` と同じ失敗を報告します。

Player の公開 API で失敗をどう伝えるかは、[Player API 規約 §7](../20_design/30_api_conventions.ja.md#7-失敗の伝え方) に従ってください。

## 5. アニメーション名の一覧

ssab には複数のアニメーションが入っています。名前の一覧は ssab から読みます。

```cpp
auto binary = ss::format::GetSsAnimeBinary(ssab_data_ptr);
if (binary->animations()) {
    for (auto anime : *binary->animations()) {
        std::string name = anime->name()->str();
        // Player の公開 API（animationNames）で返す
    }
}
```


---
[目次に戻る](../README.ja.md)
