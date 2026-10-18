# Platform クレートの概要

このディレクトリは、ワークスペース内の Cargo パッケージ `platform` です。実行環境の OS・CPU architecture・OS version を共通の型で提供し、`java` と `mc-resources` などのクレートがプラットフォームを判定する際に利用します。

## 公開 API

- `Os` は `Windows`、`MacOS`、`Linux`、`Other(&'static str)` を表します。`Os::current()` は現在の実行環境の OS を返し、`Os::version()` は現在の OS と一致する値に対して検出済みの version を返します。検出不能または別 OS の場合は `None` です。
- `Arch` は `X86`、`X64`、`Arm`、`Aarch64`、`Other(&'static str)` を表します。`Arch::current()` は現在の実行環境の architecture を返します。
- OS と architecture の値は `Copy`、`Clone`、`PartialEq`、`Eq` を実装しています。

## ファイル構成

| パス | 内容 |
| --- | --- |
| `Cargo.toml` | `os-version` 依存関係 |
| `src/lib.rs` | OS／architecture の列挙型、現在の環境と OS version の検出 |

## 実装範囲

- 現在認識する OS 名は Rust の `std::env::consts::OS` に基づく Windows、macOS、Linux です。それ以外は `Os::Other` に保持します。
- architecture は Rust の `std::env::consts::ARCH` に基づいて x86、x86_64、arm、aarch64 を識別し、それ以外は `Arch::Other` に保持します。
- OS version の取得は `os-version` クレートに委譲します。取得に失敗した場合や値を提供しない OS の場合は `None` です。
- 配布元ごとの platform 表記への変換や、OS・architecture の対応可否判定はこのクレートの責務に含みません。
