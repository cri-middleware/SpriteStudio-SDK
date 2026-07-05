> **日本語** | [English](./20_troubleshooting.md)

# トラブルシューティング

Player 実装中やビルド時に発生する、よくある問題とその解決方法です。現状はビルド・リンク時のプラットフォーム固有の問題を扱います。実行時のトラブルや他カテゴリの問題は、今後ここに追記します。

## Windows 環境でのリンクエラー (MSVC)

### LNK2019 / LNK2001: 未解決の外部シンボル

**症状**: `libssconverter.lib` や `libssruntime.lib` を Windows の C++ プロジェクトで利用する際にリンクで失敗し、`LNK2019` / `LNK2001` (未解決の外部シンボル) が出る。

**原因**: Rust 標準ライブラリが内部で参照している Windows API のシンボルが、最終バイナリのリンク時に解決されていない。

**対処**: プロジェクトのリンク設定に以下の Windows 標準ライブラリを追加してください。

- **Userenv.lib**
- **Bcrypt.lib**
- **Ntdll.lib**
- **Ws2_32.lib**

ビルド環境ごとの記述例:

- **Visual Studio**: プロジェクトのプロパティ → リンカー → 入力 → 追加の依存ファイル に上記 4 つを追加
- **SCons**: `env.Append(LIBS=["Userenv", "Bcrypt", "Ntdll", "Ws2_32"])`
- **CMake**: `target_link_libraries(<target> PRIVATE Userenv Bcrypt Ntdll Ws2_32)`

### LNK2005 / LNK1169: 重複シンボル (`ssruntime.lib` と `ssconverter.lib` の同時リンク)

**症状**: 両方の静的ライブラリを同じ最終バイナリ (例: Godot custom module / GDExtension の Windows 版など) にリンクすると、`link.exe` が `LNK2005` / `LNK1169` を出して失敗する。

```
ssruntime.windows.editor.x86_64.lib(...) : error LNK2005: ___rust_alloc は既に ssconverter.windows.editor.x86_64.lib(...) で定義されています。
godot.windows.editor.x86_64.exe : fatal error LNK1169: 1 つ以上の複数定義のシンボルが見つかりました。
```

**原因**: Rust の各 `staticlib` は自己完結したランタイム (アロケータシム / パニック関連シンボル) を内包するため、両方の `.lib` に `__rust_alloc` / `rust_begin_unwind` / `rust_eh_personality` 等が重複します。MSVC のリンカは duplicate symbol を error 扱いします (macOS の `ld` は warning に降格して通るため Windows のみ顕在化)。

**対処**: 最終バイナリ側のリンクフラグに `/FORCE:MULTIPLE` を追加します。

```python
# SCons (Godot custom module / GDExtension) の例
if env["platform"] == "windows":
    env.Append(LINKFLAGS=["/FORCE:MULTIPLE", "/IGNORE:4006"])
```

- `/FORCE:MULTIPLE` : `LNK2005` を warning (`LNK4006`) に降格してリンク続行
- `/IGNORE:4006` : その warning も抑止 (任意)

#### `/WX` を併用している場合 (`LNK4088` 対策)

godot-cpp 等が `/WX` (warning as error) を有効化していると、`/FORCE:MULTIPLE` で必ず出る `LNK4088` ("/FORCE オプションによってイメージが生成されています") が `LNK1218` に昇格してビルドが落ちます。

```
warning LNK4088: /FORCE オプションによってイメージが生成されています...
LINK : error LNK1218: 警告がエラーとして扱われました。
```

`LNK4088` は MSVC が `/IGNORE` で抑止できない warning のリストに含まれており、**`/IGNORE:4088` は効きません**。代わりに `/WX:NO` をリンカに渡して、リンク時の warning-as-error を打ち消してください。

```python
if env["platform"] == "windows":
    env.Append(LINKFLAGS=["/FORCE:MULTIPLE", "/IGNORE:4006", "/WX:NO"])
```

`/WX:NO` は **リンカ専用**の打ち消しフラグで、コンパイラ側 (`CCFLAGS` / `CXXFLAGS`) の `/WX` には影響しません。ソースコードの警告は引き続き error として検出されます。

> [!IMPORTANT]
> `/WX:NO` を入れると **以降のリンク時の警告がすべて素通り** します。リンクエラー (`LNK2005` 等) を全部解消した上で、最後にこのフラグを付与してください。先に `/WX:NO` を入れてしまうと、別の重要な警告も埋もれる可能性があります。

#### 補足: 動作上の影響

ワークスペースの `Cargo.toml` で `panic = "abort"` が指定されているため `rust_begin_unwind` / `rust_eh_personality` は実質 no-op、アロケータシムも双方が MSVCRT の `malloc` / `free` を呼ぶため機能等価です。どちらの定義が採用されてもバイナリの動作に差は出ません。

## macOS 環境でのリンク警告 (ld)

### duplicate symbol 警告 (`libssruntime.a` と `libssconverter.a` の同時リンク)

**症状**: 両方の静的ライブラリを同じ最終バイナリ (例: Godot custom module / GDExtension の macOS 版など) にリンクすると `ld` が duplicate symbol warning を出す。**リンク自体は成立** し、生成バイナリは正常動作する。

```
ld: warning: duplicate symbol '___rust_alloc' in:
    .../libssconverter.a[arm64](...)
    .../libssruntime.a[arm64](...)
```

**原因**: Windows 側 [LNK2005 / LNK1169 節](#lnk2005--lnk1169-重複シンボル-ssruntimelib-と-ssconverterlib-の同時リンク) と同じ。両 staticlib に Rust ランタイムが重複して含まれることに起因。

**対処**: `ld` は最初に見つけた定義を採用してリンクが成立するため、**追加対処は不要**です。動作上の影響については上記 [補足: 動作上の影響](#補足-動作上の影響) を参照してください。警告ログを抑止したい場合はビルドログ側で `grep -v "duplicate symbol"` する程度の対応が現実的です。

## 実装・実行時のよくあるトラブル

### ネイティブポインタのライフサイクルと並列処理の競合 (クラッシュ・Segfault)

**症状**: C# (Unity) 等の高水準言語側で管理する `SsPlayerHandle` などのネイティブポインタを破棄した際、またはアプリ終了時に、エンジン全体がアクセス違反 (Segfault) で強制終了する。

**原因**: Unity の Burst Job System のような別スレッド（ワーカー）の非同期ジョブにネイティブポインタやデータバッファを渡している状態で、メインスレッド側でそのポインタを `ss_player_release` 等で解放してしまうと競合が発生します。

**対処**: ネイティブポインタを破棄する前（または次フレームの更新前）に、必ずそのポインタを参照している非同期ジョブの完了を同期（例: `JobHandle.Complete()` を呼び出す）し、安全が保証された状態で `Dispose` や解放処理を行ってください。

### アニメーションが切り替わらない / 無音で再生が破綻する (ハッシュ計算の不一致)

**症状**: 実行時にエラーは出ないが、別のアニメーションが再生されたり、インスタンス（サブアニメ）パーツの参照が解決されず何も表示されなかったりする。

**原因**: アニメーション名やパーツ名からハッシュ値 (`uint32`) を生成する際、C# などの移植先言語で FNV-1a 等のアルゴリズムを**自前で再実装**し、その結果が Rust 側の `ss_runtime_hash_string` の出力と 1bit でもズレた場合に発生します。

**対処**: 文字列からのハッシュ生成は、**絶対に自前で実装せず、FFI 経由で提供されている `ss_runtime_hash_string(const char* s)` を必ず呼び出して使用**してください。

---
[目次に戻る](../README.ja.md)
