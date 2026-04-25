use crate::fabric_manifest_model::{FabricLibrary, FabricProfile};
use crate::models::{AssetIndexContent, VersionManifest};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

fn extract_zip(data: &[u8], target_dir: &Path, strip_toplevel: bool) -> anyhow::Result<()> {
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let Some(enclosed_name) = file.enclosed_name() else {
            continue;
        };
        let path = if strip_toplevel {
            let mut components = enclosed_name.components();
            components.next();
            components.as_path().to_path_buf()
        } else {
            enclosed_name.to_path_buf()
        };
        if path.as_os_str().is_empty() {
            continue;
        }
        let out_path = target_dir.join(&path);
        if file.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out_file = std::fs::File::create(&out_path)?;
            std::io::copy(&mut file, &mut out_file)?;
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub const JAVA_EXECUTABLE: &str = "java.exe";
#[cfg(not(target_os = "windows"))]
pub const JAVA_EXECUTABLE: &str = "java";

#[cfg(target_os = "windows")]
const CLASSPATH_SEPARATOR: &str = ";";
#[cfg(not(target_os = "windows"))]
const CLASSPATH_SEPARATOR: &str = ":";

pub async fn get_manifest() -> anyhow::Result<VersionManifest> {
    let url = "https://piston-meta.mojang.com/v1/packages/c9811ffdbcd77d79c12412836f21ed4e3c592102/1.20.1.json";

    // Create client with user agent
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    let response = client.get(url).send().await?;

    let text = response.text().await?;

    if text.is_empty() {
        return Err(anyhow::anyhow!("Server did not return text"));
    }

    println!(
        "Firsts 50 characters received: {}",
        &text[..std::cmp::min(50, text.len())]
    );

    let manifest: VersionManifest = serde_json::from_str(&text).map_err(|e| {
        // Error management
        anyhow::anyhow!(
            "Error parsing JSON: {} | Content: {}",
            e,
            &text[..std::cmp::min(100, text.len())]
        )
    })?;

    Ok(manifest)
}

pub async fn listar_librerias() -> anyhow::Result<()> {
    let manifest = get_manifest().await?;

    for lib in manifest.libraries {
        if let Some(artifact) = lib.downloads.artifact {
            println!("Library: {}", lib.name);
            println!("  -> URL: {}", artifact.url);
            if let Some(path) = artifact.path {
                println!("  -> Path: {}", path);
            }
        } else {
            println!("Library with no artifact: {}", lib.name);
        }
    }
    Ok(())
}

pub async fn download_libraries(
    manifest: &VersionManifest,
    base_path: &Path,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let libraries_dir = base_path.join("libraries");

    println!("Downloading libraries in: {:?}", libraries_dir);

    for lib in &manifest.libraries {
        // Only downloads when artifact is in
        if let Some(artifact) = &lib.downloads.artifact {
            let relative_path = artifact
                .path
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
            let target_path = libraries_dir.join(relative_path);

            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent).await?;
            }

            if !target_path.exists() {
                println!("Downloading: {}", lib.name);
                let bytes = client.get(&artifact.url).send().await?.bytes().await?;
                let mut file = fs::File::create(&target_path).await?;
                file.write_all(&bytes).await?;
            }
        }
    }

    println!("All libraries are ready!");
    Ok(())
}

pub async fn download_client(
    manifest: &VersionManifest,
    base_path: &std::path::Path,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let version_dir = base_path.join("versions").join(&manifest.id);
    let target_path = version_dir.join(format!("{}.jar", manifest.id));

    fs::create_dir_all(&version_dir).await?;

    // Download file
    if !target_path.exists() {
        println!("Downloading game code (client.jar)...");

        let url = &manifest.downloads.client.url;
        let bytes = client.get(url).send().await?.bytes().await?;

        let mut file = fs::File::create(&target_path).await?;
        file.write_all(&bytes).await?;

        println!(
            "client.jar downloaded successfully ({} MB)",
            bytes.len() / 1_024 / 1_024
        );
    } else {
        println!("Client.jar already exists, skiping download.");
    }

    Ok(())
}

pub fn gen_classpath(manifest: &VersionManifest, base_path: &std::path::Path) -> String {
    let mut cp_parts = Vec::new();
    let libraries_dir = base_path.join("libraries");

    let separador_cp = CLASSPATH_SEPARATOR;

    // Add Vanilla libraries
    for lib in &manifest.libraries {
        if let Some(artifact) = &lib.downloads.artifact {
            if let Some(rel_path) = &artifact.path {
                let absolute_path = libraries_dir.join(rel_path);
                if let Some(path_str) = absolute_path.to_str() {
                    cp_parts.push(path_str.to_string());
                }
            }
        }
    }

    // Add client.jar at the end
    let client_jar = base_path
        .join("versions")
        .join(&manifest.id)
        .join(format!("{}.jar", manifest.id));
    if let Some(path_str) = client_jar.to_str() {
        cp_parts.push(path_str.to_string());
    }

    cp_parts.join(separador_cp)
}

// ------------------------------------- ASSETS

pub async fn download_assets(
    manifest: &VersionManifest,
    base_path: &std::path::Path,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let assets_dir = base_path.join("assets");
    let objects_dir = assets_dir.join("objects");
    let indexes_dir = assets_dir.join("indexes");

    fs::create_dir_all(&indexes_dir).await?;
    fs::create_dir_all(&objects_dir).await?;

    let index_url = &manifest.asset_index.url;
    let index_path = indexes_dir.join(format!("{}.json", manifest.asset_index.id));

    let index_content = if index_path.exists() {
        fs::read_to_string(&index_path).await?
    } else {
        println!("Downloading assets index (5.json)...");
        let content = client.get(index_url).send().await?.text().await?;
        fs::write(&index_path, &content).await?;
        content
    };

    let index_data: AssetIndexContent = serde_json::from_str(&index_content)
        .map_err(|e| anyhow::anyhow!("Error parsing assets index: {}", e))?;

    println!("Verifying {} assets...", index_data.objects.len());

    // Download loop
    for (_name, obj) in index_data.objects {
        let prefix = &obj.hash[..2];
        let url = format!(
            "https://resources.download.minecraft.net/{}/{}",
            prefix, obj.hash
        );
        let folder = objects_dir.join(prefix);
        let file_path = folder.join(&obj.hash);

        if !file_path.exists() {
            fs::create_dir_all(&folder).await?;

            if let Ok(res) = client.get(&url).send().await {
                if let Ok(bytes) = res.bytes().await {
                    let _ = fs::write(&file_path, &bytes).await;
                }
            }
        }
    }

    println!("All assets are ready to load up.");
    Ok(())
}

// ------------------------------------------ FABRIC

pub async fn get_fabric_manifest() -> anyhow::Result<FabricProfile> {
    let url = "https://meta.fabricmc.net/v2/versions/loader/1.20.1/0.19.2/profile/json";

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    let response = client.get(url).send().await?;

    let manifest: FabricProfile = response.json().await?;

    Ok(manifest)
}

pub fn gen_fabric_path(lib: &FabricLibrary) -> std::path::PathBuf {
    let parts: Vec<&str> = lib.name.split(':').collect();
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    let jar_name = format!("{}-{}.jar", artifact, version);

    // Build path natively using PathBuf components
    let mut path = std::path::PathBuf::new();
    for part in group.split('/') {
        path.push(part);
    }
    path.push(artifact);
    path.push(version);
    path.push(jar_name);
    path
}

pub async fn download_fabric_libraries(
    manifest_fabric: &FabricProfile,
    base_path: &Path,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let libraries_dir = base_path.join("libraries");

    println!("Starting download of Fabric libraries...");

    for lib in &manifest_fabric.libraries {
        let relative_path_buf = gen_fabric_path(lib);

        // Native path for the OS
        let target_path = libraries_dir.join(&relative_path_buf);

        // URL path (always forward slashes)
        let url_path = relative_path_buf.to_string_lossy().replace('\\', "/");
        let download_url = format!("{}{}", lib.url, url_path);

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        if !target_path.exists() {
            println!("Downloading Fabric Lib: {}", lib.name);

            let response = client.get(&download_url).send().await?;
            if response.status().is_success() {
                let bytes = response.bytes().await?;
                let mut file = fs::File::create(&target_path).await?;
                file.write_all(&bytes).await?;
            } else {
                println!("Error 404 in Fabric Lib: {}", download_url);
            }
        }
    }

    println!("Fabric libraries downloaded successfully!");
    Ok(())
}

pub fn gen_cp_fabric(
    manifest_mc: &VersionManifest,
    manifest_fabric: &FabricProfile,
    base_path: &std::path::Path,
) -> String {
    let mut cp_parts = Vec::new();
    let libraries_dir = base_path.join("libraries");

    let classpath_separator = CLASSPATH_SEPARATOR;

    // Process Vanilla libraries
    for lib in &manifest_mc.libraries {
        if let Some(artifact) = &lib.downloads.artifact {
            if let Some(path) = &artifact.path {
                let full_path = libraries_dir.join(path);
                if let Some(path_str) = full_path.to_str() {
                    cp_parts.push(path_str.to_string());
                }
            }
        }
    }

    // Process Fabric libraries
    for lib in &manifest_fabric.libraries {
        let relative_path_buf = gen_fabric_path(lib);

        // PathBuf handles joining natively for each OS
        let full_path = libraries_dir.join(relative_path_buf);

        if let Some(path_str) = full_path.to_str() {
            cp_parts.push(path_str.to_string());
        }
    }

    // Add Client JAR at the end
    let client_jar = base_path
        .join("versions")
        .join(&manifest_mc.id)
        .join(format!("{}.jar", manifest_mc.id));

    if let Some(path_str) = client_jar.to_str() {
        cp_parts.push(path_str.to_string());
    }

    cp_parts.join(classpath_separator)
}

// --------------------------------- INJECT MODPACK

pub async fn inject_modpack(url: &str, base_path: &std::path::Path) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    println!("Downloading modpack from: {}", url);

    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;

    let mods_dir = base_path.join("mods");
    let config_dir = base_path.join("config");

    if mods_dir.exists() {
        fs::remove_dir_all(&mods_dir).await?;
    }
    if config_dir.exists() {
        fs::remove_dir_all(&config_dir).await?;
    }

    let target_dir = base_path;

    println!("Extracting files in {:?}...", target_dir);

    extract_zip(&bytes, target_dir, false)?;

    println!("Modpack injected successfully.");
    Ok(())
}

// --------------------------------- MULTIPLATFORM

use directories::ProjectDirs;
use std::process::Command;

pub fn base_path() -> std::path::PathBuf {
    // Win: C:\Users\Nombre\AppData\Roaming\OxideMC\data
    // Lin: /home/nombre/.local/share/oxidemc
    // Mac: /Users/Nombre/Library/Application Support/OxideMC
    if let Some(proj_dirs) = ProjectDirs::from("com", "s3fflex", "oxidemc") {
        return proj_dirs.data_dir().to_path_buf();
    }
    std::path::PathBuf::from(".minecraft")
}

// ----------------------------------------- JAVA

#[cfg(target_os = "windows")]
fn java_download_url(version: i64) -> anyhow::Result<(&'static str, &'static str)> {
    match version {
        17 => Ok(("jdk-17.0.5+8", "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.5%2B8/OpenJDK17U-jdk_x64_windows_hotspot_17.0.5_8.zip")),
        21 => Ok(("jdk-21.0.8+9", "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.8%2B9/OpenJDK21U-jdk_x64_windows_hotspot_21.0.8_9.zip")),
        _ => Err(anyhow::anyhow!("Java version {} not supported", version)),
    }
}

#[cfg(target_os = "linux")]
fn java_download_url(version: i64) -> anyhow::Result<(&'static str, &'static str)> {
    match version {
        17 => Ok(("jdk-17.0.5+8", "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.5%2B8/OpenJDK17U-jdk_x64_linux_hotspot_17.0.5_8.tar.gz")),
        21 => Ok(("jdk-21.0.8+9", "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.8%2B9/OpenJDK21U-jdk_x64_linux_hotspot_21.0.8_9.tar.gz")),
        _ => Err(anyhow::anyhow!("Java version {} not supported", version)),
    }
}

#[cfg(target_os = "macos")]
fn java_download_url(version: i64) -> anyhow::Result<(&'static str, &'static str)> {
    match version {
        17 => Ok(("jdk-17.0.5+8", "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.5%2B8/OpenJDK17U-jdk_x64_mac_hotspot_17.0.5_8.tar.gz")),
        21 => Ok(("jdk-21.0.8+9", "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.8%2B9/OpenJDK21U-jdk_x64_mac_hotspot_21.0.8_9.tar.gz")),
        _ => Err(anyhow::anyhow!("Java version {} not supported", version)),
    }
}

#[cfg(target_os = "windows")]
fn extract_java_archive(data: &[u8], runtime_dir: &std::path::Path) -> anyhow::Result<()> {
    extract_zip(data, runtime_dir, true)?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn extract_java_archive(data: &[u8], runtime_dir: &std::path::Path) -> anyhow::Result<()> {
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

pub async fn download_java_runtime(
    base_path: &std::path::Path,
    version: i64,
) -> anyhow::Result<String> {
    let runtime_dir = base_path.join("runtime");

    let (full_name, url) = java_download_url(version)?;

    println!("Downloading JDK {} portable...", version);

    if runtime_dir.exists() {
        println!("Erasing old java...");
        fs::remove_dir_all(&runtime_dir).await?;
    }

    let client = reqwest::Client::new();
    let bytes = client.get(url).send().await?.bytes().await?;

    fs::create_dir_all(&runtime_dir).await?;
    extract_java_archive(&bytes, &runtime_dir)?;

    println!("Java successfully installed.");

    Ok(full_name.to_string())
}

pub fn check_java_version() -> anyhow::Result<i32> {
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
