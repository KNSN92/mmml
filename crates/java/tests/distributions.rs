use std::{
    error::Error,
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
};

use java::{
    ExecutionParams,
    distribution::{
        GraalVMDistribution, JavaDistribution, JavaDistributionError, TemurinDistribution,
    },
};
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

type TestResult<T = ()> = Result<T, Box<dyn Error + Send + Sync>>;

fn tests_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests")
}

async fn capture_spawn(
    runtime: &java::JavaRuntime,
    cwd: &Path,
    class_name: &str,
    stdin_data: Option<&[u8]>,
) -> TestResult<(ExitStatus, Vec<u8>, Vec<u8>)> {
    let stdin = if stdin_data.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    };
    let mut child = runtime.spawn(
        cwd,
        vec![class_name.to_owned()],
        ExecutionParams {
            stdin,
            stdout: Stdio::piped(),
            stderr: Stdio::piped(),
            ..ExecutionParams::default()
        },
    )?;

    let mut stdout = child.stdout.take().expect("stdout should be piped");
    let mut stderr = child.stderr.take().expect("stderr should be piped");
    let stdout_task = tokio::spawn(async move {
        let mut output = Vec::new();
        stdout.read_to_end(&mut output).await?;
        Ok::<_, std::io::Error>(output)
    });
    let stderr_task = tokio::spawn(async move {
        let mut output = Vec::new();
        stderr.read_to_end(&mut output).await?;
        Ok::<_, std::io::Error>(output)
    });

    if let Some(stdin_data) = stdin_data {
        let mut stdin = child.stdin.take().expect("stdin should be piped");
        stdin.write_all(stdin_data).await?;
        // Dropping stdin sends EOF so the Java program's readLine() can finish.
    }

    let status = child.wait().await?;
    let stdout = stdout_task.await??;
    let stderr = stderr_task.await??;
    Ok((status, stdout, stderr))
}

async fn install_and_check_distribution<D: JavaDistribution>(
    distribution: D,
    install_path: &Path,
) -> TestResult {
    assert!(!distribution.is_installed().await?);

    distribution.install().await?;

    assert!(distribution.is_installed().await?);
    assert!(matches!(
        distribution.install().await,
        Err(JavaDistributionError::AlreadyInstalled)
    ));

    let runtime = distribution.runtime()?;
    assert!(
        runtime.path().is_file(),
        "missing Java executable: {}",
        runtime.path().display()
    );

    let cwd = tests_dir();
    let hello_status = runtime
        .execute(
            &cwd,
            vec!["HelloWorld".into()],
            ExecutionParams {
                stdout: Stdio::null(),
                stderr: Stdio::null(),
                ..ExecutionParams::default()
            },
        )
        .await?;
    assert!(
        hello_status.success(),
        "HelloWorld exited with {hello_status}"
    );

    let (status, stdout, stderr) = capture_spawn(&runtime, &cwd, "HelloWorld", None).await?;
    assert!(
        status.success(),
        "HelloWorld exited with {status}: {}",
        String::from_utf8_lossy(&stderr)
    );
    assert_eq!(String::from_utf8_lossy(&stdout).trim(), "Hello, World!");

    let (status, stdout, stderr) =
        capture_spawn(&runtime, &cwd, "StdinEcho", Some(b"input from Rust\n")).await?;
    assert!(
        status.success(),
        "StdinEcho exited with {status}: {}",
        String::from_utf8_lossy(&stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&stdout).trim(),
        "received:input from Rust"
    );

    let (status, _, stderr) =
        capture_spawn(&runtime, &cwd, "MissingMainClassForTest", None).await?;
    assert!(
        !status.success(),
        "a missing main class should exit unsuccessfully"
    );
    assert!(
        !stderr.is_empty(),
        "Java should explain the failed launch on stderr"
    );

    assert!(install_path.exists());
    Ok(())
}

#[tokio::test]
#[ignore = "downloads and extracts a Temurin runtime"]
async fn temurin_install_and_runtime_io() -> TestResult {
    let temp_dir = TempDir::new()?;
    let install_path = temp_dir.path().join("temurin");
    let distribution = TemurinDistribution::new(&install_path, 25);
    install_and_check_distribution(distribution, &install_path).await
}

#[tokio::test]
#[ignore = "downloads and extracts a GraalVM runtime"]
async fn graalvm_install_and_runtime_io() -> TestResult {
    let temp_dir = TempDir::new()?;
    let install_path = temp_dir.path().join("graalvm");
    let distribution = GraalVMDistribution::new(&install_path, 25);
    install_and_check_distribution(distribution, &install_path).await
}

#[tokio::test]
async fn graalvm_rejects_unsupported_version_without_downloading() -> TestResult {
    let temp_dir = TempDir::new()?;
    let install_path = temp_dir.path().join("unsupported-graalvm");
    let distribution = GraalVMDistribution::new(&install_path, 16);

    assert!(matches!(
        distribution.install().await,
        Err(JavaDistributionError::UnsupportedVersion(16))
    ));
    assert!(!install_path.exists());
    Ok(())
}
