# Java クレート

このクレートは、Java ランタイムのダウンロードとインストール、および Java プロセスの起動を提供するワークスペース内ライブラリです。現在は Eclipse Temurin と GraalVM を扱います。

## 主な機能

- Temurin JRE と GraalVM JDK のインストール
- OS と CPU アーキテクチャに応じた配布アーカイブの選択
- ZIP と tar.gz の非同期展開
- Java 実行ファイルのパス解決
- Java プロセスの終了待ち、または子プロセスとしての起動
- 子プロセスの環境変数と標準入出力の指定

インストール先は利用側が指定します。`is_installed()` は、その場所にディストリビューション固有の Java 実行ファイルがあるかを確認します。

## 対応ディストリビューション

| 型 | 配布元・内容 | バージョンの扱い |
| --- | --- | --- |
| `TemurinDistribution` | Adoptium の Temurin JRE | Java 8 は macOS arm64 を除いてサポート判定されます。 |
| `GraalVMDistribution` | Oracle の GraalVM JDK | Java 17 以降をサポート判定します。Java 17 は 17.0.12、18 以降は指定したメジャーバージョンの latest を取得します。 |

現在の実装が扱う OS は Windows、macOS、Linux です。CPU アーキテクチャやバージョンによって、配布元に実際のアーカイブがない場合があります。

Java 実行ファイルへの相対パスは次のとおりです。

| OS | パス |
| --- | --- |
| Windows | `bin/java.exe` |
| macOS | `Contents/Home/bin/java` |
| Linux | `bin/java` |

## 使い方

以下は Temurin 25 を必要に応じてインストールし、`java -version` を実行する例です。

```rust,no_run
use std::{error::Error, path::PathBuf, process::Stdio};

use java::distribution::{JavaDistribution, TemurinDistribution};
use java::ExecutionParams;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let install_path = PathBuf::from("./runtime/temurin-25");
    let distribution = TemurinDistribution::new(&install_path, 25);

    if !distribution.is_installed().await? {
        distribution.install().await?;
    }

    let runtime = distribution.runtime()?;
    let status = runtime
        .execute(
            ".",
            vec!["-version".into()],
            ExecutionParams {
                stdout: Stdio::inherit(),
                stderr: Stdio::inherit(),
                ..ExecutionParams::default()
            },
        )
        .await?;

    if !status.success() {
        return Err(format!("Java exited with {status}").into());
    }

    Ok(())
}
```

`JavaRuntime::execute()` はプロセスの終了まで待ち、`ExitStatus` を返します。プロセス出力を戻り値としては返しません。端末へ直接表示する場合は例のように `Stdio::inherit()` を指定します。stdin/stdout/stderr のパイプを呼び出し元から読み書きする場合は `JavaRuntime::spawn()` を使い、返された `tokio::process::Child` の非同期ハンドルを扱います。

`ExecutionParams` では環境変数、stdin、stdout、stderr を指定できます。標準設定は stdin が破棄、stdout と stderr が pipe です。`execute()` は Tokio の `Command::status()` を使うため、パイプを戻り値から読み取ることはできません。出力を取得・転送する場合は `ExecutionParams` で `std::io::pipe()` などを使うか `spawn()` と `Child` のパイプを利用してください。

## アーカイブ展開

- ZIP は `async_zip` を使って展開します。
- tar.gz は `async-compression` の gzip デコーダーと `tokio-tar` を組み合わせて展開します。
- 展開処理は `src/extract.rs` にあります。

配布アーカイブの取得にはネットワークアクセスが必要です。

## テスト

```sh
cargo test -p java
```

通常のテストでは、GraalVM の未対応バージョン判定などネットワーク不要のケースを実行します。実際にランタイムをダウンロードする結合テストは `ignore` 指定されています。Temurin と GraalVM のインストール、HelloWorld の実行、stdin/stdout/stderr、異常終了を確認するには、次を実行してください。

```sh
cargo test -p java --test distributions -- --ignored --test-threads=1
```

結合テストは各ディストリビューションの Java 25 アーカイブを取得するため、ネットワーク接続とダウンロード・展開用のディスク容量が必要です。
