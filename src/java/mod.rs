use crate::net::get_http_client;
use anyhow::Result;
use std::process::Command;
use tokio::fs;

#[cfg(target_os = "windows")]
fn java_download_url(version: i64) -> Result<(&'static str, &'static str)> {
    match version {
        8 => Ok(("jdk8u472-b08", "https://github.com/adoptium/temurin8-binaries/releases/download/jdk8u472-b08/OpenJDK8U-jdk_x64_windows_hotspot_8u472b08.zip")),
        17 => Ok(("jdk-17.0.5+8", "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.5%2B8/OpenJDK17U-jdk_x64_windows_hotspot_17.0.5_8.zip")),
        21 => Ok(("jdk-21.0.8+9", "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.8%2B9/OpenJDK21U-jdk_x64_windows_hotspot_21.0.8_9.zip")),
        _ => Err(anyhow::anyhow!("Java version {} not supported", version)),
    }
}

#[cfg(target_os = "linux")]
fn java_download_url(version: i64) -> Result<(&'static str, &'static str)> {
    match version {
        8 => Ok(("jdk8u472-b08", "https://github.com/adoptium/temurin8-binaries/releases/download/jdk8u472-b08/OpenJDK8U-jdk_x64_linux_hotspot_8u472b08.tar.gz")),
        17 => Ok(("jdk-17.0.5+8", "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.5%2B8/OpenJDK17U-jdk_x64_linux_hotspot_17.0.5_8.tar.gz")),
        21 => Ok(("jdk-21.0.8+9", "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.8%2B9/OpenJDK21U-jdk_x64_linux_hotspot_21.0.8_9.tar.gz")),
        _ => Err(anyhow::anyhow!("Java version {} not supported", version)),
    }
}

#[cfg(target_os = "macos")]
fn java_download_url(version: i64) -> Result<(&'static str, &'static str)> {
    match version {
        8 => Ok(("jdk8u472-b08", "https://github.com/adoptium/temurin8-binaries/releases/download/jdk8u472-b08/OpenJDK8U-jdk_x64_mac_hotspot_8u472b08.tar.gz")),
        17 => Ok(("jdk-17.0.5+8", "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.5%2B8/OpenJDK17U-jdk_x64_mac_hotspot_17.0.5_8.tar.gz")),
        21 => Ok(("jdk-21.0.8+9", "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.8%2B9/OpenJDK21U-jdk_x64_mac_hotspot_21.0.8_9.tar.gz")),
        _ => Err(anyhow::anyhow!("Java version {} not supported", version)),
    }
}

#[cfg(target_os = "windows")]
fn extract_java_archive(data: &[u8], runtime_dir: &std::path::Path) -> Result<()> {
    extract_zip(data, runtime_dir, true)?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn extract_java_archive(data: &[u8], runtime_dir: &std::path::Path) -> Result<()> {
    use std::path::PathBuf;

    let decoder = flate2::read::GzDecoder::new(data);
    let mut archive = tar::Archive::new(decoder);
    // Strip top-level directory (e.g. "jdk-17.0.5+8/") so bin/java lands directly in runtime_dir
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path: PathBuf = entry.path()?.into_owned();
        let stripped: PathBuf = path.components().skip(1).collect();
        if stripped.as_os_str().is_empty() {
            continue;
        }
        let target = runtime_dir.join(&stripped);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        entry.unpack(&target)?;
    }
    Ok(())
}

pub async fn download_java_runtime(base_path: &std::path::Path, version: i64) -> Result<String> {
    let runtime_dir = base_path.join("runtime");

    let (full_name, url) = java_download_url(version)?;

    println!("Downloading JDK {} portable...", version);

    if runtime_dir.exists() {
        println!("Erasing old java...");
        fs::remove_dir_all(&runtime_dir).await?;
    }

    let client = get_http_client();
    let bytes = client.get(url).send().await?.bytes().await?;

    fs::create_dir_all(&runtime_dir).await?;
    extract_java_archive(&bytes, &runtime_dir)?;

    println!("Java successfully installed.");

    Ok(full_name.to_string())
}

pub fn check_java_version() -> Result<i32> {
    let output = Command::new("java").arg("-version").output();
    println!("Checking Java version...");

    match output {
        Ok(out) => {
            let version_info = String::from_utf8_lossy(&out.stderr);
            println!(
                "Java detected: {}",
                version_info.lines().next().unwrap_or("unknown")
            );

            // Parse major version from e.g. `openjdk version "17.0.5" ...` or `"1.8.0_392"`
            let major = version_info
                .split('"')
                .nth(1)
                .and_then(|v| {
                    let first = v.split('.').next()?;
                    let num: i32 = first.parse().ok()?;
                    // Java 8 and earlier use "1.x" scheme
                    if num == 1 {
                        v.split('.').nth(1)?.parse().ok()
                    } else {
                        Some(num)
                    }
                })
                .unwrap_or(0);

            Ok(major)
        }
        Err(_) => Err(anyhow::anyhow!("Java not found in PATH")),
    }
}
