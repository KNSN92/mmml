# Platform クレート

このクレートは、実行中の OS、CPU architecture、OS version を共通の型で扱うワークスペース内ライブラリです。`java` クレートの Java 配布物選択や、`mc-resources` クレートの Minecraft ルール・native library 解決で利用します。

## 主な機能

- `Os::current()` による OS の判定
- `Arch::current()` による CPU architecture の判定
- `Os::version()` による OS version の取得
- 判定対象外の OS と architecture を `Other` バリアントで保持

## 使い方

```rust
use platform::{Arch, Os};

match (Os::current(), Arch::current()) {
    (Os::Windows, Arch::X64) => println!("Windows x64"),
    (Os::MacOS, Arch::Aarch64) => println!("macOS arm64"),
    (Os::Linux, Arch::X64) => println!("Linux x64"),
    (os, arch) => println!("Other platform: {os:?}, {arch:?}"),
}

if let Some(version) = Os::current().version() {
    println!("OS version: {version}");
}
```

対応する OS は `Windows`、`MacOS`、`Linux` です。それ以外は `Os::Other(&'static str)` になります。architecture は `X86`、`X64`、`Arm`、`Aarch64` を認識し、それ以外は `Arch::Other(&'static str)` になります。

`Os::version()` は現在の実 OS と一致する `Os` に対してのみ値を返します。別 OS を表す値や、検出できない OS version に対しては `None` を返します。

## ファイル構成

| パス | 内容 |
| --- | --- |
| `Cargo.toml` | OS version 検出用依存関係 |
| `src/lib.rs` | `Os` と `Arch` の定義、現在の環境の検出 |

## 利用クレート

- `java`: Temurin/GraalVM の配布アーカイブと Java 実行ファイル位置の選択
- `mc-resources`: Minecraft の OS ルールと native classifier の解決

このクレートは環境情報を提供します。特定の配布元や Minecraft の OS 名・architecture 名への変換は、利用側で行います。
