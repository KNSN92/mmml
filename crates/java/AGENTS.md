# Java クレートの概要

このディレクトリは、ワークスペース内の Cargo パッケージ `java` です。Java ランタイムの配布物を取得・展開し、Java 実行ファイルを管理してプロセスを起動する機能を含みます。

現在では、Adoptium Temurin JRE と GraalVM JDK の 2 種類のディストリビューションをサポートしています。今後、追加のディストリビューションについても同様のインターフェースで扱えるように拡張する計画です。

## 公開 API

- `distribution::JavaDistribution` はディストリビューションの共通インターフェースです。サポート判定、インストール状態の確認、インストール、`JavaRuntime` の取得を提供します。
- `distribution::TemurinDistribution` は Adoptium Temurin JRE を扱います。
- `distribution::GraalVMDistribution` は GraalVM JDK を扱います。
- `JavaRuntime` は Java 実行ファイルのパスを保持し、`execute()` と `spawn()` で Java プロセスを起動します。
- `ExecutionParams` は子プロセスの環境変数と標準入出力の設定です。

## ファイル構成

| パス | 内容 |
| --- | --- |
| `Cargo.toml` | クレートの依存関係と Tokio の有効機能 |
| `src/lib.rs` | クレートの公開モジュールと OS／CPU 判定マクロ |
| `src/runtime.rs` | Java 実行ファイルの表現、プロセス実行、起動パラメーター |
| `src/extract.rs` | ZIP と gzip 圧縮 TAR の非同期展開 |
| `src/distribution/mod.rs` | 共通ディストリビューション API、共通インストール処理、エラー型 |
| `src/distribution/temurin.rs` | Temurin の対応判定、配布 URL、実行ファイル位置 |
| `src/distribution/graalvm.rs` | GraalVM の対応判定、配布 URL、実行ファイル位置 |
| `examples/` | 個別ディストリビューションの実行例と HelloWorld のサンプル |
| `tests/distributions.rs` | インストールと Java 実行の結合テスト |
| `tests/*.java`, `tests/*.class` | 結合テストで実行する Java プログラムとコンパイル済みクラス |

## 実装上の範囲

- 対応 OS は Windows、macOS、Linux です。
- インストール先のパスは各ディストリビューションのコンストラクターで指定されます。
- インストールを伴う結合テストは外部ネットワークから大きなアーカイブを取得するため `ignore` 指定されています。通常のテストではネットワーク不要のケースが実行されます。
