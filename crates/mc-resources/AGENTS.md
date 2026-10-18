# MC Resources クレートの概要

このディレクトリは、ワークスペース内の Cargo パッケージ `mc-resources` です。Minecraft Java Edition の version manifest、version JSON、アセット、ライブラリなどのモデルと、これらのリソースの解決・取得を含みます。

## 主な API

- `VersionManifest` は Mojang のバージョン一覧を表し、latest release/snapshot や指定 ID の検索を提供します。`fetch_source()` は manifest のダウンロード元を表す値を返し、取得自体は `ResourceDownloader::fetch()` が行います。最新バージョン ID など検索用の内部状態は非公開です。
- `version::VersionInfo` と関連モデルはバージョン JSON、client download、asset index、Java version、logging の情報を表します。
- `asset::AssetIndex` はアセット名と `AssetObject` の対応を保持します。`AssetObject` はアセットサーバー上の相対パス、サイズ、SHA-1 をダウンロード処理に提供します。
- `download::ResourceDownloader` は JSON manifest の fetch、ストリームへの download、ファイルへの download を提供します。応答のサイズと SHA-1 を確認し、ファイル保存時は一時ファイルを使います。
- `download::DownloadableResource`、`DownloadableManifest`、`DownloadableFile` は、ダウンロード可能なモデルが URL・サイズ・ハッシュ・保存先を提供するためのトレイトです。
- `version::RuleContext` と `RuleList` は OS、architecture、feature に基づく Mojang のルール評価を提供します。個々のルール条件は JSON の内部表現です。
- `version::CompatibleArguments` は旧形式の文字列引数と新形式の構造化引数を読み込み、`version::ResolvedArguments` として解決済みの JVM/game 引数を返します。`Argument` や `Arguments` も `version` 直下から参照できます。
- `version::Libraries` はルール適用後のライブラリと OS 別 native classifier を `version::ResolvedLibrary` として返します。ライブラリのデータモデルは `version` 直下から参照できます。
- `variable::VariableResolver` は引数や classifier 名に含まれる `${...}` 変数を呼び出し側の値で置き換えます。
- `hash::ResourceHash` は20バイトの SHA-1 値を表し、文字列との相互変換やチャンク単位の計算を提供します。

## ファイル構成

| パス | 内容 |
| --- | --- |
| `Cargo.toml` | 依存関係、Tokio feature、`debug` feature |
| `src/lib.rs` | 公開モジュールと主要型のクレート直下再公開 |
| `src/manifest.rs` | Mojang version manifest のモデルと検索機能 |
| `src/version/mod.rs` | version JSON のモデルと downloadable resource 実装 |
| `src/version/args.rs` | 旧形式・新形式の引数モデルと解決処理 |
| `src/version/rule.rs` | rule モデルと OS・feature 条件の評価 |
| `src/version/library.rs` | library モデルと artifact/native 解決 |
| `src/asset.rs` | asset index と asset object |
| `src/hash.rs` | SHA-1 型と計算処理 |
| `src/variable.rs` | 変数置換 |
| `src/download.rs` | HTTP fetch/download、整合性検証、保存処理 |
| `examples/` | Mojang metadata、asset、library の取得例 |
| `tests/download.rs` | ローカル HTTP モックを使った download の統合テスト |
| `tests/models.rs` | モデルのデシリアライズと lookup の統合テスト |

## 実装範囲

- データモデルは Mojang の version manifest と version JSON を主な対象とします。
- Mojang の argument rules と library 定義の評価に必要な platform 情報は、ワークスペース内の `platform` クレートから受け取ります。
- 保存先パスの結合と traversal 拒否には、ワークスペース内の `safe-path` クレートを使います。
- `debug` feature を有効にすると、Serde モデルで未知フィールドを拒否し、JSON のエラー位置にフィールドパスを含めます。
- データとして扱うレコードのフィールドは公開し、検索・評価・解決の内部状態はメソッドを介して扱います。内部状態のためだけの getter は設けません。
- 現在 `OsVersionRange` はデシリアライズ用モデルのみで、範囲判定は行いません。
- HTTP 統合テストはモックサーバーを利用し、Mojang への実通信に依存しません。
