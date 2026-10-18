# MC Resources クレート

このクレートは、Minecraft Java Edition の Mojang メタデータを読み込み、バージョン情報、アセット、クライアント関連リソース、ライブラリを扱うワークスペース内ライブラリです。Serde モデル、OS と feature に応じたルール評価、起動引数とライブラリの解決、サイズ・SHA-1 を検証するダウンロード処理を提供します。

## 主な機能

- Mojang の version manifest と各バージョンの version JSON の読み込み
- アセットインデックスとアセットオブジェクトのモデル化
- リソースのサイズ・SHA-1 検証と、指定ディレクトリへの保存
- Mojang の library 定義、OS ルール、native classifier の解決
- 新形式と旧形式の起動引数の読み込み、ルール適用、変数置換
- `debug` feature 有効時の未知フィールド拒否と、Serde エラーのフィールドパス表示

## 使い方

version manifest を取得し、最新リリースの version JSON を読み込む例です。

```rust,no_run
use mc_resources::{ResourceDownloader, VersionManifest};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let downloader = ResourceDownloader::new(Client::new());
    let manifest = downloader.fetch(&VersionManifest::fetch_source()).await?;
    let latest = manifest
        .latest_release()
        .ok_or("latest release is missing from the manifest")?;
    let version_info: mc_resources::VersionInfo = downloader.fetch(latest).await?;

    println!("Minecraft {}", version_info.id);
    Ok(())
}
```

ファイルとしてダウンロードできるモデルは `DownloadableFile` を実装しています。`ResourceDownloader::download_to_file(base_path, resource)` はモデル内の相対パスを `base_path` に結合し、保存先を検証したうえでダウンロードします。ダウンロード完了後にサイズと SHA-1 を確認し、一時ファイルから保存先へ置き換えます。アセットはハッシュに基づくパス、ライブラリは Mojang metadata に記録されたパスを使います。

ライブラリや引数の解決には、実行環境に合わせた `RuleContext` と、`${...}` 形式の値を供給する `VariableResolver` を渡します。ランチャー固有の変数名や値は呼び出し側が設定します。

## 公開 API の使い分け

よく使う入口はクレート直下から参照できます。`ResourceDownloader`、`ResourceDownloadError`、`VersionManifest`、`Version`、`VersionType`、`VersionInfo` は `mc_resources::...` から利用できます。引数・ライブラリ・ルール関連の型は `mc_resources::version::...` から利用できます。従来の `mc_resources::download::...` や `mc_resources::version::args::...` などのモジュール経由のパスも引き続き使えます。

Serde のレコードとして利用する `Version`、`VersionInfo` とその情報モデル、`AssetObject`、ライブラリ定義などは、データフィールドを公開しています。一方、manifest の検索状態、アセット索引、ライブラリ一覧、ルール一覧、変数リゾルバーの内部状態は隠し、検索・解決メソッドを通して扱います。`Rule` や OS 条件のモデルは Mojang JSON の内部表現として扱い、公開 API にはしていません。

## ファイル構成

| パス | 内容 |
| --- | --- |
| `Cargo.toml` | クレートの依存関係と `debug` feature |
| `src/lib.rs` | 公開モジュール |
| `src/manifest.rs` | version manifest とバージョン一覧の検索 API |
| `src/version/mod.rs` | version JSON、client download、asset index、logging のモデル |
| `src/version/args.rs` | 旧形式・新形式の引数モデルと解決 |
| `src/version/rule.rs` | OS・architecture・feature・OS version に基づくルール評価 |
| `src/version/library.rs` | ライブラリ定義、artifact/native classifier の解決 |
| `src/asset.rs` | asset index と asset object のモデル |
| `src/hash.rs` | SHA-1 値の型、解析、表示、チャンク計算 |
| `src/variable.rs` | `${...}` 形式の変数置換 |
| `src/download.rs` | fetch/download API、検証、ファイル保存 |
| `examples/` | manifest、version、asset、library を取得する例 |
| `tests/` | HTTP モックを使った download とモデルの統合テスト |

## テスト

```sh
cargo test -p mc-resources
cargo test -p mc-resources --features debug
```

HTTP テストはローカルのモックサーバーを使うため、Mojang サーバーへの実通信を必要としません。
