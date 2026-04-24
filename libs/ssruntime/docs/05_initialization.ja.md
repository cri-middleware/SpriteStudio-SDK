# 実装フェーズ 1：初期化 (Initialization)

Player の基本機能の実装において、初期化時に行うべき処理と、メモリ管理の戦略について記載します。

## 1. libssruntime の初期化 (Resource と Context の分離)
最新のアーキテクチャでは、バイナリデータの所有権(`SsabData`)と、パース済みの読み取り専用メタデータ(`AnimationResource`)、および再生状態・計算バッファ(`Context`)が安全に分離されています。
同一アニメーションを複数キャラクターで再生する場合、リソースは 1 つだけ作成し、キャラクターごとにコンテキストを作成してリソースをバインド(`bind_resource`)することでパフォーマンスが向上します。

### 1.5. libssruntime の no_std 対応とメモリ管理
`libssruntime` はフットプリント削減とポータビリティ向上のため、Rust の標準ライブラリ (`std`) に依存しない `no_std` 環境を完全にサポートしています。
デスクトップでのビルド安定性を考慮し、デフォルトでは `std` が有効になっていますが、iOS/Android 向けのリリースビルドや組み込み環境向けには `no_std` 構成（`--no-default-features`）でビルドされます。

C-API (`cdylib` / `staticlib`) を `no_std` でビルドする際は、内部の動的メモリ確保 (ヒープアロケーション) を行うためにグローバルアロケータが必要です。SDK では `libc` の `malloc` / `free` を利用する `libc_alloc` 機能を用意しています。

#### デバッグログ出力の移譲
`no_std` 環境では標準出力 (`println!`, `eprintln!`) が利用できません。libssruntime 内部からのエラーや警告ログを受け取るには、初期化時にログコールバックを登録する必要があります。

```cpp
#include "runtime/ssruntime.h"

// ログを受け取るコールバック関数
void my_log_callback(int level, const char* message) {
    // 例: printf("libssruntime [%d]: %s\n", level, message);
    // ゲームエンジンのログ出力関数 (UE_LOG, Debug.Log, print_line など) に渡す
}

// 初期化の最初にコールバックを登録する
ss_runtime_set_log_callback(my_log_callback);
```

### 1.6. データバリデーション (Player 側の責任)
パフォーマンスとバイナリサイズの最適化のため、`libssruntime` は入力された `.ssab` データの構造的なバリデーション（FlatBuffers の境界チェック等）を行いません。
不正なデータや破損したデータを `ss_resource_create_*` や `ss_runtime_load_ssab_*` に渡すと、未定義動作（Out-of-bounds Read によるクラッシュ等）を引き起こす可能性があります。
Player の実装者は、入力データが信頼できるソースからのものであることを保証するか、必要に応じて事前にデータ検証（チェックサムや FlatBuffers バリデータ等）を行う責任があります。

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
// 1. データの参照 (SsabData) の確立
// ssab_data_ptr: *const u8, ssab_data_len: usize
let ssab_data = unsafe { 
    ssruntime::SsabData::load_borrow(ssab_data_ptr, ssab_data_len).unwrap() 
};

// 2. メタデータ (AnimationResource) の作成
let resource = ssruntime::AnimationResource::new(ssab_data.as_slice()).unwrap();

// 3. コンテキストの作成とバインド
let mut context = ssruntime::Context::new();
context.bind_resource(&resource);
```

### 💡 メモリ管理とライフタイム (Reference Counting)
最新の `libssruntime` では、Resource の内部管理に参照カウント (`Arc`) が導入されており、Player 側でのメモリ管理の負担を軽減しています。

#### 所有権と破棄のルール
| オブジェクト | 所有権の持ち方 | 破棄の影響 |
| :--- | :--- | :--- |
| **SsabData (バイナリ)** | **Player 側** (Borrow 時) または **Library 側** (Copy 時) | Borrow の場合、Resource 破棄まで維持が必要。 |
| **Resource (メタデータ)** | **Library 側** (参照カウント管理) | `ss_resource_destroy` でカウント減。Context があれば存続。 |
| **Context (再生状態)** | **Player 側** (明示的に破棄が必要) | `ss_runtime_destroy` で Context 本身と参照先リソースを解放。 |

#### オブジェクトの相関図
```mermaid
graph TD
    subgraph Owner ["アプリ側 (オーナー)"]
        Raw[バイナリデータ]
    end

    subgraph Lib ["libssruntime"]
        Res[AnimationResource]
        CtxA[Context A]
        CtxB[Context B]
    end

    Raw -->|A: Borrow / ポインタ参照| Res
    Raw -.->|B: Copy / 内部で複製| Res
    
    Res ==>|Bind| CtxA
    Res ==>|Bind| CtxB

    classDef owner fill:#f9f,stroke:#333;
    class Raw owner;
```

1. **バインド後の安全性**: `ss_runtime_bind_resource` を実行した後は、`ss_resource_destroy` を呼び出しても、そのリソースを参照している Context が存在し続ける限りメモリは解放されません。
2. **安全な破棄の順序**: 
    - 基本的には **Context → Resource** の順で破棄することを推奨しますが、ライブラリ内部で参照カウントを管理しているため、逆の順序（Resource → Context）で破棄しても Use-after-free は発生しません。
    - ただし、`ss_resource_create_borrow` を使用した場合は、**Resource が完全に解放される（全ての Context が破棄される）まで、元のバイナリデータ（`ssab_data_ptr`）のメモリを解放しないでください**。

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
Player の破棄時には、使用していた Context と Resource を破棄してメモリを解放します。
前述の通り内部で参照カウントを利用しているため、破棄の順序（Context が先か、Resource が先か）によるクラッシュの危険性はありませんが、明示的なリソース管理のために両方を適切に破棄してください。

```cpp
// コンテキスト（再生状態）を破棄する
ss_runtime_destroy(context);

// リソース（メタデータ）を破棄する
ss_resource_destroy(resource);
```

---
[目次に戻る](./README.ja.md)
