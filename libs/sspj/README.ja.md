[**English**](./README.md) | [**日本語**](./README.ja.md)

# libsspj

SpriteStudio のプロジェクトファイル (`.sspj`) および、それに関連する XML アセットを解析するための Rust ライブラリです。
`quick-xml` と `serde` を使用して、XML 構造を型安全な Rust の構造体にデシリアライズします。

# 対応フォーマット

このライブラリは、SpriteStudio プロジェクトを構成する以下のファイルタイプの解析に対応しています。

- **SSPJ (`.sspj`)**: プロジェクト設定およびファイル参照情報。
- **SSAE (`.ssae`)**: アニメーションデータ。
- **SSCE (`.ssce`)**: セルマップ定義 (スプライトシート情報)。
- **SSEE (`.ssee`)**: パーティクルエフェクトデータ。
- **SSQE (`.ssqe`)**: シーケンスデータ。
- **SSSE (`.ssse`)**: サウンドエフェクトリスト。
- **BitmapFont (`.fnt` / `.json`)**: 文字マップ (`sscharmap` モジュール経由で解析)。

# インストール

`Cargo.toml` に以下のように記述して使用します。

```toml
[dependencies]
sspj = "7.0.0-beta.1"
```

# 使い方

メインのエントリポイントは `SsProject` 構造体です。`SsLoader` トレイトを使用することでプロジェクトファイルを読み込むことができます。
`.sspj` ファイルをロードする際、そこから参照されている外部アセット（アニメーションパック、セルマップ、エフェクトファイルなど）は、`rayon` を使用して自動的に**並列ロード**されます。

```rust
use std::path::Path;
use sspj::sspj::SsProject;
use sspj::ssloader::SsLoader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("path/to/project.sspj");

    // プロジェクトと、参照されているすべてのファイルをロード
    let project = SsProject::load_from_file(path)?;

    println!("Project Name: {}", project.name);
    println!("Version: {}", project.version);

    // ロードされたリソースへのアクセス
    for anime_pack in &project.anime_list {
        println!("Loaded AnimePack: {}", anime_pack.name);
        for anime in &anime_pack.anime_list {
             println!(" - Animation: {} (Frames: {})", anime.name, anime.settings.frame_count);
        }
    }

    for cell_map in &project.cellmap_list {
        println!("Loaded CellMap: {}", cell_map.name);
    }

    Ok(())
}
```
# 特徴

- **完全な型サポート**: SpriteStudio の内部型（補間タイプ、ブレンドモード、各属性値など）を Rust の enum や struct にマッピングしており、安全に扱えます。
- **並列ロード**: `rayon` を活用し、`.ssae` や `.ssce` などの依存ファイルを並列に読み込むことで、ロード時間を短縮しています。
- **Serde 統合**: すべてのデータ構造で `Deserialize` を導出（derive）しているため、他の Serde エコシステムのツールとも容易に統合可能です。

# アーキテクチャ

- `sspj::sspj`: ルートとなる `SsProject` 構造体の定義。
- `sspj::ssae`: アニメーションデータ、パーツ、キーフレームの定義。
- `sspj::ssattribute`: 個別の属性定義（座標、回転、スケール、ユーザーデータなど）。
- `sspj::ssce`: セルマップとセルの定義。
- `sspj::ssee`: エフェクトノードとパーティクル挙動の定義。
