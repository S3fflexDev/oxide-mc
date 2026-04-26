pub(crate) mod models;

use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use crate::mc::models::VersionManifest;
use crate::net::get_http_client;

pub async fn get_manifest() -> anyhow::Result<VersionManifest> {
    let url = "https://piston-meta.mojang.com/v1/packages/900a4d828d608162c1061113b1176e656000cb45/1.21.1.json";

    let client = get_http_client();

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
    let client = get_http_client();
    let libraries_dir = base_path.join("libraries");

    println!("Downloading libraries in: {:?}", libraries_dir);

    for lib in &manifest.libraries {
        let Some(artifact) = &lib.downloads.artifact else {
            continue;
        };
        let relative_path = artifact
            .path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
        let target_path = libraries_dir.join(relative_path);

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        if target_path.exists() {
            continue;
        }
        println!("Downloading: {}", lib.name);
        let bytes = client.get(&artifact.url).send().await?.bytes().await?;
        let mut file = fs::File::create(&target_path).await?;
        file.write_all(&bytes).await?;
    }

    println!("All libraries are ready!");
    Ok(())
}

pub async fn download_client(
    manifest: &VersionManifest,
    base_path: &std::path::Path,
) -> anyhow::Result<()> {
    let client = get_http_client();

    let version_dir = base_path.join("versions").join(&manifest.id);
    let target_path = version_dir.join(format!("{}.jar", manifest.id));

    fs::create_dir_all(&version_dir).await?;

    if target_path.exists() {
        println!("Client.jar already exists, skipping download.");
        return Ok(());
    }

    println!("Downloading game code (client.jar)...");
    let url = &manifest.downloads.client.url;
    let bytes = client.get(url).send().await?.bytes().await?;

    let mut file = fs::File::create(&target_path).await?;
    file.write_all(&bytes).await?;

    println!(
        "client.jar downloaded successfully ({} MB)",
        bytes.len() / 1_024 / 1_024
    );

    Ok(())
}

pub(crate) fn collect_vanilla_cp(
    libraries: &[crate::mc::models::Library],
    libraries_dir: &Path,
    cp_parts: &mut Vec<String>,
) {
    for lib in libraries {
        let Some(artifact) = &lib.downloads.artifact else {
            continue;
        };
        let Some(rel_path) = &artifact.path else {
            continue;
        };
        let full_path = libraries_dir.join(rel_path);
        let Some(path_str) = full_path.to_str() else {
            continue;
        };
        cp_parts.push(path_str.to_string());
    }
}

pub fn gen_classpath(manifest: &VersionManifest, base_path: &std::path::Path) -> String {
    let mut cp_parts = Vec::new();
    let libraries_dir = base_path.join("libraries");

    collect_vanilla_cp(&manifest.libraries, &libraries_dir, &mut cp_parts);

    let client_jar = base_path
        .join("versions")
        .join(&manifest.id)
        .join(format!("{}.jar", manifest.id));
    if let Some(path_str) = client_jar.to_str() {
        cp_parts.push(path_str.to_string());
    }

    cp_parts.join(crate::platform::CLASSPATH_SEPARATOR)
}