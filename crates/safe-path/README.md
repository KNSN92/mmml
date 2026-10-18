# Safe Path クレート

このクレートは、相対パスの正規化、ファイル名のサニタイズ、基準ディレクトリ配下への安全なパス結合を行う小さなワークスペース内ライブラリです。現在は主に `mc-resources` クレートのダウンロード先を組み立てるために使われています。

## 主な機能

- `SanitizeNew`: パスの各要素をファイル名としてサニタイズした `PathBuf` を作成
- `NormalizePath`: `.` と `..` を処理する字句的なパス正規化
- `SafeJoin`: 絶対パスと基準ディレクトリの外へ出る `..` を拒否して相対パスを結合

## 使い方

```rust
use std::path::{Path, PathBuf};
use safe_path::{NormalizePath, SafeJoin, SanitizeNew};

let safe_name = PathBuf::sanitize_new(r#"mods\example.jar"#);
let target = Path::new("game").safe_join(&safe_name)?;
assert_eq!(target, Path::new("game/mods/example.jar"));

let normalized = Path::new("mods/../resourcepacks").norm_path();
assert_eq!(normalized, Path::new("resourcepacks"));
# Ok::<(), safe_path::SafeJoinError>(())
```

`SafeJoin` は入力パスの構成要素を確認し、処理中に基準ディレクトリより上へ出ようとした場合に `SafeJoinError::PathTraversal` を返します。絶対パスは `SafeJoinError::AbsolutePath` になります。

## ファイル構成

| パス | 内容 |
| --- | --- |
| `Cargo.toml` | `sanitize-filename` と `thiserror` への依存関係 |
| `src/lib.rs` | `SanitizeNew`、`NormalizePath`、`SafeJoin` とそのエラー型 |

## テスト

```sh
cargo test -p safe-path
```

テストでは、`.` と内部の `..` の正規化、パストラバーサルと絶対パスの拒否を確認します。
