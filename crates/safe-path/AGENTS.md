# Safe Path クレートの概要

このディレクトリは、ワークスペース内の Cargo パッケージ `safe-path` です。ファイル保存先などを組み立てる際に使う、パスのサニタイズ、字句的な正規化、安全な相対パス結合を提供します。`mc-resources` クレートは `ResourceDownloader::download_to_file()` の保存先検証にこのクレートを利用します。

## 公開 API

- `SanitizeNew` は `PathBuf` に実装されており、入力を `/` 区切りに扱った後、各要素を `sanitize-filename` でサニタイズします。
- `NormalizePath` は `AsRef<Path>` を実装する型に使えます。`.` を除き、`..` で直前の要素を取り除く字句的な正規化を行います。
- `SafeJoin` は `AsRef<Path>` を実装する基準パスに使えます。相対パスの要素を検査し、基準より上へ出る `..` は `SafeJoinError::PathTraversal`、絶対パスは `SafeJoinError::AbsolutePath` として拒否します。安全な相対パスは基準パスと結合した `PathBuf` を返します。
- `SafeJoinError` は安全な結合で拒否した理由を表すエラー型です。

## ファイル構成

| パス | 内容 |
| --- | --- |
| `Cargo.toml` | ファイル名サニタイズとエラー型の依存関係 |
| `src/lib.rs` | 公開 trait、パス処理、エラー型、単体テスト |

## 実装範囲

- パスの正規化と `SafeJoin` はファイルシステムを参照しない字句的な処理です。
- `SafeJoin` のチェックはシンボリックリンクの解決や実際のファイルシステム上の canonical path 検証を行いません。
- `SanitizeNew` はパス要素をファイル名向けにサニタイズします。既存のパスが安全かを判定する機能とは異なります。
- テストは `cargo test -p safe-path` で実行します。
