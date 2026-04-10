# 実装フェーズ 1：初期化 (Initialization)

Player の基本機能の実装において、初期化時に行うべき処理と、メモリ管理の戦略について記載します。

## 1. libssruntime の初期化 (Resource と Context の分離)
最新のアーキテクチャでは、バイナリデータの所有権(`SsabData`)と、パース済みの読み取り専用メタデータ(`AnimationResource`)、および再生状態・計算バッファ(`Context`)が安全に分離されています。
同一アニメーションを複数キャラクターで再生する場合、リソースは 1 つだけ作成し、キャラクターごとにコンテキストを作成してリソースをバインド(`bind_resource`)することでパフォーマンスが向上します。

### C-API (C/C++) を使用する場合
C-API では `SsabData` と `AnimationResource` は内部的に `FfiResource` として隠蔽されているため、以下の2ステップで初期化します。

```cpp
#include "runtime/ssruntime.h"

// 1. リソース（データの所有権とパース済みメタデータ）の作成
// ss_resource_create_borrow または ss_resource_create_copy を使用
void* resource = ss_resource_create_borrow(ssab_data_ptr, ssab_data_len);

// 2. コンテキスト（再生状態）の作成
void* context = ss_runtime_create();

// 3. リソースをコンテキストにバインド
ss_runtime_bind_resource(context, resource);
```

### Rust を直接使用する場合
Rust のエコシステム（Bevy や Amethyst など）から直接利用する場合は、ライフタイムを安全に管理するため明示的な 3 ステップになります。

```rust
// 1. データの所有権 (SsabData) の確立
let ssab_data = ssruntime::SsabData::load_copy(ssab_data_ptr).unwrap();

// 2. メタデータ (AnimationResource) の作成
let resource = ssruntime::AnimationResource::new(ssab_data.as_slice()).unwrap();

// 3. コンテキストの作成とバインド
let mut context = ssruntime::Context::new();
context.bind_resource(&resource);
```

### ⚠️ 重要な警告 (メモリ管理とライフタイム)
**必ず Context を破棄してから Resource を破棄してください。**
C/C++ 等の Player 実装において、コンテキストより先にリソースを破棄（またはガベージコレクションによって回収）してしまうと、コンテキスト内で保持しているリソースへの参照がダングリングポインタ（Use-after-free）となりクラッシュの原因となります。

## 2. ssab ファイルのロードと libssruntime への渡し方
ネイティブライブラリである libssruntime へ ssab データを渡す方法は、利用するプログラミング言語のポインタ操作能力に応じて 3 つの運用パターンがあります。

### パターン A：ポインタを直接渡せる場合（推奨）
- **対象言語:** C++, C# (unsafe) など
- **運用:** ssab データのオーナー（メモリ管理責任者）は Player 側になります。ゲームエンジン側のアセット管理（Godot の Resource や Unity の ScriptableObject 等）でバイナリを保持し続ける場合に最適です。
- **実装:** `ss_resource_create_borrow` を使用します。
- **メリット:** メモリコピーが発生せず、最も効率的です。

```cpp
// Godot (C++) での実装例
// Resource として保持している PackedByteArray からポインタを取得して渡す
void* resource = ss_resource_create_borrow(
    ssab_resource->get_data_ptr(), 
    ssab_resource->get_data_size()
);
```

### パターン B：ポインタ経由で参照のみ可能な場合
- **対象言語:** Kotlin, TypeScript(JavaScript) など
- **運用:** ssab データのオーナーを libssruntime 側に移譲します。Player はライブラリが所有するデータを参照して利用します。
- **実装:** 
    1. `ss_resource_create_copy` を実行してデータをライブラリ側にコピーします。
    2. コピー完了後、Player 側の元のデータは解放します。

*(※ 従来の後方互換用APIとして `ss_runtime_load_ssab_borrow` や `ss_runtime_load_ssab_copy` も引き続き利用可能ですが、新規実装では Resource と Context を明示的に分けることを推奨します。)*

### パターン C：ポインタの受け渡しが困難な場合
- **対象言語:** 一部の高レベル言語
- **運用:** Player 側と libssruntime の両方が ssab データのコピーを保持します。
- **実装:** `ss_resource_create_copy` を実行し、Player 側でも同じデータを保持し続けます。
- **デメリット:** メモリ消費が 2 倍になるため、可能な限り避けてください。

## 3. アセット（テクスチャ）のロードと紐付け
ssab 内には「どのセルマップ（画像）を使用するか」という情報が含まれています。Player は設計方針で選択した `ssab` のタイプに合わせて、テクスチャを準備する必要があります。

### Reference (参照方式) の場合
- **ロード元**: 外部ファイル（PNG 等）。
- **手順**: `cellmap->image_path()` から取得できるファイルパスを基に、エンジンの標準的なアセットロード機構を用いてテクスチャを読み込みます。
- **注意点**: `image_path` は通常 `.sspj` や `.ssab` からの相対パスです。Player 側でベースディレクトリと結合して絶対パス（またはエンジン内のリソースパス）を生成する必要があります。

```cpp
// Godot (C++) での実装例
void loadTextures(const ss::format::SsAnimeBinary* binary, String base_dir) {
    for (int i = 0; i < binary->cellmaps()->size(); i++) {
        auto cellmap = binary->cellmaps()->Get(i);
        // パスの解決
        String image_path = base_dir.path_join(String::utf8(cellmap->image_path()->c_str()));
        // エンジンの機能でロード
        Ref<Texture2D> texture = ResourceLoader::get_singleton()->load(image_path);
        // ハッシュ値で管理
        _textures[cellmap->name_hash()] = texture;
    }
}
```

### Embedded (埋め込み方式) の場合
- **ロード元**: ssab バイナリ内部。
- **手順**: `ssab->embedded_assets()` を走査し、各セルマップに対応するバイナリデータを直接取得します。このメモリ上のデータを、エンジンの「画像データからのテクスチャ作成」機能に渡して読み込みます。

### 共通の処理：セル情報のインデックス化
描画パフォーマンスを向上させるため、初期化時に以下の情報をキャッシュしておくことを強く推奨します。

- **CellMap のインデックス化**: `ssab->cellmaps()` の並び順や `name_hash()` をキーとして、ロードしたテクスチャオブジェクトを紐付けます。
- **セルの高速検索**: 各 CellMap 内の `cells()` を走査し、`name_hash()` をキーとして、そのセルの切り出し矩形（UV）や原点（Pivot）情報を即座に引けるようにしておきます。

## 4. 再生するアニメーション情報の管理
ssab データには複数のアニメーションが格納されています。

```cpp
// ssab 内のアニメーション名一覧を取得する例
auto ss_anime_binary = ss::format::GetSsAnimeBinary(binary_ptr);
if (ss_anime_binary->animations()) {
    for (auto anime : *ss_anime_binary->animations()) {
        std::string name = anime->name()->str();
        // この名前を Player の外部 API（Play("name") など）で利用できるようにする
    }
}
```

## 5. 後処理（インスタンス破棄時）
メモリリークやクラッシュ（Use-after-free）を防ぐため、Player の破棄時には**必ず Context を先に破棄し、その後に Resource を破棄**してください。

```cpp
// 1. 必ず先にコンテキスト（再生状態）を破棄する
ss_runtime_destroy(context);

// 2. その後、誰も参照しなくなったリソースを破棄する
ss_resource_destroy(resource);
```